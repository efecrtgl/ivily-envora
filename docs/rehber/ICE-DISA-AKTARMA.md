# İçe ve dışa aktarma

## Biçimler

| Biçim | Ne için | Dışa | İçe |
|-------|---------|:----:|:---:|
| Ivily Envora JSON | Değişkenler, profil, proje, anlık görüntü, PATH, şablon | ✓ | ✓ |
| CSV (`name,value,scope,kind`) | Değişkenler | ✓ | ✓ |
| `.env` | Değişkenler | ✓ | ✓ |
| Metin (satır başına bir dizin) | PATH | ✓ | ✓ |

## Dışa aktarma

- **İçe / dışa aktar** ekranından: ortam değişkenleri (kapsam ve biçim seçerek), kullanıcı PATH, sistem PATH.
- Profil, proje, anlık görüntü ve şablon sayfalarındaki **Dışa aktar** düğmesiyle.
- **Hassas değerler varsayılan olarak dahil edilmez.** JSON'da adları `"excluded": true` ile kalır;
  CSV ve `.env`'de satır tamamen çıkarılır.
- "Hassas değerleri dahil et" açılırsa dosya **düz metin** gizli değer içerir; bunu ayrıca onaylamanız
  istenir ve JSON belgesi `"containsSecrets": true` ile işaretlenir.
- CSV'de elektronik tablolarda formül olarak yorumlanabilecek hücrelerin (`=`, `+`, `-`, `@` ile başlayan)
  başına `'` eklenir; Envora içe aktarırken bunu geri alır.
- Yalnızca `.json`, `.csv`, `.env` ve `.txt` uzantılı dosyalara yazılır.

## İçe aktarma

1. **Dosya seç** (biçim otomatik algılanır veya elle seçilir; en fazla 5 MB, UTF-8 veya UTF-16).
2. Dosya **ayrıştırılır ve önizlenir**; hiçbir şey değişmez. Uyarılar satır numarasıyla gösterilir.
3. Türe göre:
   - **Değişkenler:** hangilerinin alınacağını seçin, hedef kapsamı (kullanıcı/sistem) ve var olanlar için
     tutumu (üzerine yaz / atla) belirleyin. İsterseniz önce **karşılaştırın**. Ardından normal değişiklik
     akışı (önizleme → yedek → onay) çalışır.
   - **PATH listesi:** hedef kapsam ve birleştirme kipi — eksikleri sona ekle, eksikleri başa ekle, tamamen değiştir.
   - **Profil / proje / anlık görüntü / şablon:** ad (çakışırsa "(içe aktarılan)" eklenir) ve proje için
     dizin seçilip oluşturulur. Değeri olmayan (dışa aktarılırken hariç tutulmuş) değişkenler atlanır.

### Güvenlik

- İçe aktarılan içerik **asla çalıştırılmaz veya genişletilmez.** `.env` içindeki `$(komut)`, `` `komut` ``,
  `$AD` ve `${AD}` biçimleri metin olarak alınır ve uyarı gösterilir.
- İçe aktarılan proje dosyasındaki **özel başlatma komutu varsayılan olarak alınmaz**; komut gösterilir ve
  yalnızca "Bu komutu koru" işaretlenirse saklanır (yine de yalnızca siz başlattığınızda çalışır).
- Ayrıştırılan değerler arka uçta en fazla 30 dakika bellekte tutulur ve arayüze gönderilmez.
- Dosya gizli değer içeriyorsa uyarılırsınız; içe aktarmadan sonra dosyayı silmeyi değerlendirin.

## `.env` varyasyonları

`.env` biçiminin tek bir standardı yoktur. Envora şunları destekler:

```dotenv
# yorum
export A=1                 # "export " öneki
B = iki kelime # satır içi yorum (tırnaksız değerlerde " #" sonrası)
C='değişmez $HOME'         # tek tırnak: olduğu gibi
D="satır1\nsatır2 \"tırnak\""   # çift tırnak: \n \t \" \\ kaçışları
E="çok
satırlı"                   # çok satırlı çift tırnak
F=                         # boş değer
```

Aynı ad birden çok kez geçerse **son tanım** kullanılır (uyarı gösterilir).

## JSON belge yapısı

```json
{
  "format": "ivily-envora",
  "version": 1,
  "kind": "variables",
  "exportedAt": "2026-11-27T10:00:00Z",
  "appVersion": "0.1.0",
  "containsSecrets": false,
  "data": {
    "variables": [
      { "name": "JAVA_HOME", "value": "C:\\jdk-21", "scope": "user", "kind": "string", "sensitive": false, "excluded": false, "action": "set" },
      { "name": "GITHUB_TOKEN", "value": null, "scope": "user", "kind": "string", "sensitive": true, "excluded": true, "action": "set" }
    ]
  }
}
```

`kind` değerleri: `variables`, `profile`, `project`, `snapshot`, `path`, `template`. Daha yeni bir `version`
içeren veya `format` değeri farklı belgeler reddedilir.
