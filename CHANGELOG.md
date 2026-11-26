# Değişiklik Günlüğü

Bu dosyadaki tüm önemli değişiklikler burada belgelenir.
Biçim [Keep a Changelog](https://keepachangelog.com/tr-TR/1.1.0/) temel alınarak hazırlanmıştır ve proje
[Anlamsal Sürümleme](https://semver.org/lang/tr/) kullanır.

## [Yayımlanmamış]

## [0.1.0] - 2026-11-27

İlk sürüm.

### Eklendi

- **Ortam değişkenleri:** işlem, kullanıcı ve sistem kapsamlarını inceleme; arama, kategori ve kapsam
  süzgeçleri; oluşturma, düzenleme, çoğaltma, yeniden adlandırma ve silme. `REG_SZ` / `REG_EXPAND_SZ`
  türü korunur; `%AD%` başvuruları önizlenir.
- **Güvenli değişiklik akışı:** her kalıcı değişiklik için önizleme, onay, otomatik yedek, hata durumunda
  geri alma ve `WM_SETTINGCHANGE` bildirimi. Sistem kapsamı için yalnızca o işlem UAC ile yükseltilir.
- **Gizli değer koruması:** ad ve değer kalıplarıyla hassas değişken algılama, kullanıcı geçersiz
  kılmaları, maskeleme, istek üzerine gösterme ve otomatik gizleme, pano geçmişine alınmayan ve süreli
  temizlenen kopyalama, uygulama verisinde Windows DPAPI şifrelemesi.
- **PATH yöneticisi:** girdi analizi (bulunamayan, yinelenen, göreli, çözülemeyen, tırnaklı, ağ yolu),
  uzunluk ölçeri, taslak tabanlı düzenleme ve sıralama, yürütülebilir dosya çakışmaları.
- **Profiller:** değişken ve PATH kümeleri, sürüm geçmişi ve geri yükleme, doğrulama, kalıcı uygulama ve
  geri alma, arşivleme.
- **Projeler:** dizin doğrulama, profil ilişkilendirme, projeye özgü değişkenler, başlatma ortamı
  önizlemesi, açık onayla terminal veya `.exe` başlatma.
- **Karşılaştırma:** kapsamlar, profiller, projeler, anlık görüntüler ve içe aktarılan dosyalar arasında.
- **Anlık görüntüler ve yedekleme:** elle ve otomatik anlık görüntüler, seçimli ve önizlemeli geri yükleme,
  veritabanı yedeği ve doğrulamalı geri yükleme.
- **İçe / dışa aktarma:** Ivily Envora JSON, CSV, `.env` ve PATH listesi; maskeli önizleme, çakışma
  tutumu, hassas değerleri varsayılan olarak hariç tutma; içe aktarılan içerik asla çalıştırılmaz.
- **Analiz:** 23 kurallık, yapılandırılabilir analiz motoru.
- **Araç keşfi:** 28 geliştirici aracı; sürümler dosya bilgisinden okunur, Store takma adları çalıştırılmaz.
- **Şablonlar:** 11 yerleşik, gizli değer içermeyen şablon; kullanıcı şablonları.
- **Pano, etkinlik merkezi, tanılama raporu** (değer içermez).
- **Ivily Ecosystem Protocol (IEP) v1:** yalnızca yerel, varsayılan olarak kapalı, yetenek tabanlı özetler.
- **Güncellemeler:** minisign imzalı paketler, kararlı/ön sürüm kanalı, kullanıcı onaylı kurulum.
- Türkçe (birincil) ve İngilizce arayüz; açık, koyu ve sistem teması; yoğunluk ve hareketi azaltma seçenekleri.
- NSIS yükleyicisi (kullanıcı başına, yönetici gerektirmez); GitHub Actions CI ve sürüm iş akışları.

### Bilinen sınırlamalar

- Yükleyici kod imzalı değildir (SmartScreen uyarısı görülebilir); güncelleme paketleri minisign imzalıdır.
- UAC ile sistem kapsamı yazımı ve pano işlemleri otomatik testlerde değil, elle doğrulanmıştır.
- Ayrıntılar: [docs/GUVENLIK.md](docs/GUVENLIK.md), [docs/TEST.md](docs/TEST.md).

[Yayımlanmamış]: https://github.com/efecrtgl/ivily-envora/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/efecrtgl/ivily-envora/releases/tag/v0.1.0
