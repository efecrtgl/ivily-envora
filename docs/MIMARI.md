# Mimari

Ivily Envora; Rust (Tauri 2) arka ucu ve React (düz JavaScript) ön ucundan oluşan bir Windows masaüstü uygulamasıdır.
Bu belge katmanları, modül sınırlarını ve temel güvenlik kararlarını açıklar.

## Genel bakış

```
┌──────────────────────── WebView2 (React) ─────────────────────────┐
│ app/ (kabuk, gezinme, tema, i18n)   pages/ (ekranlar)              │
│ components/ui (tasarım sistemi)     lib/api.js (tek IPC kapısı)    │
└───────────────────────────────┬────────────────────────────────────┘
                                │ Tauri invoke (yalnızca tanımlı komutlar)
┌───────────────────────────────┴────────────────────────────────────┐
│ commands/   ince komut katmanı: doğrulama + servis çağrısı         │
├────────────────────────────────────────────────────────────────────┤
│ env/        model, kapsamlar, sağlayıcılar, değişiklik planları    │
│ pathmgr/    PATH ayrıştırma, analiz, düzenleme                     │
│ security/   sınıflandırma, maskeleme, DPAPI kasası                 │
│ profiles/   projects/   snapshots/   compare/   templates/         │
│ io/         JSON / CSV / .env içe-dışa aktarma                     │
│ analysis/   kural motoru        discovery/  araç keşfi             │
│ activity/   settings/   db/     iep/        update/   diagnostics/ │
├────────────────────────────────────────────────────────────────────┤
│ platform/   Windows kayıt defteri, WM_SETTINGCHANGE, yükseltme     │
└────────────────────────────────────────────────────────────────────┘
```

## Ortam kapsamları

| Kapsam | Kaynak | Kalıcı | Yazma |
|--------|--------|--------|-------|
| İşlem (Process) | Uygulamanın kendi işlem ortamı | Hayır | Uygulama kendi ortamını değiştirmez; işlem kapsamı yalnızca **başlatılan** süreçlere verilir (proje terminali). |
| Kullanıcı (User) | `HKCU\Environment` | Evet | Standart kullanıcı yetkisi yeterlidir. |
| Sistem (Machine) | `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment` | Evet | Yönetici yetkisi gerekir; yalnızca o değişiklik için yükseltilmiş yardımcı kip çalıştırılır. |

Kalıcı bir değişiklikten sonra `WM_SETTINGCHANGE("Environment")` yayınlanır. Bu mesajı dinleyen
uygulamalar (ör. Explorer) ortamlarını yeniler; **çalışan diğer uygulamalar eski ortamla devam eder** ve
yeni değerleri görmek için yeniden başlatılmaları gerekir. Arayüz bunu her değişiklikte açıkça belirtir.

## Değişiklik akışı (plan → önizleme → onay → uygulama)

Kalıcı ortam üzerinde yapılan her değişiklik bir `ChangePlan` nesnesiyle ifade edilir:

1. Ön uç bir plan ister (ör. "Kullanıcı kapsamında `JAVA_HOME` ayarla").
2. Arka uç planı doğrular (ad kuralları, değer uzunluğu, kapsam yetkisi) ve mevcut değerlerle farkı hesaplar.
3. Ön uç etkilenen kapsamı ve değişiklikleri (hassas değerler maskeli) gösterir; kullanıcı onaylar.
4. Arka uç, etkilenen değişkenlerin önceki hallerini otomatik bir anlık görüntüye kaydeder, ardından uygular.
5. Etkinlik kaydına değer içermeyen bir giriş eklenir.

Bir plan tek kapsama aittir; başka bir kapsamdaki değişken hiçbir zaman sessizce değiştirilmez.

## Sistem kapsamı ve yükseltme

Uygulama standart kullanıcı yetkisiyle çalışır. Sistem kapsamındaki bir plan onaylandığında:

1. Plan JSON olarak serileştirilir ve DPAPI (`CRYPTPROTECT_LOCAL_MACHINE`) ile şifrelenmiş geçici bir dosyaya yazılır.
2. Aynı yürütülebilir dosya `--envora-elevated-apply <dosya>` argümanıyla `runas` fiiliyle başlatılır (UAC istemi).
3. Yardımcı kip arayüz açmaz; planı çözer, **yalnızca sistem kapsamındaki ortam değişkeni ayarlama/silme**
   işlemlerine izin verir, uygular, sonucu yazar ve çıkar. Geçici dosya silinir.

Yardımcı kip rastgele komut çalıştırmaz; plan şeması dışındaki her girdiyi reddeder.

## Gizli değerler

- Sınıflandırıcı ad kalıplarına (`*_TOKEN`, `*SECRET*`, `*PASSWORD*`, `*_API_KEY` …) ve değer biçimlerine
  (ör. `ghp_`, `sk-`, `AKIA`, PEM başlıkları, kimlik bilgisi içeren URL'ler) bakar. Kullanıcı her değişken adı için
  sınıflandırmayı geçersiz kılabilir.
- Uygulama veritabanında saklanan hassas değerler (profil, proje, anlık görüntü) Windows DPAPI
  (`CryptProtectData`, geçerli kullanıcı) ile şifrelenir. Hassas olmayan değerler düz metin olarak saklanır.
- Etkinlik kaydı, hata mesajları ve tanılama raporları hiçbir zaman değişken **değeri** içermez.
- Sınırlamalar `docs/GUVENLIK.md` içinde açıklanır: DPAPI aynı Windows kullanıcısı olarak çalışan kötü amaçlı
  yazılımlara karşı koruma sağlamaz; kalıcı ortam değişkenleri kayıt defterinde zaten düz metin durur.

## Veri katmanı

SQLite (`rusqlite`, gömülü derleme). Veritabanı `%LOCALAPPDATA%\IvilyEnvora\envora.db` konumundadır.
Geçişler `PRAGMA user_version` ile sürümlenir ve tek bir işlem (transaction) içinde uygulanır. Ayrıntılar
`docs/VERITABANI.md` içindedir.

## Ön uç

- Yönlendirme kütüphanesi yoktur; `app/` içindeki gezinme durumu hangi sayfanın tembel (lazy) yükleneceğini belirler.
- Tüm metinler `src/i18n/tr.js` ve `src/i18n/en.js` içindedir; bileşenler yalnızca `t('anahtar')` kullanır.
- Tema, `<html data-theme>` özniteliği ve CSS değişkenleriyle uygulanır; tema değişimi uygulama verisine dokunmaz.
- Tüm arka uç çağrıları `src/lib/api.js` üzerinden geçer.

## Genişletilebilirlik

Analiz kuralları, araç tanımları, içe/dışa aktarma biçimleri ve şablonlar kayıt tabloları (registry) olarak
tasarlanmıştır; yeni bir kural/biçim eklemek tek bir dosya ve kayıt satırı gerektirir. Üçüncü taraf eklenti
sistemi **yoktur**; gelecekte eklenirse yetenek sınırları ve izinlerle tasarlanacaktır (bkz. `docs/IEP.md`).
