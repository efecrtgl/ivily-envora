# Performans ölçümleri

Bu belgedeki tüm sayılar **gerçek ölçümlerdir**; nasıl ölçüldükleri ve ölçüm ortamı aşağıda belirtilmiştir.
Farklı donanımda ve farklı bir ortamda (PATH uzunluğu, değişken sayısı, kurulu araçlar) sonuçlar değişir.

## Ölçüm ortamı

| Öğe | Değer |
|-----|-------|
| İşlemci | AMD Ryzen 9 8940HX |
| Bellek | 31,2 GB |
| İşletim sistemi | Windows 11 (derleme 26200) |
| Ortam | 69 işlem, 4 kullanıcı, 17 sistem değişkeni; kullanıcı PATH 22, sistem PATH 17 girdi |
| Derleme | `npx tauri build --no-bundle` (release, LTO, `opt-level = "s"`) |
| Tarih | 2026-11-27, sürüm 0.1.0 |

## Uygulama

| Ölçüm | Sonuç | Yöntem |
|-------|-------|--------|
| Yürütülebilir dosya boyutu | 9,6 MB | `ivily-envora.exe` (release) |
| Pencerenin oluşması (ilk soğuk başlatma) | 232 ms | Süreç başlatma → `MainWindowHandle ≠ 0` |
| Pencerenin oluşması (sonraki başlatmalar) | 62–63 ms | Aynı yöntem, 2 tekrar |
| Envora sürecinin çalışma kümesi | 31–37 MB | Açılıştan 6 sn sonra, pano açıkken |
| Süreç ağacının özel belleği (Envora + 6 WebView2 alt süreci) | 285–288 MB | `PrivateMemorySize64` toplamı |
| Süreç ağacının çalışma kümesi toplamı | 377–389 MB | `WorkingSet64` toplamı (paylaşılan sayfalar birden çok sayılır) |
| Boştayken CPU (5 sn) | 16–31 ms toplam CPU süresi | Pano açık, kullanıcı etkileşimi yok |

Bellek kullanımının büyük kısmı Microsoft Edge WebView2 çalışma zamanına aittir; Envora'nın kendi süreci
~31–37 MB'tır. Boştayken CPU kullanımının neredeyse sıfır olması, arka planda yoklama (polling) veya sürekli
tarama yapılmadığını doğrular.

Ölçüm betiği: süreci başlatır, pencere tanıtıcısını 10 ms aralıklarla bekler, 6 sn sonra süreç ağacını
(`Win32_Process` üst süreç ilişkisiyle) bulur ve bellek değerlerini toplar.

## Arka uç işlemleri

Release derlemesinde, gerçek ortam üzerinde, salt okunur; her işlem 3–50 kez tekrarlanıp ortalaması alındı.

| İşlem | Optimizasyondan önce | Sonra |
|-------|---------------------:|------:|
| Değişken listesi (3 kapsam) | 0,21 ms | 0,22 ms |
| Ortama genel bakış | 0,22 ms | 0,22 ms |
| Yeni işlem ortamı hesabı | 0,12 ms | 0,13 ms |
| PATH analizi (kullanıcı) | 1,12 ms | 1,15 ms |
| Komut çakışması analizi | 5,10 ms | 5,79 ms |
| Ortam analizi (23 kural) | **19,10 ms** | **8,31–8,57 ms** |
| Araç keşfi (sürüm sorgusu olmadan) | **103,85 ms** | **10,07 ms** |
| Yeni veritabanı açma + 6 geçiş | 7,74 ms | 8,41 ms |

Küçük farklar (±%15) ölçüm gürültüsüdür.

### Yapılan iyileştirmeler

- **Analiz:** PATH kuralları her biri iki kapsamı yeniden analiz ediyordu (10 analiz + 1 dizin taraması).
  PATH raporları ve çakışma listesi artık analiz başına bir kez hesaplanıp kurallar arasında paylaşılıyor.
- **Araç keşfi:** Her araç × dizin × uzantı için ayrı dosya sorgusu (~4500) yerine her PATH dizini bir kez
  listeleniyor ve aramalar bellekte yapılıyor.
- **Ön uç paketi:** İngilizce sözlük yalnızca İngilizce seçildiğinde yükleniyor. Ana paket
  331,45 kB → 289,86 kB (gzip: 106,16 kB → 93,04 kB); İngilizce 41,93 kB'lık ayrı bir parça.

## Ön uç paketi

| Parça | Boyut | gzip |
|-------|------:|-----:|
| Ana paket (React, kabuk, Türkçe metinler, ortak bileşenler) | 289,86 kB | 93,04 kB |
| İngilizce sözlük (yalnızca seçilirse) | 41,93 kB | 13,98 kB |
| CSS | 18,46 kB | 4,58 kB |
| En büyük sayfa parçası (Ayarlar) | 14,64 kB | 3,94 kB |

Her sayfa ayrı bir parça olarak **yalnızca açıldığında** yüklenir.

## Tasarım gereği performans kararları

- Arka planda zamanlayıcı, dosya izleyici veya sürekli tarama yoktur.
- Analiz yalnızca istendiğinde veya pano ilk açıldığında (önceki sonuç yoksa) çalışır; sonuç bellekte tutulur.
- Araç keşfi kendiliğinden başlamaz; sürüm sorguları zaman aşımlıdır ve yalnızca `.exe` araçlar içindir.
- Değişken listesi 500 satırla sınırlanır (arama ile daraltılır); filtreleme bellekte ve ezberlenmiş (memoized) yapılır.
- SQLite WAL kipinde çalışır; etkinlik kaydı zaman ve kategori dizinlerine sahiptir.

## Bilinen sınırlamalar

- WebView2 bellek kullanımı Envora'nın denetiminde değildir.
- Ağ sürücülerini içeren PATH dizinleri, bağlantı yoksa PATH analizini ve araç keşfini yavaşlatabilir.
- Ölçümler tek bir bilgisayarda yapılmıştır; otomatik performans regresyon testi yoktur.
