# Testler

## Çalıştırma

```powershell
npm test                                    # arayüz birim ve bileşen testleri (Vitest + jsdom)
cd src-tauri
cargo test                                  # Rust birim ve entegrasyon testleri
cargo clippy --all-targets -- -D warnings   # lint
cargo fmt --check                           # biçim
node ../scripts/check-version.mjs           # sürüm tutarlılığı
```

Aynı adımlar her `main` gönderiminde ve çekme isteğinde GitHub Actions'ta (`.github/workflows/ci.yml`,
`windows-latest`) çalışır.

## İlke: gerçek ortam değiştirilmez

Otomatik testler kullanıcının veya sistemin ortamını **hiçbir zaman değiştirmez**.

- Yazma, silme, geri alma, profil uygulama, anlık görüntü geri yükleme ve içe aktarma akışları
  `MemoryProvider` (bellek içi ortam sağlayıcı) ile sınanır.
- Gerçek Windows kayıt defterine yalnızca **okuyan** testler vardır (ör. sistem PATH'inin okunabilmesi).
- DPAPI testleri gerçek Windows API'sini kullanır (yalnızca bellek içi veriyi şifreler/çözer).
- Veritabanı testleri bellek içi veya geçici dizindeki SQLite dosyalarını kullanır.

## Kapsam

| Alan | Testler |
|------|---------|
| Ortam modeli ve doğrulama | Ad/değer kuralları, lint bulguları, kapsam kuralları, büyük/küçük harf duyarsızlığı |
| Hassas değer algılama | Ad ve değer kalıpları, yanlış pozitif listesi, kullanıcı geçersiz kılmaları |
| Gizlilik | Liste, önizleme, plan, karşılaştırma, dışa aktarma, etkinlik, tanılama ve IEP yanıtlarının gizli değer içermediği; hata mesajlarının girdi değerini yansıtmadığı; veritabanında düz metin bulunmadığı |
| Değişiklik planları | Doğrulama, önizleme eylemleri, kritik değişken uyarıları, gölgeleme, uygulama, yedek, hata durumunda geri alma |
| Yükseltilmiş yardımcı | Plan şifreleme, kurcalanmış/yabancı/bilinmeyen işlem reddi, yalnızca sistem kapsamı, yalnızca `elevate` klasörü |
| PATH | Ayırma, anahtar karşılaştırma, ekleme/çıkarma/sıralama, analiz bayrakları, çakışmalar |
| Profiller | CRUD, doğrulama, revizyonlar, gizli değeri ön uca göndermeden düzenleme, etkinleştirme/devre dışı bırakma |
| Projeler | Dizin ve tanımlayıcı doğrulama, başlatma ortamı, `.exe` dışı programların reddi |
| Anlık görüntüler | Şifreli saklama, seçimli oluşturma, "yoktu" öğeleri, geri yükleme |
| İçe/dışa aktarma | `.env` varyasyonları, CSV formül koruması, JSON belgeleri, gidiş-dönüş, PATH birleştirme kipleri |
| Analiz | Kural bulguları, yapılandırma, sıralama |
| Araç keşfi | PATH/PATHEXT sırası, Store takma adlarının çalıştırılmaması, `package.json` sürümü |
| Veritabanı | Geçişler, yeniden açma, daha yeni şemanın reddi, işlem geri alma, yedek ve geri yükleme |
| Ayarlar | Toleranslı okuma, alan bazında düzeltme, katı yazma |
| IEP | Zarf doğrulama, sürüm, yetenek izinleri, değer/ad içermeyen özetler |
| Uçtan uca | `tests/workflow.rs`: profil → uygula → anlık görüntü → karşılaştır → dışa/içe aktar → geri al → geri yükle |
| Arayüz | i18n anahtar eşitliği ve kullanım denetimi, tema, filtreler, taslak işlemleri, `SecretValue`, `Modal`, `VarRowsEditor` |

## Elle doğrulananlar

Aşağıdakiler otomatik testlerde gerçek sisteme dokunmamak için **elle** doğrulanmıştır (geliştirme sırasında,
bu bilgisayarda):

- Kullanıcı kapsamında benzersiz adlı geçici bir değişkenin gerçek kayıt defterine yazılması
  (`REG_EXPAND_SZ` türüyle), otomatik yedek alınması ve silinmesi (`reg query` ile doğrulandı).
- Yükseltilmiş yardımcının yönetici olmadan çalıştırıldığında yazmayı reddetmesi, plan dosyasını silmesi,
  değer içermeyen sonuç yazması; yanlış klasördeki planı okumadan reddetmesi.
- Proje başlatmanın hesaplanan ortamı ve çalışma dizinini gerçek bir `cmd.exe` sürecine geçirmesi.
- Gerçek ortamda `.env`, CSV ve JSON dışa/içe aktarma gidiş-dönüşü.
- `--iep` komut satırı kipinin gerçek yürütülebilir dosyayla yanıt vermesi.
- Arayüz akışlarının (değişkenler, PATH, profiller, karşılaştırma, anlık görüntüler, analiz, araçlar,
  şablonlar, pano, etkinlik, dil değişimi) gerçek uygulamada ekran görüntüleriyle denetlenmesi.

## Bilinen sınırlamalar

- **UAC ile yükseltilmiş sistem kapsamı yazımı** otomatik test edilemez (onay istemi kullanıcı gerektirir);
  yardımcı mantığı bellek içi sağlayıcıyla, başlatma yolu ise yönetici olmadan doğrulanmıştır.
- **Pano** işlemleri kullanıcının panosunu değiştireceği için otomatik test edilmez.
- Yükleyici (NSIS) kurulumu ve güncelleme indirmesi otomatik test edilmez; sürüm iş akışı yükleyicinin
  ve imzasının üretildiğini doğrular.
- Uçtan uca arayüz testleri (gerçek WebView2 üzerinde) CI'da çalışmaz.
