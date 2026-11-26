# Katkıda Bulunma

Ivily Envora'ya katkıda bulunmak istediğiniz için teşekkürler. *(English summary below.)*

## Başlamadan önce

- Geliştirme ortamı kurulumu için [docs/GELISTIRME.md](docs/GELISTIRME.md) belgesine bakın.
- Büyük değişiklikler için önce bir konu (issue) açarak yaklaşımı tartışın.

## Kurallar

- **TypeScript kullanılmaz.** Ön uç düz JavaScript (JSX) ile yazılır.
- Yeni bağımlılık eklemeden önce gerçekten gerekli olup olmadığını değerlendirin ve PR açıklamasında gerekçelendirin.
- Arayüz metinleri yalnızca `src/i18n/tr.js` ve `src/i18n/en.js` içine eklenir; iki dosya aynı anahtarlara sahip olmalıdır.
- Ortam değişkeni **değerleri** hiçbir koşulda günlüklere, hata mesajlarına veya etkinlik kaydına yazılmaz.
- Otomatik testler gerçek kullanıcı veya sistem ortamını **değiştirmemelidir**; bellek içi sağlayıcıyı (`MemoryProvider`) kullanın.
- Commit mesajları İngilizce [Conventional Commits](https://www.conventionalcommits.org/) biçimindedir:
  `feat: ...`, `fix: ...`, `docs: ...`, `test: ...`, `refactor: ...`, `perf: ...`, `ci: ...`, `chore: ...`.

## PR göndermeden önce

```powershell
npm test               # arayüz testleri
npm run build          # arayüz derlemesi
cd src-tauri
cargo test             # Rust testleri
cargo clippy -- -D warnings
```

Gerçek API anahtarı, token, parola veya kişisel veri içeren dosyaları **asla** commit etmeyin.

---

## English summary

Plain JavaScript only (no TypeScript), keep dependencies minimal, put every UI string in both
`src/i18n/tr.js` and `src/i18n/en.js`, never log environment variable values, never let automated tests modify
the real user/system environment, and use English Conventional Commit messages. Run `npm test`, `npm run build`,
`cargo test` and `cargo clippy -- -D warnings` before opening a pull request.
