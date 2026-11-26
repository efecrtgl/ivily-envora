# Profiller

Bir **profil**, birlikte kullanılan değişkenleri ve PATH değişikliklerini bir arada tutar (ör. "Node.js
geliştirme", "Java 21"). Profiller Envora'nın veritabanında saklanır; siz uygulayana kadar ortamınızı
değiştirmez.

## Profil içeriği

- Ad, açıklama, kategori, etiketler
- **Değişkenler:** her biri için değer, tür (otomatik / metin / genişletilebilir), **kapsam tercihi**
  (kullanıcı / sistem / işlem) ve işlem (**ayarla** veya **kaldır**)
- **PATH değişiklikleri:** başa ekle / sona ekle / kaldır, kapsam tercihiyle
- Sürüm numarası, oluşturulma ve değişiklik tarihleri

Hassas değerler veritabanında Windows DPAPI ile şifrelenir. Düzenleyicide mevcut bir gizli değer
**"Değişmedi"** olarak görünür; değeri görmeden profilin geri kalanını düzenleyebilir veya **Değiştir** ile
yeni değer girebilirsiniz.

## İşlemler

- Oluştur, düzenle (değişiklik notuyla), çoğalt, yeniden adlandır, arşivle / arşivden çıkar, sil
- **Doğrulama:** profili değiştirmeden bulguları listeler — tanımsız `%AD%` başvuruları, bulunamayan veya
  göreli PATH dizinleri, kritik sistem değişkenleri, aynı anda hem PATH değişkeni hem PATH girdisi tanımı
- **Geçmiş:** her düzenlemeden önceki hal bir sürüm olarak saklanır (en fazla 50). Bir sürüm görüntülenebilir
  ve **geri yüklenebilir**; geri yükleme de yeni bir sürüm oluşturur. Kalıcı ortam bu işlemden etkilenmez.
- **Dışa aktar** (JSON) — bkz. [İçe ve dışa aktarma](ICE-DISA-AKTARMA.md)

## Kalıcı olarak uygulama

**Kalıcı olarak uygula**, profili her kalıcı kapsam (kullanıcı, sistem) için ayrı bir değişiklik planına
dönüştürür ve her birini sırayla önizletir:

- Kapsam tercihi **işlem** olan değişkenler kalıcı olarak **yazılmaz** (yalnızca proje başlatmada kullanılır).
- PATH değişiklikleri mevcut PATH'e uygulanır; önizlemede eklenen/kaldırılan girdiler görünür.
- Sistem kapsamı için UAC yalnızca o adımda istenir.
- Her uygulamadan önce otomatik yedek alınır ve bu yedek profilin "uygulama kaydına" bağlanır.

Planlar gizli değer içermez; değerler arka uçta profilden okunur.

## Geri alma (devre dışı bırakma)

Profil uygulanmışsa ayrıntı sayfasında **Geri al** görünür. Geri alma, uygulama öncesinde alınan yedekten bir
geri yükleme planı üretir ve önizletir: profilin eklediği değişkenler silinir, değiştirdikleri eski
değerlerine döner. Profil uygulandıktan sonra bu değişkenleri elle değiştirdiyseniz önizlemede görürsünüz.

Aynı kapsamda tekrar uygulanan bir profilin geri alınması, **ilk** uygulamadan önceki hale döner.

## Şablonlardan profil

**Şablonlar** ekranında bir şablon seçip **Bu şablondan profil oluştur** deyin. Düzenleyici şablon
değişkenleriyle açılır; "Zorunlu" değerleri doldurup kaydedin.
