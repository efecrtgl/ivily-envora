# Kurulum, güncelleme ve kaldırma

## Gereksinimler

- Windows 10 (1809 veya üstü) ya da Windows 11, 64 bit (x64)
- Microsoft Edge WebView2 Çalışma Zamanı — Windows 11'de hazır gelir. Yoksa kurulum programı Microsoft'un
  önyükleyicisini indirip kurar (internet bağlantısı gerekir).

## Kurulum

1. [GitHub Sürümleri](https://github.com/efecrtgl/ivily-envora/releases) sayfasından
   `Ivily Envora_<sürüm>_x64-setup.exe` dosyasını indirin.
2. İsteğe bağlı: aynı sürümdeki `SHA256SUMS.txt` ile dosyayı doğrulayın:
   ```powershell
   (Get-FileHash ".\Ivily Envora_0.1.0_x64-setup.exe" -Algorithm SHA256).Hash.ToLower()
   ```
3. Dosyayı çalıştırın. Kurulum **yönetici yetkisi gerektirmez**; uygulama yalnızca sizin hesabınıza kurulur.

| Öğe | Konum |
|-----|-------|
| Program | `%LOCALAPPDATA%\Ivily Envora\` |
| Başlat menüsü kısayolu | `Başlat → Ivily Envora` |
| Uygulama verisi (veritabanı) | `%LOCALAPPDATA%\IvilyEnvora\envora.db` |

### SmartScreen uyarısı

Yükleyici ve uygulama şu an bir kod imzalama sertifikasıyla imzalı **değildir**. Windows SmartScreen
"Windows bilgisayarınızı korudu" uyarısı gösterebilir. Dosyayı resmi sürüm sayfasından indirdiyseniz ve
SHA-256 özeti eşleşiyorsa "Ek bilgi → Yine de çalıştır" ile devam edebilirsiniz.

Uygulama içi **güncellemeler** ise ayrıca minisign ile imzalanır ve imza doğrulanmadan kurulmaz
(bkz. aşağıda).

## Sessiz kurulum

```powershell
& ".\Ivily Envora_0.1.0_x64-setup.exe" /S
```

## Güncelleme

- **Ayarlar → Güncellemeler → Güncellemeleri denetle** ile yeni sürüm olup olmadığına bakılır.
- "Açılışta denetle" açıksa uygulama günde en fazla bir kez denetler ve yalnızca bildirim gösterir.
- "İndir ve kur" yalnızca onayınızdan sonra çalışır: paket GitHub'daki resmi sürümden indirilir,
  **imzası uygulamaya gömülü genel anahtarla doğrulanır**, imza geçersizse kurulmaz.
- Kanal: **Kararlı** (varsayılan) veya **Ön sürüm** (test sürümlerini de içerir).
- Güncelleme, ayarlarınızı ve verilerinizi korur; veritabanı şeması gerekiyorsa otomatik ve işlem
  güvenli biçimde yükseltilir.

Yeni bir yükleyiciyi elle çalıştırarak da güncelleyebilirsiniz; veriler korunur.

## Kaldırma

**Ayarlar → Uygulamalar → Yüklü uygulamalar → Ivily Envora → Kaldır** veya
`%LOCALAPPDATA%\Ivily Envora\uninstall.exe`.

- Program dosyaları, kısayol ve kaldırma kaydı silinir.
- `%LOCALAPPDATA%\IvilyEnvora` klasöründeki **verileriniz silinmez** (profiller, projeler, anlık görüntüler,
  ayarlar). Tamamen kaldırmak isterseniz bu klasörü elle silin.
- Envora'nın daha önce kalıcı ortama yazdığı değişkenler **geri alınmaz**; kaldırmadan önce gerekiyorsa
  profilleri "Geri al" ile devre dışı bırakın veya bir anlık görüntüden geri yükleyin.

## Doğrulama durumu

0.1.0 yükleyicisi geliştirme bilgisayarında şu şekilde denendi: sessiz kurulum (1,5 sn, yönetici gerekmedi),
kurulu uygulamanın başlatılması, sessiz kaldırma; program dosyalarının, kısayolun ve kaldırma kaydının
silindiği, kullanıcı verisinin korunduğu doğrulandı. Güncelleme paketinin imzası, uygulamadaki genel
anahtarla bağımsız olarak doğrulandı.
