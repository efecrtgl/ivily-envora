# Proje ortamları

Bir **proje**, bilgisayarınızdaki bir dizini bir ortamla ilişkilendirir. Kalıcı ortamı değiştirmeden,
yalnızca o projeden başlatılan programlara özel değişkenler vermenin yoludur.

## Proje içeriği

- Ad, tanımlayıcı (ör. `web-app`, addan üretilir), açıklama, etiketler
- **Dizin:** mutlak ve var olan bir dizin olmalıdır
- **İlişkili profil** (isteğe bağlı)
- **Projeye özgü değişkenler ve PATH değişiklikleri** — gizli değerler DPAPI ile şifreli saklanır
- **Başlatma yapılandırması:** varsayılan kabuk ve isteğe bağlı özel program

Proje eklenirken dizindeki `package.json`, `Cargo.toml`, `pyproject.toml` gibi dosyaların **yalnızca var
olup olmadığına** bakılarak proje türü gösterilir. Bu dosyalar okunmaz ve hiçbir şey çalıştırılmaz.

## Başlatma ortamı

**Başlatma ortamı** sekmesi, bu projeden başlatılan bir programın ortamının güncel kalıcı ortamdan farkını
gösterir (eklenen / değişen / kaldırılan; kaynak: profil veya proje). Hesap şöyledir:

1. Güncel kalıcı ortam (Envora'nın oturum değişkenleri + kayıt defterindeki güncel kullanıcı ve sistem
   değerleri, PATH birleştirilmiş, `%AD%` başvuruları genişletilmiş)
2. + İlişkili profilin **tüm** değişkenleri ve PATH değişiklikleri (kapsam tercihinden bağımsız)
3. + Projeye özgü değişkenler ve PATH değişiklikleri (profilin üzerine yazar)

Bu hesap, Explorer'ın yeni programlara verdiği ortamın yakın bir benzeridir; birebir aynısı değildir.

## Başlatma

Hiçbir şey projeyi açtığınızda veya kendiliğinden çalışmaz. Yalnızca siz bir düğmeye basıp onayladığınızda:

- **Terminal aç:** seçilen kabuk (PowerShell 7, Windows PowerShell veya cmd) proje dizininde, proje
  ortamıyla yeni bir pencerede açılır. Bulunmayan kabuklar listelenmez.
- **Özel komutu çalıştır:** projede tanımladığınız program, onay penceresinde **tam yolu ve argümanları**
  gösterildikten sonra çalışır.
  - Yalnızca `.exe` ve `.com` programlar desteklenir (betik dosyaları kabuk yorumuna yol açacağı için reddedilir).
  - Argümanlar her satıra bir tane olacak şekilde girilir ve kabuk tarafından yorumlanmaz.

İçe aktarılan proje dosyalarındaki özel komutlar varsayılan olarak alınmaz.

## Karşılaştırma ve anlık görüntü

- **Karşılaştır** ekranında iki projenin (profil + proje değişkenleri) tanımları karşılaştırılabilir.
- **Anlık görüntüler → Proje** kaynağı, projenin başlatma ortamını kaydeder (karşılaştırma içindir; kalıcı
  ortama geri yüklenemez).
