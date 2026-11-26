# Yedekleme ve geri yükleme

Envora'da iki tür yedek vardır:

| Tür | Ne korur | Nerede |
|-----|----------|--------|
| **Anlık görüntü** | Ortam değişkenlerinin (veya bir profilin/projenin) belirli bir andaki hali | Envora veritabanı |
| **Veritabanı yedeği** | Envora'nın tüm verisi: profiller, projeler, anlık görüntüler, şablonlar, ayarlar, etkinlik | Seçtiğiniz bir `.db` dosyası |

## Anlık görüntüler

### Otomatik yedekler

Her kalıcı değişiklikten önce (değişken düzenleme, PATH, profil uygulama, içe aktarma, geri yükleme)
yalnızca **etkilenen değişkenlerin** önceki hali "Otomatik yedek" olarak kaydedilir. Değişiklikten önce
var olmayan değişkenler "yoktu" olarak işaretlenir; geri yüklemede bunlar silinir.

Ayarlar → Yedekleme:
- **Değişikliklerden önce otomatik yedek al** (varsayılan: açık; kapatılması önerilmez)
- **Saklanacak otomatik yedek sayısı** (varsayılan 50); daha eskileri silinir, elle alınanlar silinmez

### Elle anlık görüntü

**Anlık görüntüler → Anlık görüntü al**:
- **Kaynak:** Ortam (seçilen kapsamlar, isteğe bağlı yalnızca belirli değişkenler), Profil veya Proje
- **Hassas değişkenleri hariç tut:** açıksa gizli değerler kaydedilmez; kapalıysa DPAPI ile şifreli saklanır

### Geri yükleme

1. Anlık görüntüyü açın; isterseniz yalnızca belirli değişkenleri işaretleyin.
2. **Tümünü / Seçilenleri geri yükle** → her kalıcı kapsam için ayrı önizleme → onay.
3. Geri yüklemeden önce mevcut hal de otomatik olarak yedeklenir; yani geri yükleme de geri alınabilir.

İşlem kapsamı öğeleri kalıcı ortama geri yüklenemez (karşılaştırma için kullanılabilir).
**Güncel ortamla karşılaştır** düğmesi, geri yüklemeden önce farkları görmenizi sağlar.

## Veritabanı yedeği

- **Ayarlar → Veri → Veritabanını yedekle:** tutarlı bir kopya (`VACUUM INTO`) oluşturur; var olan bir
  dosyanın üzerine yazmaz.
- **Ayarlar → Veri → Yedekten geri yükle:**
  1. Seçilen dosya salt okunur açılır; bütünlük, şema sürümü ve içerik özeti gösterilir.
  2. Geçerli ve bu sürümden yeni değilse veri klasörüne hazırlanır.
  3. Uygulama yeniden başlatıldığında, veritabanı açılmadan önce mevcut dosya `envora.db.bak-<zaman>`
     adıyla saklanır ve yedek yerine konur. Mevcut veritabanı **silinmez**.

Veritabanı yedekleri ortam değişkenlerinizi değiştirmez; yalnızca Envora'nın verisini değiştirir.
Hassas değerler yedek dosyada da DPAPI ile şifrelidir; yedek **yalnızca aynı Windows hesabında**
çözülebilir.
