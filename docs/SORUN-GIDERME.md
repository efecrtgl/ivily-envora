# Sorun giderme

### Değişkeni değiştirdim ama terminalde eski değer görünüyor

Windows, ortamı bir program **başlatılırken** kopyalar. Açık terminali, IDE'yi veya programı kapatıp yeniden
açın. VS Code gibi uygulamalarda tüm pencereleri kapatmak gerekir; bazı uygulamalar (ör. Explorer'dan
başlatılmayanlar) oturumu kapatıp açmayı gerektirebilir. Envora değişiklikten sonra `WM_SETTINGCHANGE`
bildirimi gönderir; bu bildirimi dinleyen Explorer, ondan **yeni** başlatılan programlara güncel değeri verir.

### "Envora'nın işlem ortamı eski" analizi

Envora başlatıldıktan sonra kalıcı ortam değişmiş demektir (Envora'nın kendisi veya başka bir araç
değiştirmiş olabilir). Envora'yı yeniden başlatırsanız İşlem sekmesi de güncellenir. Kalıcı değerler ve
proje başlatma her zaman güncel kayıt defteri değerlerini kullanır.

### Sistem değişkeni değiştirirken UAC istemi çıkmıyor / "Yönetici izni verilmedi"

UAC isteminde "Evet" seçilmediyse hiçbir değişiklik yapılmaz. Standart kullanıcı hesabıyla başka bir
yöneticinin bilgileri girilerek yapılan yükseltme şu an desteklenmez (yükseltilmiş süreç farklı bir
kullanıcının veri klasörüne bakar). Bu durumda değişikliği yönetici hesabıyla yapın.

### Bir değişken hassas sayılıyor ama değil (veya tersi)

Değişkenin ayrıntısından **Hassasiyet: Hassas değil / Hassas / Otomatik** seçin. Geçersiz kılmalar
Ayarlar → Güvenlik'te listelenir ve oradan kaldırılabilir.

### "Veritabanı açılamadı" uyarısı

Envora geçici bellekle çalışıyordur; değişiklikler kaydedilmez. Olası nedenler:

- Veritabanı daha yeni bir Envora sürümüyle oluşturulmuş (eski sürüme dönüldü). Yeni sürümü kurun.
- Dosya bozuk veya başka bir program kilitlemiş. Envora'yı kapatıp `%LOCALAPPDATA%\IvilyEnvora` klasörünü
  yedekleyin; Ayarlar → Veri → **Yedekten geri yükle** ile önceki bir yedeği kullanın.

### Projede "Terminal aç" çalışmıyor

- Proje dizini taşınmış veya silinmiş olabilir (proje sayfasında "Dizin bulunamadı" görünür). Projeyi
  düzenleyip dizini güncelleyin.
- Seçilen kabuk kurulu değilse listede görünmez; PowerShell 7 (`pwsh`) ayrı kurulum gerektirir.

### Özel komut "Yalnızca .exe ve .com programlar başlatılabilir" hatası

Güvenlik nedeniyle `.bat`, `.cmd`, `.ps1` dosyaları doğrudan başlatılmaz. Bir betik çalıştırmak için
"Terminal aç" ile projenin terminalini açıp betiği orada çalıştırın.

### `python` komutu Microsoft Store'u açıyor

PATH'te Store'un `python.exe` takma adı gerçek kurulumdan önce geliyor. PATH → Komut çakışmaları bunu
gösterir. Ayarlar → Uygulamalar → Gelişmiş uygulama ayarları → **Uygulama yürütme takma adları** bölümünden
"python.exe" ve "python3.exe" takma adlarını kapatın.

### Güncelleme denetimi başarısız

İnternet bağlantısını ve GitHub'a erişimi kontrol edin. "Ön sürüm" kanalı GitHub API'sini kullanır; çok
sık denetim yapılırsa GitHub geçici olarak istek sınırı uygulayabilir. Yükleyiciyi Sürümler sayfasından elle
de indirebilirsiniz.

### SmartScreen "Windows bilgisayarınızı korudu" diyor

Yükleyici şu an kod imzalı değil. Bkz. [KURULUM.md](KURULUM.md#smartscreen-uyarısı).

### Hata bildirmek

Ayarlar → Uygulama bilgisi → **Tanılama raporu** içeriğini (değişken adı/değeri içermez) GitHub'da bir
hata bildirimi açarak ekleyin. Gizli değerleri asla paylaşmayın.
