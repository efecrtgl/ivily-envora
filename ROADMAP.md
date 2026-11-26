# Ivily Envora — Yol Haritası

Bu belge Ivily Envora'nın geliştirme aşamalarını, kapsamlarını ve kabul ölçütlerini tanımlar.
Her aşama küçük, mantıksal olarak ayrılmış commit'ler halinde geliştirilir, test edilir ve
`main` dalına gönderilir. Durum sütunu gerçek ilerlemeyi yansıtır; tamamlanmamış bir aşama
tamamlanmış gibi gösterilmez.

Durum işaretleri: ✅ tamamlandı · 🚧 sürüyor · ⏳ planlandı

| # | Aşama | Durum |
|---|-------|-------|
| 1 | Depo denetimi ve başlatma | ✅ |
| 2 | Depo belgeleri ve altyapı | ✅ |
| 3 | Uygulama kabuğu ve tasarım sistemi | ✅ |
| 4 | Veritabanı ve yapılandırma temeli | ✅ |
| 5 | Ortam değişkeni inceleme | ✅ |
| 6 | Ortam değişkeni yönetimi | ✅ |
| 7 | Ortam profilleri | ✅ |
| 8 | Proje ortamları | ✅ |
| 9 | PATH yöneticisi | ✅ |
| 10 | Ortam karşılaştırma | ✅ |
| 11 | Anlık görüntü ve yedekleme | ✅ |
| 12 | İçe ve dışa aktarma | ✅ |
| 13 | Ortam analizi (zekâ) | ✅ |
| 14 | Geliştirici aracı keşfi ve şablonlar | ✅ |
| 15 | Pano ve etkinlik merkezi | ✅ |
| 16 | Güvenlik sıkılaştırması | ✅ |
| 17 | Ivily ekosistemi (IEP) hazırlığı | 🚧 |
| 18 | Performans iyileştirme | ⏳ |
| 19 | Test ve güvenilirlik | ⏳ |
| 20 | Yükleyici ve güncelleme sistemi | ⏳ |
| 21 | Belgelerin tamamlanması | ⏳ |
| 22 | Sürüm hazırlığı | ⏳ |

---

## Aşama 1 — Depo denetimi ve başlatma

- **Kapsam:** Yerel klasörün ve uzak deponun durumunu belirlemek, güvenli şekilde başlatmak.
- **Görevler:** Klasör içeriğini incele; git deposu olup olmadığını, uzak depoyu ve dalı doğrula;
  `.gitignore`, `.gitattributes`, `LICENSE`, ilk `README.md` dosyalarını oluştur.
- **Kabul ölçütü:** `main` dalı `origin`e gönderilmiş; yerel araç ayarları (`.claude/settings.local.json`) depo dışında.
- **Commit:** `chore: initialize repository infrastructure`

## Aşama 2 — Depo belgeleri ve altyapı

- **Kapsam:** Yol haritası, mimari tasarım, katkı/güvenlik/davranış kuralları, sorun ve PR şablonları.
- **Dosyalar:** `ROADMAP.md`, `docs/MIMARI.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`,
  `CHANGELOG.md`, `.github/ISSUE_TEMPLATE/*`, `.github/pull_request_template.md`.
- **Kabul ölçütü:** Şablonlar kullanıcıdan hiçbir ortam değişkeni değeri istemez; değerlerin maskelenmesini açıkça ister.
- **Commit:** `docs: add roadmap, architecture and community documentation`

## Aşama 3 — Uygulama kabuğu ve tasarım sistemi

- **Kapsam:** Tauri 2 + React (düz JavaScript) + Vite iskeleti; tek renkli cam tasarım sistemi;
  açık/koyu tema; merkezi i18n (TR birincil, EN ikincil); gezinme; Rust tarafında modül iskeleti.
- **Dosyalar:** `package.json`, `vite.config.js`, `index.html`, `src/main.jsx`, `src/app/*`,
  `src/styles/*`, `src/i18n/*`, `src/components/ui/*`, `src-tauri/*`.
