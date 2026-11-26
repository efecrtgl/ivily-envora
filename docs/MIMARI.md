# Mimari

Ivily Envora; Rust (Tauri 2) arka ucu ve React (düz JavaScript) ön ucundan oluşan bir Windows masaüstü
uygulamasıdır. Bu belge katmanları, modül sınırlarını ve temel güvenlik kararlarını açıklar.

## Genel bakış

```
┌──────────────────────────── WebView2 (React) ─────────────────────────────┐
│ app/ kabuk, gezinme, ayarlar, tema     pages/ ekranlar (tembel yüklenir)    │
│ components/ tasarım sistemi + ortam bileşenleri   i18n/ TR (gömülü), EN     │
│ lib/api.js  ← tek IPC kapısı                                                │
└───────────────────────────────────┬────────────────────────────────────────┘
                                    │ Tauri invoke (yalnızca tanımlı komutlar)
┌───────────────────────────────────┴────────────────────────────────────────┐
│ commands/   ince komut katmanı: doğrulama → servis → etkinlik kaydı         │
├─────────────────────────────────────────────────────────────────────────────┤
│ env/        model · provider (Windows / Memory) · validate · category       │
│             inspect · plan · apply · elevate · effective                    │
│ pathmgr/    entries (ayırma, değişiklik) · analyze (bayraklar, çakışmalar)  │
│ security/   classify (hassasiyet) · overrides · vault (DPAPI)               │
│ envset      profil/proje değişken ve PATH depolaması (ortak)                │
│ profiles/   model · store · validate · activate                             │
│ projects/   model · store · launch          snapshots/  store · manage      │
│ compare/    io/ (json, csv, dotenv, import, export)    templates/           │
│ analysis/   kural motoru          discovery/  araç keşfi                    │
│ activity/   settings/   db/ (geçişler, yedek, geri yükleme)   sources       │
│ iep/        ekosistem protokolü   update      diagnostics   platform/       │
└─────────────────────────────────────────────────────────────────────────────┘
```

Yürütülebilir dosyanın üç kipi vardır (`main.rs`):

| Argüman | Kip |
|---------|-----|
| — | Normal masaüstü uygulaması |
| `--envora-elevated-apply <plan> <sonuç>` | Sistem kapsamı için UAC ile başlatılan yardımcı (arayüz yok) |
| `--iep` | Ivily Ecosystem Protocol: tek istek/yanıt (arayüz yok) — bkz. [IEP.md](IEP.md) |

## Ortam kapsamları ve sağlayıcılar

| Kapsam | Kaynak | Yazma |
|--------|--------|-------|
| İşlem | Envora'nın kendi ortamı (başlatıldığı andaki kopya) | Yazılmaz; yalnızca başlatılan süreçlere verilir |
| Kullanıcı | `HKCU\Environment` | Standart yetki |
| Sistem | `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment` | Yalnızca o işlem için yükseltilmiş yardımcı |

`EnvProvider` arayüzünün iki uygulaması vardır: `WindowsProvider` (kayıt defteri, `REG_SZ`/`REG_EXPAND_SZ`
korunur, değişiklik sonrası `WM_SETTINGCHANGE`) ve `MemoryProvider` (testler). Kalıcı bir değişiklikten sonra
açık programlar eski ortamla çalışmaya devam eder; arayüz bunu her değişiklikte belirtir.

`env/effective.rs`, yeni başlatılan bir sürecin göreceği ortamı yaklaşık hesaplar (oturum değişkenleri +
güncel sistem + güncel kullanıcı değerleri, `%AD%` genişletme, PATH birleştirme). Proje başlatma, araç
keşfi, karşılaştırma ve analiz bunu kullanır.

## Değişiklik planları

Kalıcı ortamdaki her değişiklik tek bir kapsama ait bir `ChangePlan` ile ifade edilir:

```
plan (ön uç) → validate → resolve → preview (maskeli) → onay → yedek (anlık görüntü) → apply → yayın
```

İşlem türleri: `set`, `delete`, `copy` (başka kapsam/addan), `fromProfile`, `fromSnapshot`, `fromImport`.
Son dördü değeri **arka uçta** bir `ValueSource` (`sources.rs`: veritabanı + içe aktarma önbelleği) üzerinden
çözer; böylece gizli değerler çoğaltma, yeniden adlandırma, profil uygulama, geri yükleme ve içe aktarma
sırasında WebView'e hiç gönderilmez.

`apply_ops` işlemleri sırayla uygular; biri başarısız olursa öncekileri eski hallerine döndürür.
`env_plan_apply` isteğe bağlı bir bağlam alır (`profileActivate`, `profileDeactivate`, `snapshotRestore`);
uygulama kayıtları ön uca güvenilmeden sunucu tarafında yazılır.

### Sistem kapsamı ve yükseltme

1. Çözümlenmiş plan JSON olarak serileştirilir, DPAPI (`LocalMachine`) ile şifrelenip
   `%LOCALAPPDATA%\IvilyEnvora\elevate\` altına yazılır.
2. Aynı yürütülebilir dosya `runas` ile (UAC) yardımcı kipte başlatılır.
3. Yardımcı yalnızca bu klasördeki dosyaları kabul eder, planı çözer ve yeniden doğrular, yalnızca sistem
   kapsamında ayarlama/silme yapar, değer içermeyen bir sonuç yazar.

## Gizli değerler

Ayrıntılar: [GUVENLIK.md](GUVENLIK.md). Özet: listelerde hassas değerler `None`; gösterme ayrı bir komut ve
etkinlik kaydına değer olmadan yazılır; kopyalama arka uçta (pano geçmişi dışı, süreli temizleme);
veritabanında DPAPI; etkinlik kaydı değer alanlarını reddeder.

## Veri katmanı

SQLite (`rusqlite`, gömülü). Sürümlü geçişler, WAL, `secure_delete`. Ayrıntılar: [VERITABANI.md](VERITABANI.md).

## Ön uç

- Yönlendirme kütüphanesi yoktur; `app/nav.js` sayfaları `React.lazy` ile yükler.
- Tüm arka uç çağrıları `src/lib/api.js` üzerinden geçer.
- Metinler `src/i18n` altında; Türkçe pakete gömülü, İngilizce yalnızca seçilince yüklenir.
- Tema yalnızca `<html data-theme>` ve CSS değişkenleriyle uygulanır; veriye dokunmaz.
- Değişiklik akışı `components/env/ChangeFlow.jsx` ile tektir; değişkenler, PATH, profiller, içe aktarma ve
  geri yükleme aynı önizleme/onay bileşenini kullanır.

## Genişletilebilirlik

Kurallar (`analysis/rules.rs`), araç tanımları (`discovery::TOOLS`), hassasiyet kalıpları
(`security/classify.rs`), yerleşik şablonlar (`templates/builtin.rs`), içe/dışa aktarma biçimleri (`io/`)
ve IEP yetenekleri (`iep::CAPABILITIES`) tablo olarak tanımlıdır; yeni bir öğe tek bir işlev ve tablo
satırıyla eklenir. Üçüncü taraf eklenti sistemi **yoktur**; ileride eklenirse yetenek sınırları, izinler ve
sürümleme ile tasarlanacaktır ve hassas değerlere açık bir izin olmadan erişemeyecektir.
