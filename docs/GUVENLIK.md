# Güvenlik

Bu belge Ivily Envora'nın gizli değerleri nasıl ele aldığını, tehdit modelini ve **bilinen sınırlamaları**
açıklar. Güvenlik açığı bildirmek için [SECURITY.md](../SECURITY.md) belgesine bakın.

## Özet

| Konu | Davranış |
|------|----------|
| Hassas değer listeleri | Arayüze **gönderilmez**; yalnızca "Göster" ile, tek tek istenir ve bu istek etkinlik kaydına (değer olmadan) yazılır. |
| Gösterilen değer | Yalnızca o bileşenin belleğinde tutulur; ayarlanan sürede (varsayılan 30 sn) yeniden gizlenir. |
| Kopyalama | Değer arayüze gelmeden arka uçta panoya yazılır; pano geçmişine/bulut panosuna alınmaz; süre sonunda (değişmediyse) temizlenir. |
| Uygulama veritabanı | Profil, proje, anlık görüntü ve revizyonlardaki hassas değerler Windows DPAPI ile şifrelenir. |
| Etkinlik kaydı, hatalar, tanılama | Hiçbir zaman değişken **değeri** içermez (birim testleriyle doğrulanır). |
| Değişiklik planları | Gizli değer taşımaz; değerler arka uçta kayıtlı kaynaktan (profil, anlık görüntü, içe aktarma, mevcut kapsam) alınır. |
| Dışa aktarma | Hassas değerler varsayılan olarak hariç tutulur; dahil etmek açık onay ister ve dosya düz metindir. |
| İçe aktarma | İçerik çalıştırılmaz veya genişletilmez; `$(...)`, `${...}` metin olarak kalır ve uyarı verilir. |
| Telemetri | Yoktur. Uygulama, güncelleme denetimi dışında ağa bağlanmaz. |

## Hassas değişken algılama

Bir değişken şu durumlarda hassas sayılır:

- Ad parçalarından biri `SECRET`, `PASSWORD`, `TOKEN`, `APIKEY`, `CREDENTIAL`, `AUTH`, `DSN` … ya da
  `API_KEY`, `ACCESS_KEY`, `PRIVATE_KEY`, `CLIENT_SECRET`, `DATABASE_URL` gibi bir çift ise,
- Değer bilinen bir anahtar biçimine benziyorsa (GitHub, GitLab, OpenAI, Stripe, Slack, AWS, Google, npm,
  PyPI, Hugging Face önekleri, JWT, PEM özel anahtar başlığı),
- Değer parola içeren bir bağlantı adresi (`şema://kullanıcı:parola@sunucu`) veya bağlantı dizesi
  (`Password=…`, `AccountKey=…`) ise,
- Kullanıcı adı **elle hassas olarak işaretlediyse**.

Her değişken gizli sayılmaz (`PATH`, `PWD`, `SESSIONNAME` gibi bilinen adlar istisnadır). Algılama yanlış
sonuç verirse değişken ayrıntısından "Hassas / Hassas değil / Otomatik" seçilebilir; geçersiz kılmalar
Ayarlar → Güvenlik bölümünde listelenir. Kurallar `src-tauri/src/security/classify.rs` dosyasındadır.

## Şifreleme (DPAPI)

- Kullanılan API: `CryptProtectData` / `CryptUnprotectData`, **geçerli kullanıcı** kapsamı,
  `CRYPTPROTECT_UI_FORBIDDEN`, uygulamaya özgü (gizli olmayan) bir entropi değeri.
- Koruduğu durum: veritabanı dosyasının başka bir kullanıcıya veya bilgisayara kopyalanması, yedeklerin
  yanlışlıkla paylaşılması.
- **Korumadığı durum:** sizin Windows hesabınızla çalışan kötü amaçlı yazılım DPAPI verisini çözebilir.
  Bu, Windows'un tasarımıdır; uygulama düzeyinde şifreleme bu saldırgana karşı koruma sağlamaz.
- Sistem kapsamı değişiklikleri için yükseltilmiş yardımcıya aktarılan kısa ömürlü plan dosyası
  **makine** kapsamıyla şifrelenir (yükseltilmiş süreç farklı bir oturumda çalışabilir); dosya işlem
  bitince silinir.

## Kalıcı ortam değişkenleri hakkında önemli not

