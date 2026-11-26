# Sürüm yayımlama

## Sürüm numarası

Anlamsal sürümleme kullanılır. Sürüm `package.json` ve `src-tauri/Cargo.toml` içinde **aynı** olmalıdır
(`tauri.conf.json` sürümü `package.json`'dan okur). `node scripts/check-version.mjs` bunu denetler.
Ön sürümler `-beta.1`, `-rc.1` gibi bir ek alır ve GitHub'da "pre-release" olarak işaretlenir.

## Güncelleme imzalama anahtarı (tek seferlik)

Güncelleme paketleri bir minisign anahtarıyla imzalanır. **Genel anahtar** `src-tauri/tauri.conf.json`
(`plugins.updater.pubkey`) içindedir. **Özel anahtar asla depoya eklenmez**; geliştiricinin bilgisayarında
`%USERPROFILE%\.tauri\ivily-envora.key` (parolası `ivily-envora.key.password.txt`) olarak durur.

GitHub Actions ile yayımlamak için depoda **Settings → Secrets and variables → Actions** altında:

- `TAURI_SIGNING_PRIVATE_KEY` — özel anahtar dosyasının içeriği
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — anahtarın parolası

Anahtarı ve parolayı güvenli bir yerde (ör. parola yöneticisi) yedekleyin. **Anahtar kaybolursa** mevcut
kurulumlar yeni sürümleri uygulama içinden kuramaz; yeni bir anahtarla oluşturulan ve genel anahtarı
güncelleyen bir sürümün elle kurulması gerekir.

## Yayımlama adımları

1. `CHANGELOG.md` içinde `## [X.Y.Z] - YYYY-AA-GG` başlığıyla sürüm notlarını yazın.
2. `package.json` ve `src-tauri/Cargo.toml` sürümünü güncelleyin; `cd src-tauri; cargo build` ile
   `Cargo.lock`'u güncelleyin; `node scripts/check-version.mjs` çalıştırın.
3. Commit: `chore: release vX.Y.Z` ve `main`'e gönderin; CI'nın geçtiğini doğrulayın.
4. Etiket: `git tag vX.Y.Z; git push origin vX.Y.Z`.
5. **Sürüm** iş akışı (`.github/workflows/release.yml`) şunları yapar:
   - etiket/sürüm eşleşmesi, testler,
   - NSIS yükleyicisi ve imzalı güncelleme paketi (`.exe.sig`),
   - `latest.json` (uygulama içi güncelleyicinin okuduğu dosya),
   - yapıt doğrulaması (yükleyici boyutu, imza varlığı, ürün sürümü),
   - `SHA256SUMS.txt`,
   - CHANGELOG'dan alınan notlarla **taslak** GitHub sürümü.
6. Taslağı gözden geçirip yayımlayın. Uygulama içi güncelleyici yalnızca **yayımlanmış** sürümleri görür
   (kararlı kanal: en son kararlı sürüm; ön sürüm kanalı: ön sürümler dahil en yenisi).

## Yerel derleme ve elle yayımlama

GitHub Actions gizlileri tanımlı değilse yerel olarak da derlenebilir:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content "$env:USERPROFILE\.tauri\ivily-envora.key" -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = Get-Content "$env:USERPROFILE\.tauri\ivily-envora.key.password.txt" -Raw
npx tauri build
```

Çıktı: `src-tauri/target/release/bundle/nsis/` altında yükleyici ve `.sig` dosyası. `latest.json`
elle oluşturulursa biçimi:

```json
{
  "version": "0.1.0",
  "notes": "…",
  "pub_date": "2026-11-27T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "<.sig dosyasının içeriği>",
      "url": "https://github.com/efecrtgl/ivily-envora/releases/download/v0.1.0/Ivily.Envora_0.1.0_x64-setup.exe"
    }
  }
}
```

GitHub, yüklenen dosya adlarındaki boşlukları noktaya çevirir (`Ivily.Envora_…`); `url` ve
`SHA256SUMS.txt` bu adı kullanmalıdır.

## Yayımlamadan önce elle denetim listesi

- [ ] Temiz bir kullanıcı hesabında sessiz kurulum (`/S`), uygulamanın açılması
- [ ] Önceki sürümün üzerine kurulum; ayarlar ve profiller korunuyor mu
- [ ] Kaldırma; veri klasörü korunuyor mu
- [ ] Uygulama içi güncelleme (önceki sürüm kuruluyken)
- [ ] `SHA256SUMS.txt` ile yükleyici özetinin eşleşmesi
