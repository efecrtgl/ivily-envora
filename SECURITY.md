# Güvenlik Politikası

## Desteklenen sürümler

Güvenlik düzeltmeleri yalnızca en son kararlı sürüme uygulanır.

## Güvenlik açığı bildirme

Güvenlik açıklarını **herkese açık bir issue ile bildirmeyin.** Bunun yerine GitHub'daki
**Security → Report a vulnerability** (özel güvenlik bildirimi) özelliğini kullanın.

Bildiriminize şunları ekleyin:

- Etkilenen Ivily Envora ve Windows sürümü,
- Yeniden üretme adımları,
- Olası etki.

**Gerçek ortam değişkeni değerlerini, token'ları veya parolaları bildirime eklemeyin.** Gerekirse
`ornek-deger` gibi yer tutucular kullanın.

## Kapsam

Özellikle şu alanlardaki sorunlar önemlidir:

- Hassas değerlerin günlüklere, hata mesajlarına, dışa aktarımlara veya etkinlik kaydına sızması,
- Yükseltilmiş yardımcı kipin plan şeması dışında işlem yapabilmesi,
- İçe aktarılan dosyaların komut çalıştırabilmesi,
- Güncelleme imzası doğrulamasının atlatılması.

Uygulamanın tehdit modeli ve bilinen sınırlamaları için [docs/GUVENLIK.md](docs/GUVENLIK.md) belgesine bakın.

*English: Please report vulnerabilities privately via GitHub's "Report a vulnerability" feature. Never include
real environment values, tokens or passwords in reports.*