- **Uygulama:** CSS değişkenleriyle tasarım belirteçleri; tema yalnızca `data-theme` özniteliğini değiştirir,
  veriye dokunmaz. Yönlendirme kütüphanesi yok; durum tabanlı basit gezinme.
- **Testler:** i18n anahtar eşitliği (TR ve EN aynı anahtarlara sahip), tema değişiminin durum sıfırlamaması.
- **Kabul ölçütü:** `npm run build` ve `cargo build` başarılı; uygulama açılıyor.
- **Commit'ler:** `feat: scaffold tauri application shell`, `feat: add design system and theming`, `feat: add centralized i18n`

## Aşama 4 — Veritabanı ve yapılandırma temeli

- **Kapsam:** SQLite (rusqlite, gömülü), sürümlü geçişler (`PRAGMA user_version`), ayarlar deposu ve doğrulama,
  hata türleri, uygulama veri dizini, DPAPI tabanlı gizli değer şifreleme katmanı.
- **Dosyalar:** `src-tauri/src/db/*`, `src-tauri/src/settings/*`, `src-tauri/src/security/vault.rs`.
- **Testler:** Geçişlerin boş ve mevcut veritabanında idempotent çalışması, ayar doğrulaması, DPAPI gidiş-dönüşü.
- **Commit'ler:** `feat: add sqlite database with versioned migrations`, `feat: add validated settings store`, `feat: add DPAPI secret vault`

## Aşama 5 — Ortam değişkeni inceleme

- **Kapsam:** İşlem, kullanıcı ve sistem kapsamlarını okumak. Kayıt defteri (`HKCU\Environment`,
  `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`) ve işlem ortamı.
  `REG_SZ` / `REG_EXPAND_SZ` türlerinin korunması. Hassas değişken sınıflandırması.
- **Uygulama:** `EnvProvider` arayüzü; Windows sağlayıcısı ve testler için bellek içi sağlayıcı.
- **Testler:** Sınıflandırıcı kuralları, ad doğrulama, bellek sağlayıcı üzerinden listeleme.
- **Commit:** `feat: implement environment variable inspection`

## Aşama 6 — Ortam değişkeni yönetimi

- **Kapsam:** Oluşturma, düzenleme, silme, çoğaltma (kapsamlar arası kopyalama); değişiklik planı → önizleme →
  onay → uygulama akışı; değişiklik öncesi otomatik yedek; `WM_SETTINGCHANGE` yayını.
  Sistem kapsamı için yalnızca o işlemi yükseltilmiş yardımcı kipte çalıştırma (tüm uygulama yükseltilmez).
- **Testler:** Plan doğrulama, kapsam karışmasının engellenmesi, bellek sağlayıcıda uygula/geri al.
- **Commit'ler:** `feat: add environment change plans with backups`, `feat: add elevated helper for machine scope`

## Aşama 7 — Ortam profilleri

- **Kapsam:** Profil CRUD, çoğaltma, arşivleme, etiketler, sürüm ve revizyon geçmişi, geri yükleme,
  doğrulama, kalıcı etkinleştirme (yedek + geri alma ile) ve devre dışı bırakma.
- **Commit:** `feat: add environment profile management`

## Aşama 8 — Proje ortamları

- **Kapsam:** Proje CRUD, dizin doğrulama, profil ilişkilendirme, projeye özgü değişkenler ve PATH girdileri,
  birleşik ortam hesaplama, açık onayla terminal başlatma (proje betikleri asla otomatik çalıştırılmaz).
- **Commit:** `feat: add project environment management`

## Aşama 9 — PATH yöneticisi

- **Kapsam:** Girdi ayrıştırma, yinelenen/eksik/göreli/geçersiz yol tespiti, `%VAR%` başvuruları, uzunluk sınırları,
  sıralama ve gölgeleme uyarıları, ekleme/silme/düzenleme/sıralama (önizleme + onay + yedek).
