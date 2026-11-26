# PATH rehberi

`PATH`, bir komutu (ör. `node`, `git`) adıyla yazdığınızda Windows'un hangi dizinlerde arayacağını belirler.
Dizinler `;` ile ayrılır ve **sırayla** aranır; bulunan ilk program çalışır. Yeni bir işlemin PATH'i,
sistem PATH'i ile kullanıcı PATH'inin birleşimidir (önce sistem).

## PATH yöneticisi

**PATH** ekranında üç sekme vardır: Kullanıcı, Sistem ve İşlem (salt okunur). Her girdi için bayraklar:

| Bayrak | Anlamı | Önerilen işlem |
|--------|--------|----------------|
| Bulunamadı / Dizin değil | Dizin yok veya bir dosyayı gösteriyor | Program kaldırılmışsa girdiyi kaldırın |
| Yinelenen | Aynı dizin bu kapsamda daha önce var | Sonraki kopyanın etkisi yok; kaldırılabilir |
| Diğer kapsamda da var | Hem kullanıcı hem sistem PATH'inde | Genellikle birinde tutmak yeter |
| Göreli | `bin` gibi mutlak olmayan yol | Mutlak yol kullanın (güvenlik ve tutarlılık) |
| Çözülemeyen başvuru | Tanımsız `%AD%` | Değişkeni tanımlayın veya girdiyi düzeltin |
| Boş | `;;` | Zararsız ama gereksiz |
| Tırnaklı / Boşluk | `"C:\x"` veya baş/son boşluk | Birçok araç tırnaklı girdiyi anlamaz |
| Ağ yolu | `\\sunucu\paylaşım` | Bağlantı yokken komut aramayı yavaşlatabilir |

Envora **hiçbir girdiyi otomatik olarak kaldırmaz veya sıralamaz.**

### Düzenleme

1. Girdileri ekleyin (başa/sona), düzenleyin, yukarı/aşağı taşıyın veya kaldırın. "Yinelenenleri seç",
   "Bulunamayanları seç" gibi düğmeler yalnızca seçim yapar.
2. Tüm değişiklikler bir **taslakta** birikir; "Değişiklikleri at" ile vazgeçebilirsiniz.
3. **Değişiklikleri incele** → önizlemede eklenen (+) ve kaldırılan (−) girdiler, sıra değişikliği uyarısı →
   **Uygula**. Öncesinde otomatik yedek alınır.

### Uzunluk

Windows'ta bir değişken en fazla 32.767 karakter olabilir; ancak PATH 2.047 karakteri aştığında bazı eski
araçlar ve kurulum programları onu kesebilir. Ekrandaki ölçer bu eşiğe göre gösterilir.

## Komut çakışmaları

"Komut çakışmaları" bölümü, aynı adlı bir programın birden fazla PATH dizininde bulunduğu durumları listeler.
İlk sıradaki kopya **kullanılır**, diğerleri **gölgededir**. Sık görülen bir örnek: Microsoft Store'un
`python.exe` takma adı (WindowsApps) gerçek Python kurulumundan önce gelirse `python` komutu Store'u açar.
Bu takma adlar **Ayarlar → Uygulamalar → Gelişmiş uygulama ayarları → Uygulama yürütme takma adları**
bölümünden kapatılabilir. Çakışma analizi dizinleri yalnızca listeler; hiçbir dosyayı çalıştırmaz.

## Profillerde PATH

Profiller ve projeler PATH'i tamamen değiştirmez; "başa ekle", "sona ekle" ve "kaldır" değişiklikleri
tanımlar. Kalıcı uygulamada bu değişiklikler mevcut PATH'e uygulanır ve önizlemede gösterilir.
