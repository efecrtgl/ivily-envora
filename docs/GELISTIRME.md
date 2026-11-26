# Geliştirme ortamı

## Gereksinimler

- Windows 10/11 x64
- [Rust](https://rustup.rs/) (kararlı, 1.82 veya üstü) — MSVC araç zinciri ve "Desktop development with C++"
  bileşenli Visual Studio Build Tools
- [Node.js](https://nodejs.org/) 22 veya üstü
- Microsoft Edge WebView2 (Windows 11'de hazır)

## Başlangıç

```powershell
git clone https://github.com/efecrtgl/ivily-envora.git
cd ivily-envora
npm ci
npm run tauri dev        # Vite geliştirme sunucusu + Tauri penceresi
```

| Komut | Açıklama |
|-------|----------|
| `npm run dev` | Yalnızca arayüz (tarayıcıda açılır; arka uç çağrıları "yalnızca masaüstünde" hatası verir) |
| `npm run tauri dev` | Masaüstü uygulaması, sıcak yeniden yükleme |
| `npm run build` | Arayüz üretim derlemesi (`dist/`) |
| `npx tauri build --no-bundle` | Release yürütülebilir dosyası |
| `npx tauri build` | NSIS yükleyicisi (güncelleme imzası için anahtar gerekir; bkz. [SURUM.md](SURUM.md)) |
| `npm test` | Arayüz testleri |
| `cd src-tauri; cargo test` | Rust testleri |
| `cargo clippy --all-targets -- -D warnings` | Lint |
| `cargo fmt` | Biçimlendirme |
| `node scripts/check-version.mjs` | Sürüm tutarlılığı |

Geliştirme kipi de **gerçek** ortam değişkenlerinizi okur ve onayladığınızda değiştirir. Denemeler için
benzersiz adlı, geçici değişkenler kullanın.

## Proje yapısı

```
src/                      React arayüzü (düz JavaScript, TypeScript yok)
  app/                    kabuk, gezinme, ayarlar bağlamı, tema
  components/ui/          tasarım sistemi bileşenleri
  components/env/         değişken bileşenleri (SecretValue, ChangeFlow, VariableEditor …)
  components/envset/      profil/proje değişken ve PATH düzenleyicileri
  pages/                  ekranlar (her biri ayrı ve tembel yüklenen parça)
  i18n/                   tr.js (+tr/*), en.js (+en/*)
  lib/                    api.js (tek IPC kapısı), saf yardımcılar
  styles/                 tasarım belirteçleri ve bileşen stilleri
src-tauri/src/            Rust arka ucu (bkz. MIMARI.md)
src-tauri/tests/          uçtan uca entegrasyon testleri
docs/                     belgeler
scripts/                  yardımcı betikler
```

## Kurallar

- **TypeScript kullanılmaz.** Yeni bağımlılık eklemeden önce gerçekten gerekli olup olmadığını değerlendirin.
- Arayüz metinleri yalnızca `src/i18n/tr*` ve `src/i18n/en*` içindedir. İki dil aynı anahtarlara sahip
  olmalıdır; `src/i18n/usage.test.js`, kaynakta kullanılan her sabit `t('…')` anahtarının tanımlı olduğunu
  denetler.
- Ortam değişkeni **değerleri** etkinlik kaydına, hata mesajlarına veya günlüklere yazılmaz.
  `activity::record`, `value`/`newValue` gibi alanları içeren ayrıntıları reddeder.
- Arayüze gönderilen listelerde hassas değerler `None` olmalıdır; değer gerektiren değişiklik planları
  `copy`, `fromProfile`, `fromSnapshot`, `fromImport` işlemleriyle değeri arka uçta çözer.
- Testler gerçek ortamı değiştirmez; yazma senaryoları `MemoryProvider` ile yazılır.
- Commit mesajları İngilizce Conventional Commits biçimindedir.

## Sık yapılan değişiklikler

### Yeni bir Tauri komutu

1. İş mantığını ilgili modülde saf bir işlev olarak yazın ve test edin.
2. `src-tauri/src/commands/<alan>.rs` içine ince bir `#[tauri::command]` ekleyin (doğrulama + çağrı + etkinlik kaydı).
3. `src-tauri/src/lib.rs` içindeki `generate_handler!` listesine ekleyin.
4. `src/lib/api.js` içine bir çağrı ekleyin.

### Yeni bir analiz kuralı

1. `src-tauri/src/analysis/rules.rs` içine `fn kuralim(ctx: &Context) -> Vec<Finding>` yazın
   (PATH raporları için `ctx.path_report(scope)` kullanın; yeniden hesaplamayın).
2. `RULES` tablosuna kimlik, kategori ve varsayılan önemle ekleyin.
3. `src/i18n/{tr,en}/analysis.js` içinde `rule.<kimlik_alt_çizgili>.title/body` metinlerini ekleyin
   (`src/pages/analysis/analysis.test.js` her kuralın metnini denetler).

### Yeni bir veritabanı tablosu

`src-tauri/src/db/migrations.rs` dizisinin **sonuna** yeni bir geçiş ekleyin. Var olan geçişleri asla
değiştirmeyin. `docs/VERITABANI.md` tablosunu güncelleyin.

### Yeni bir hassas değer kalıbı

`src-tauri/src/security/classify.rs` içindeki `NAME_TOKENS`, `NAME_PAIRS` veya `VALUE_PREFIXES` tablosuna
ekleyin; hem pozitif hem negatif test yazın. Test verilerinde gerçek token biçimine benzeyen düz metinler
yerine `concat!("gh", "p_", …)` gibi parçalı dizeler kullanın (gizli tarayıcılar yanlış alarm vermesin).