Windows, kalıcı kullanıcı ve sistem ortam değişkenlerini kayıt defterinde **düz metin** olarak saklar ve
başlattığınız her program bu değerleri görür. Ivily Envora bunu değiştiremez. Gizli bir değeri kalıcı ortama
yazmak yerine projeye özgü değişken olarak (DPAPI ile şifreli) saklamak ve yalnızca proje başlatılırken
işlem kapsamında vermek daha güvenlidir. Analiz bu durumları ayrıca bildirir.

## Yetki ve yükseltme

- Uygulama standart kullanıcı yetkisiyle çalışır; **tüm uygulama hiçbir zaman yükseltilmez**.
- Sistem kapsamındaki onaylanmış bir değişiklik için aynı yürütülebilir dosya `runas` ile (UAC istemi)
  yalnızca o işlem için başlatılır. Yardımcı kip:
  - arayüz açmaz, komut veya betik çalıştırmaz,
  - yalnızca uygulamanın `%LOCALAPPDATA%\IvilyEnvora\elevate` klasöründeki plan dosyalarını kabul eder,
  - planı çözer, şemayı ve tüm ad/değerleri yeniden doğrular,
  - yalnızca **sistem kapsamındaki ortam değişkeni ayarlama/silme** işlemlerini uygular,
  - bir işlem başarısız olursa önceki işlemleri geri alır ve değer içermeyen bir sonuç yazar.
- Sınırlama: Yönetici olmayan bir hesapta "başka bir yönetici hesabıyla" (over-the-shoulder) yükseltme
  yapılırsa yardımcı farklı bir kullanıcının veri klasörüne bakar ve işlem başarısız olur.

## Süreç başlatma

- Proje açıldığında hiçbir şey çalıştırılmaz. Proje dizinindeki işaret dosyaları (`package.json` vb.)
  yalnızca **var olup olmadıklarına** bakılarak algılanır; okunmaz.
- "Terminal aç" yalnızca seçilen kabuğu (pwsh / powershell / cmd) tam yoluyla başlatır.
- Özel komut yalnızca `.exe` / `.com` olabilir (betik dosyaları kabuk yorumuna yol açacağı için
  reddedilir); argümanlar kabuk dizesi olarak değil, ayrı ayrı verilir; tam yol ve argümanlar onaydan önce
  gösterilir.
- İçe aktarılan projelerdeki özel komutlar varsayılan olarak alınmaz.
- Araç keşfi: sürümler önce dosya bilgisinden okunur; gerekirse yalnızca `.exe` araçlara sabit
  `--version` argümanıyla, zaman aşımıyla ve pencere açmadan sorulur. Microsoft Store takma adları
  **hiçbir zaman** çalıştırılmaz.

## Ön uç (WebView2)

- Katı içerik güvenlik politikası (CSP): yalnızca uygulamanın kendi betikleri, `eval` yok, uzak içerik yok.
- `freezePrototype` açık; genel Tauri nesnesi (`withGlobalTauri`) kapalı.
- Tauri yetenekleri yalnızca `core:default` ve dosya aç/kaydet iletişim kutularıdır; dosya sistemi, kabuk
  ve HTTP eklentileri eklenmemiştir.
- Dosya yazan komutlar yalnızca beklenen uzantılara (`.json`, `.csv`, `.env`, `.txt`, `.db`) izin verir.

## Bilinen sınırlamalar

- DPAPI, aynı kullanıcı olarak çalışan kötü amaçlı yazılıma karşı koruma sağlamaz.
- Pano içeriği, kopyalandığı anda panoyu izleyen programlar tarafından okunabilir.
- Kalıcı ortam değişkenleri Windows tarafından düz metin olarak saklanır.
- Bellekteki gizli değerler (gösterildiğinde veya bir süreç başlatılırken) işlem belleğinden sıfırlanarak
  silinmez; Rust ve JavaScript çalışma zamanları bunu garanti etmez.
- Hassas değişken algılaması kurala dayalıdır ve yanlış pozitif/negatif üretebilir.
- Yükleyici ve uygulama şu an kod imzalı değildir; Windows SmartScreen uyarı gösterebilir. Güncelleme
  paketleri ise minisign ile imzalanır ve uygulama imzayı doğrulamadan güncelleme kurmaz.