- **Commit:** `feat: implement PATH manager`

## Aşama 10 — Ortam karşılaştırma

- **Kapsam:** Kaynak seçimi (kapsam, profil, proje, anlık görüntü, içe aktarılan dosya), eklenen/silinen/değişen/aynı,
  PATH farkı, maskeli hassas değerler.
- **Commit:** `feat: add environment comparison engine`

## Aşama 11 — Anlık görüntü ve yedekleme

- **Kapsam:** Yapılandırılabilir anlık görüntüler (kapsam, değişken seçimi, hassasları hariç tutma), DPAPI ile şifreli
  hassas değerler, karşılaştırma, doğrulamalı ve onaylı geri yükleme, veritabanı yedekleme/geri yükleme.
- **Commit:** `feat: add snapshot and backup system`

## Aşama 12 — İçe ve dışa aktarma

- **Kapsam:** JSON (uygulama biçimi, sürümlü), CSV, `.env` (yaygın varyasyonlar; komut genişletme yok),
  önizleme, çakışma çözümü, hassas değer uyarıları ve hariç tutma.
- **Commit:** `feat: add import and export`

## Aşama 13 — Ortam analizi

- **Kapsam:** Modüler kural motoru (kimlik, önem derecesi, yapılandırılabilir), PATH ve değişken kuralları,
  eyleme dönük açıklamalar; otomatik düzeltme yok.
- **Commit:** `feat: add environment analysis engine`

## Aşama 14 — Geliştirici aracı keşfi

- **Kapsam:** PATH üzerinde yürütülebilir dosya arama ve isteğe bağlı sürüm sorgusu (yalnızca kullanıcı başlattığında,
  zaman aşımıyla, sabit argümanlarla). İndirme/kurulum yok.
- **Commit:** `feat: add developer tool discovery`

## Aşama 15 — Pano ve etkinlik merkezi

- **Kapsam:** Anlamlı pano bölümleri, tembel yükleme; etkinlik kaydı (değer içermez), arama, filtre, saklama süresi.
- **Commit'ler:** `feat: add activity center`, `feat: build dashboard`

## Aşama 16 — Güvenlik sıkılaştırması

- **Kapsam:** Komut yüzeyi denetimi, CSP, yetenek (capability) daraltma, girdi doğrulama, hata mesajlarında değer sızıntısı testleri.
- **Commit:** `fix: harden command surface and redaction`

## Aşama 17 — IEP hazırlığı

- **Kapsam:** Ivily Ecosystem Protocol için sürümlü, yetenek tabanlı, yalnızca yerel özet modeli; varsayılan olarak kapalı;
  değer içermeyen özetler; belge.
- **Commit:** `feat: prepare Ivily Ecosystem Protocol integration`

## Aşama 18 — Performans

- **Kapsam:** Başlangıç süresi, bellek, gereksiz yeniden çizim ve sorguların ölçümü; ölçüm belgesi (uydurma metrik yok).
- **Commit:** `perf: ...`, `docs: add performance measurements`

## Aşama 19 — Test ve güvenilirlik

- **Kapsam:** Rust birim/entegrasyon testleri, arayüz testleri, CI.
- **Commit'ler:** `test: ...`, `ci: configure Windows build pipeline`

## Aşama 20 — Yükleyici ve güncelleme

- **Kapsam:** NSIS yükleyici (kullanıcı başına), minisign imzalı güncellemeler, kararlı/ön sürüm kanalı,
  kullanıcı onayı, sürüm iş akışı.
- **Commit'ler:** `feat: add update system`, `ci: add release workflow`

## Aşama 21 — Belgeler

- **Kapsam:** Kullanıcı kılavuzu, rehberler, geliştirici belgeleri, sorun giderme.

## Aşama 22 — Sürüm hazırlığı

- **Kapsam:** Sürüm numarası, CHANGELOG, yükleyicinin yerel doğrulaması, GitHub Releases.
