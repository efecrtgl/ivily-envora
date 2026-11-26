# Kullanım kılavuzu

Bu kılavuz Ivily Envora'nın ekranlarını ve temel iş akışlarını özetler. Konu rehberleri için
[docs/rehber](rehber/) klasörüne bakın.

## Temel kavramlar

- **Kapsam:** Bir ortam değişkeninin nerede tanımlı olduğu — *İşlem* (yalnızca bir programın belleğinde),
  *Kullanıcı* (yalnızca sizin hesabınız, kalıcı) ve *Sistem* (tüm kullanıcılar, kalıcı, yönetici gerekir).
  Ayrıntılar: [Ortam değişkenleri rehberi](rehber/ORTAM-DEGISKENLERI.md).
- **Kalıcı değişiklik:** Kullanıcı veya sistem kapsamına yazılan her değişiklik. Envora her kalıcı değişikliği
  önizler, onay ister, öncesinde otomatik yedek (anlık görüntü) alır ve sonunda Windows'a ortamın değiştiğini
  bildirir. **Açık programlar yeni değeri görmek için yeniden başlatılmalıdır.**
- **Hassas değer:** Token, parola, anahtar gibi değerler. Maskelenir; yalnızca "Göster" ile görünür.

## Ekranlar

| Ekran | Ne yapar |
|-------|----------|
| **Pano** | Ortam özeti, PATH sağlığı, son analiz, güvenlik durumu, profiller, projeler, araçlar, son etkinlikler. Bölümler Ayarlar'dan açılıp kapatılabilir ve sıralanabilir. |
| **Değişkenler** | Tüm kapsamlardaki değişkenler. Arama, kategori ve kapsam süzgeci, yalnızca hassas/uyarılı süzgeci. Satıra tıklayınca ayrıntı: her kapsamdaki değer, genişletilmiş önizleme, uyarılar, hassasiyet ayarı, düzenle/çoğalt/yeniden adlandır/sil. |
| **PATH** | Kullanıcı, sistem ve işlem PATH'i. Girdi sorunları, uzunluk, taslak düzenleme, komut çakışmaları. |
| **Profiller** | Profil oluşturma, düzenleme, sürüm geçmişi, doğrulama, kalıcı uygulama ve geri alma. |
| **Projeler** | Proje dizinleri, ilişkili profil, projeye özgü değişkenler, başlatma ortamı önizlemesi, terminal/program başlatma. |
| **Analiz** | Ortamdaki olası sorunlar ve açıklamaları; kural ayarları. |
| **Araçlar** | Kurulu geliştirici araçları, sürümleri ve konumları. |
| **Şablonlar** | Hazır ortam önerileri; profil/projeye uygulama, kendi şablonlarınız. |
| **Karşılaştır** | İki kaynağı (kapsam, profil, proje, anlık görüntü, içe aktarılan dosya) karşılaştırma. |
| **Anlık görüntüler** | Elle ve otomatik yedekler; ayrıntı, karşılaştırma, seçimli geri yükleme, dışa aktarma. |
| **İçe / dışa aktar** | Değişken, PATH, profil, proje, anlık görüntü ve şablon aktarımı. |
| **Etkinlik** | Değer içermeyen yerel işlem kaydı. |
| **Ayarlar** | Dil, tema, yoğunluk, pano, güvenlik, yedekleme, etkinlik, güncellemeler, ekosistem, veri, tanılama. |

## Sık kullanılan iş akışları

### Yeni bir kullanıcı değişkeni eklemek

1. **Değişkenler → Yeni değişken**.
2. Hedef kapsamı (Kullanıcı), adı ve değeri girin. Tür "Otomatik" bırakılırsa `%AD%` başvurusu içeren
   değerler "Genişletilebilir" olarak kaydedilir.
3. **Değişiklikleri incele** → önizlemeyi kontrol edin → **Uygula**.
4. Açık terminalleri yeniden başlatın.

### Bir aracın dizinini PATH'e eklemek

1. **PATH → Kullanıcı** sekmesi.
2. Alttaki alana dizini yazın (ör. `%USERPROFILE%\tools\bin`), "Başa ekle" veya "Sona ekle" seçin, **Ekle**.
3. **Değişiklikleri incele** → eklenen girdiyi önizlemede görün → **Uygula**.

### Bir profili geçici olarak kullanmak

Kalıcı ortamı değiştirmek istemiyorsanız profili bir projeye bağlayın ve projeyi **Terminal aç** ile
başlatın: profil yalnızca o terminale uygulanır. Kalıcı uygulama için profilde **Kalıcı olarak uygula**,
geri dönmek için **Geri al** kullanılır.

### Bir değişikliği geri almak

Her kalıcı değişiklikten önce otomatik bir anlık görüntü alınır (Ayarlar'da kapatılmadıysa).
**Anlık görüntüler** → ilgili "Otomatik yedek" → **Tümünü geri yükle** veya yalnızca seçtiğiniz değişkenler.

## Klavye

- `Esc` açık pencereyi kapatır (işlem sürerken kapatılamaz).
- `Tab` pencereler içinde odağı döngüde tutar.
- Liste satırları ve kartlar `Enter` ile açılabilir.
