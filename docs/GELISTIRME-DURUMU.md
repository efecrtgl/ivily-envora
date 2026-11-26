# Geliştirme Durumu — Duraklatma Notu (2026-11-26)

Geliştirme, proje sahibinin isteğiyle **geçici olarak duraklatıldı**. Bu dal (`wip/phase-3-app-shell`)
tamamlanmamış Aşama 3 çalışmasını korur; `main` dalına birleştirilmemiştir.

## Tamamlanan (main dalında)

- Aşama 1 — Depo başlatma (`58bd085`)
- Aşama 2 — Yol haritası, mimari ve topluluk belgeleri (`c09d698`)

## Aşama 3 — Uygulama kabuğu (bu dalda, TAMAMLANMADI)

Mevcut:
- Tauri 2 + React (düz JS) + Vite iskeleti, `package.json`, `tauri.conf.json`, yetenek dosyası, simgeler.
- Rust iskeleti (`lib.rs`, `error.rs`, `commands/app.rs`) — `cargo build` başarılı.
- Tasarım belirteçleri ve bileşen CSS'i, simge seti, temel UI bileşenleri, kipli pencere, bildirimler.
- Merkezi i18n (TR/EN), tema yönetimi, ayar bağlamı, uygulama kabuğu ve gezinme.

Eksik (ön uç derlemesi şu an **başarısız olur**):
- `src/pages/Dashboard.jsx` ve `src/pages/Settings.jsx` henüz yazılmadı (`nav.js` bunları içe aktarıyor).
- `vite.config.js` içinde `__APP_VERSION__` tanımı (`define`) eklenmedi.
- Arayüz testleri (i18n anahtar eşitliği, tema çözümleme) yazılmadı.
- `npm run build` / `npm test` doğrulaması yapılmadı.

## Devam ederken

1. Yukarıdaki eksikleri tamamla, derle ve test et.
2. Aşama 3'ü `main` dalına küçük commit'ler halinde aktar.
3. `ROADMAP.md` üzerinden Aşama 4 (veritabanı ve yapılandırma) ile devam et.
