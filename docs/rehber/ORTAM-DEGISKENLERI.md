# Ortam değişkenleri rehberi

## Kapsamlar

| Kapsam | Nerede durur | Kim görür | Kalıcı mı | Yazmak için |
|--------|--------------|-----------|-----------|-------------|
| **İşlem** | Bir programın belleği | Yalnızca o program ve başlattığı alt programlar | Hayır | Program başlatılırken verilir |
| **Kullanıcı** | `HKCU\Environment` | Sizin hesabınızla başlatılan programlar | Evet | Standart yetki |
| **Sistem** | `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment` | Bilgisayardaki tüm kullanıcılar ve hizmetler | Evet | Yönetici (UAC) |

Envora'daki **İşlem** sekmesi, Envora başlatıldığı andaki kendi ortamını gösterir ve yalnızca okunur.
Bir değeri işlem kapsamına vermenin yolu proje başlatmadır (bkz. [Proje ortamları](PROJELER.md)).

### Hangi değer geçerli olur?

Yeni başlatılan bir program ortamını genellikle Explorer'dan alır:

- Aynı ad hem kullanıcı hem sistem kapsamında varsa **kullanıcı değeri** geçerlidir.
- **PATH** birleştirilir: önce sistem PATH, sonra kullanıcı PATH.
- Değişkenin ayrıntısında "Yeni başlatılan işlemlerin göreceği kalıcı değer" satırı bunu gösterir.

### Değişiklik neden hemen görünmüyor?

Windows ortamı bir program **başlatılırken** kopyalar. Kalıcı bir değişiklikten sonra Envora açık pencerelere
`WM_SETTINGCHANGE` bildirimi gönderir; Explorer bu bildirimle kendi ortamını yeniler, böylece Explorer'dan
**yeni** açılan programlar yeni değeri görür. Zaten açık olan terminaller, IDE'ler ve diğer programlar eski
değeri kullanmaya devam eder — **yeniden başlatılmaları gerekir**.

## Değer türleri

| Tür | Kayıt türü | Açıklama |
|-----|-----------|----------|
| Metin | `REG_SZ` | Değer olduğu gibi kullanılır. |
| Genişletilebilir | `REG_EXPAND_SZ` | `%USERPROFILE%` gibi başvurular Windows tarafından genişletilir. |

Envora mevcut bir değişkenin türünü korur. Yeni değişkende tür "Otomatik" seçilirse `%AD%` içeren değerler
genişletilebilir kaydedilir. `%AD%` içeren bir "Metin" değişkeni için uyarı gösterilir.

## Ad kuralları

- Boş olamaz, `=` ve denetim karakteri içeremez, en fazla 255 karakter.
- Windows'ta adlar **büyük/küçük harfe duyarsızdır** (`Path` ile `PATH` aynıdır).
- Uyarılar: baştaki/sondaki boşluk, rakamla başlama (POSIX kabuklarında kullanılamaz), harf/rakam/`_` dışı
  karakterler (bazı kabuklar erişemez).
- Bir değer en fazla 32.767 karakter olabilir; 2.047'yi aşan değerler bazı eski araçlarda sorun çıkarabilir.

## Hassas değişkenler

Envora; adında `TOKEN`, `SECRET`, `PASSWORD`, `API_KEY` gibi ifadeler geçen veya değeri bilinen anahtar
biçimlerine benzeyen değişkenleri **hassas** sayar:

- Listede maskelenir; değer yalnızca **Göster** ile istenir ve ayarlanan süre sonunda yeniden gizlenir.
- **Kopyala** değeri arayüze getirmeden panoya yazar; pano geçmişine alınmaz ve süre sonunda temizlenir.
- Algılama yanlışsa ayrıntıdan **Hassas / Hassas değil / Otomatik** seçin.

Not: Kalıcı ortam değişkenleri Windows tarafından kayıt defterinde düz metin saklanır. Gizli değerler için
projeye özgü değişkenler (DPAPI ile şifreli) daha güvenlidir. Bkz. [Güvenlik](../GUVENLIK.md).

## Değişiklik akışı

Oluşturma, düzenleme, çoğaltma, yeniden adlandırma ve silme işlemlerinin tümü aynı akıştan geçer:

1. **Önizleme:** etkilenen kapsam, her değişkenin önceki ve sonraki hali (hassaslar maskeli), uyarılar
   (ör. "aynı ad sistem kapsamında da var", "kritik sistem değişkeni").
2. **Onay:** sistem kapsamı için UAC istemi yalnızca bu adımda açılır.
3. **Yedek:** etkilenen değişkenlerin önceki hali otomatik anlık görüntüye kaydedilir.
4. **Uygulama:** bir adım başarısız olursa öncekiler geri alınır.
5. **Bildirim:** Windows'a ortamın değiştiği bildirilir; etkinlik kaydına değer içermeyen bir giriş eklenir.

Çoğaltma ve yeniden adlandırma, gizli değerleri arayüze getirmeden arka uçta kopyalar.
