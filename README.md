# Ivily Envora

**Windows için açık kaynaklı ortam değişkeni, PATH ve geliştirme ortamı yöneticisi.**

Ivily Envora; ortam değişkenlerini, PATH girdilerini, ortam profillerini ve proje ortamlarını anlaşılır,
güvenli ve geri alınabilir biçimde yönetmenizi sağlar. Ivily ekosisteminin bir parçasıdır ancak tek başına,
hiçbir başka uygulamaya ihtiyaç duymadan çalışır.

*English summary below.*

## Özellikler

- **Değişkenler** — İşlem, kullanıcı ve sistem kapsamlarını ayrı ayrı görün; arayın, süzün, oluşturun,
  düzenleyin, çoğaltın, yeniden adlandırın, silin. `REG_SZ` / `REG_EXPAND_SZ` türü korunur.
- **Güvenli değişiklik akışı** — Her kalıcı değişiklik önce önizlenir, onay ister, öncesinde otomatik
  yedek alınır ve hata olursa geri alınır. Sistem kapsamı için uygulamanın tamamı değil, yalnızca o işlem
  yükseltilir (UAC).
- **Gizli değer koruması** — Token, parola ve anahtar gibi hassas değerler algılanır, maskelenir ve yalnızca
  istendiğinde gösterilir; arayüze, etkinlik kaydına, hata mesajlarına ve dışa aktarmalara sızmaz.
  Uygulama içinde saklananlar Windows DPAPI ile şifrelenir.
- **PATH yöneticisi** — Bulunamayan, yinelenen, göreli ve çözülemeyen girdileri gösterir; sıralar, düzenler
  ve hangi programın hangi kopyasının çalıştığını (komut çakışmaları) açıklar.
- **Profiller** — Yeniden kullanılabilir değişken ve PATH kümeleri; sürüm geçmişi, doğrulama, kalıcı
  uygulama ve tek tıkla geri alma.
- **Projeler** — Proje dizinlerini profillerle ve projeye özgü değişkenlerle eşleştirin; bir terminali veya
  tanımladığınız programı proje ortamıyla, yalnızca açık onayla başlatın.
- **Karşılaştırma** — Kapsamları, profilleri, projeleri, anlık görüntüleri ve içe aktarılan dosyaları
  yan yana karşılaştırın.
- **Anlık görüntüler ve yedekleme** — Ortamın belirli anlardaki kaydı; seçimli ve önizlemeli geri yükleme;
  veritabanı yedeği.
- **İçe / dışa aktarma** — JSON, CSV, `.env` ve PATH listesi. İçe aktarılan içerik asla çalıştırılmaz.
- **Analiz** — 23 kurallık, yapılandırılabilir analiz motoru; sorunları açıklar ama otomatik düzeltmez.
- **Araç keşfi** — Node.js, Python, Rust, Java, Go, .NET, Git, Docker ve diğerlerinin sürüm ve konumları.
- **Şablonlar** — Yaygın geliştirme ortamları için gizli değer içermeyen öneriler.
- **Pano ve etkinlik merkezi** — Anlamlı özetler ve değer içermeyen yerel işlem kaydı.
- **Türkçe ve İngilizce**, açık/koyu tema, imzalı güncellemeler.

## Kurulum

[Sürümler](https://github.com/efecrtgl/ivily-envora/releases) sayfasından
`Ivily Envora_<sürüm>_x64-setup.exe` dosyasını indirip çalıştırın. Kurulum yönetici yetkisi gerektirmez.
Ayrıntılar: [docs/KURULUM.md](docs/KURULUM.md).

Gereksinimler: Windows 10 (1809+) veya Windows 11, x64; Microsoft Edge WebView2 (Windows 11'de hazır gelir,
gerekirse kurulum programı indirir).

## Belgeler

| Kullanıcılar için | Geliştiriciler için |
|-------------------|---------------------|
| [Kurulum ve güncelleme](docs/KURULUM.md) | [Geliştirme ortamı](docs/GELISTIRME.md) |
| [Kullanım kılavuzu](docs/KULLANIM.md) | [Mimari](docs/MIMARI.md) |
| [Ortam değişkenleri rehberi](docs/rehber/ORTAM-DEGISKENLERI.md) | [Veritabanı](docs/VERITABANI.md) |
| [PATH rehberi](docs/rehber/PATH.md) | [Testler](docs/TEST.md) |
| [Profiller](docs/rehber/PROFILLER.md) | [Performans ölçümleri](docs/PERFORMANS.md) |
| [Proje ortamları](docs/rehber/PROJELER.md) | [Ivily Ecosystem Protocol](docs/IEP.md) |
| [Yedekleme ve geri yükleme](docs/rehber/YEDEKLEME.md) | [Sürüm yayımlama](docs/SURUM.md) |
| [İçe ve dışa aktarma](docs/rehber/ICE-DISA-AKTARMA.md) | [Yol haritası](ROADMAP.md) |
| [Güvenlik ve sınırlamalar](docs/GUVENLIK.md) | [Katkıda bulunma](CONTRIBUTING.md) |
| [Sorun giderme](docs/SORUN-GIDERME.md) | [Değişiklik günlüğü](CHANGELOG.md) |

## Güvenlik

Ivily Envora telemetri göndermez ve güncelleme denetimi dışında ağa bağlanmaz. Tehdit modeli ve bilinen
sınırlamalar [docs/GUVENLIK.md](docs/GUVENLIK.md) belgesindedir. Güvenlik açıklarını
[SECURITY.md](SECURITY.md) belgesindeki yolla özel olarak bildirin.

## Geliştirme

```powershell
npm ci
npm run tauri dev      # geliştirme kipi
npm test               # arayüz testleri
cd src-tauri; cargo test
```

Teknolojiler: Rust, Tauri 2, React (düz JavaScript), SQLite. Ayrıntılar: [docs/GELISTIRME.md](docs/GELISTIRME.md).

## Lisans

[MIT](LICENSE) © 2026 efecrtgl

---

## English

Ivily Envora is an open-source Windows desktop app for managing environment variables, PATH entries,
environment profiles and project environments. It is part of the Ivily ecosystem but fully usable on its own.

- Inspect and edit process, user and system variables; every persistent change is previewed, confirmed,
  backed up first and rolled back on failure. Only the system-scope change itself is elevated (UAC).
- Sensitive values (tokens, passwords, keys) are detected, masked, revealed only on request, never written to
  logs, errors or exports by default, and encrypted with Windows DPAPI when stored by the app.
- PATH manager with missing/duplicate/relative entry detection and executable conflict analysis.
- Profiles with version history and reversible activation; project environments with consented launching.
- Comparison, snapshots, JSON/CSV/`.env` import & export (imported content is never executed),
  a configurable analysis engine, developer tool discovery, templates, dashboard and a local activity log.
- Turkish (primary) and English UI, light/dark themes, signed in-app updates from GitHub Releases.

Download the installer from [Releases](https://github.com/efecrtgl/ivily-envora/releases). User documentation
is in Turkish; an English guide is available in [docs/en/GUIDE.md](docs/en/GUIDE.md).
