# Ivily Ecosystem Protocol (IEP) — Envora uygulaması

IEP, Ivily ekosistemindeki uygulamaların (Ivily Center, Optimizer, Portly, Structura) birbirinden **yerel
olarak** ve **izinle** özet bilgi istemesi için tasarlanan protokoldür. Bu belge, protokolün Ivily Envora
tarafındaki **sürüm 1** uygulamasını tanımlar.

> Durum: Envora tarafı hazırdır ve test edilmiştir. Ivily Center henüz bu protokolü kullanmamaktadır.
> Envora, Ivily Center olmadan tamamen kullanılabilir; IEP varsayılan olarak **kapalıdır**.

## Tasarım ilkeleri

- **Yalnızca yerel.** Ağ bağlantı noktası açılmaz, sürekli çalışan bir hizmet yoktur.
- **İzin tabanlı.** Kullanıcı IEP'yi açmalı ve her yeteneğe ayrıca izin vermelidir.
- **Özet, değer değil.** Yanıtlar sayılar, profil/proje adları ve sağlık bilgisi içerir. Ortam değişkeni
  **değerleri ve adları hiçbir zaman** paylaşılmaz (birim testleriyle doğrulanır).
- **Sürümlü ve katı.** Bilinmeyen alanlar, bilinmeyen yöntemler ve desteklenmeyen sürümler reddedilir.
- **Bağımsız.** Envora diğer uygulamaları başlatmaz, onlara bağlanmaz ve onlara bağımlı değildir.

## Taşıma

İstemci, Envora'nın yürütülebilir dosyasını `--iep` argümanıyla başlatır, standart girdiye **tek** bir
JSON istek yazar ve girdiyi kapatır. Envora standart çıktıya **tek** bir JSON yanıt yazar ve çıkar.
Arayüz açılmaz.

Envora bir Windows (GUI alt sistemi) uygulamasıdır; istemci süreci standart girdi/çıktı kanallarıyla
başlatıp çıkmasını **beklemelidir** (ör. Rust `std::process::Command`, Node `child_process.spawnSync`,
.NET `Process` ile `RedirectStandardInput/Output`). PowerShell'in `|` işleci GUI uygulamalarını beklemediği
için çıktıyı yakalamaz.

```js
// Node.js örneği
const { spawnSync } = require('child_process');
const exe = `${process.env.LOCALAPPDATA}\\Ivily Envora\\ivily-envora.exe`;
const r = spawnSync(exe, ['--iep'], { input: JSON.stringify({ iep: 1, id: '1', method: 'iep.hello' }), encoding: 'utf8' });
console.log(JSON.parse(r.stdout));
```

- Çıkış kodu: `0` başarılı yanıt, `1` hata yanıtı.
- En büyük istek boyutu: 64 KB.
- Her başarılı istek Envora'nın etkinlik kaydına (`ecosystem / iep_request`, değer içermez) yazılır.

## İstek ve yanıt

```json
{ "iep": 1, "id": "istemci-kimliği", "method": "envora.health" }
```

| Alan | Tür | Açıklama |
|------|-----|----------|
| `iep` | sayı | Protokol sürümü. Şu an yalnızca `1`. |
| `id` | metin veya sayı | Yanıtta aynen döner. |
| `method` | metin | Aşağıdaki yöntemlerden biri. |
| `params` | nesne (isteğe bağlı) | Sürüm 1'de yöntemler parametre almaz; boş nesne veya yok. |

Başarılı yanıt:

```json
{ "iep": 1, "id": "istemci-kimliği", "result": { "status": "ok", "schemaVersion": 6, "integrity": true } }
```

Hata yanıtı:

```json
{ "iep": 1, "id": "istemci-kimliği", "error": { "code": "permission_denied", "message": "…" } }
```

| Hata kodu | Anlamı |
|-----------|--------|
| `invalid_request` | Geçersiz JSON, bilinmeyen alan veya geçersiz `id`. |
| `request_too_large` | İstek 64 KB'tan büyük. |
| `unsupported_version` | `iep` değeri desteklenmiyor. |
| `invalid_params` | Yöntem parametre almaz. |
| `unknown_method` | Yöntem tanımlı değil. |
| `disabled` | Kullanıcı IEP'yi Envora ayarlarında açmamış. |
| `permission_denied` | Yöntemin gerektirdiği yeteneğe izin verilmemiş. |
| `internal` | Envora iç hatası (ayrıntı değer içermez). |

## Yöntemler ve yetenekler

| Yöntem | Gerekli yetenek | Yanıt |
|--------|-----------------|-------|
| `iep.hello` | — (her zaman) | `appId`, `name`, `version`, `protocolVersions`, `enabled`, `capabilities`, `granted` |
| `envora.health` | `health` | `status` (`ok`/`degraded`), `schemaVersion`, `integrity` |
| `envora.profiles.summary` | `profiles.summary` | Profil başına `name`, `variables`, `pathChanges`, `activeScopes`, `status` |
| `envora.path.summary` | `path.summary` | Kapsam başına `entries`, `length`, `issues` (bayrak → sayı) |
| `envora.analysis.summary` | `analysis.summary` | `counts` (önem → sayı), `rules` (kural → sayı), `generatedAt` |
| `envora.activity.summary` | `activity.summary` | Son 7 gün için `byCategory` (kategori → sayı) |
| `envora.workspace.projects` | `workspace.projects` | Proje başına `name`, `identifier`, `directory`, `profile` |
| `envora.update.metadata` | `update.metadata` | `version`, `channel` |

`workspace.projects` yeteneği proje dizin yollarını paylaşır; bu kişisel bilgi sayılabileceği için ayrı bir
izindir.

## İzinler

Ayarlar → **Ivily ekosistemi (IEP)** bölümünde:

1. "Ekosistem isteklerine yanıt ver" açılır,
2. Paylaşılmasına izin verilen yetenekler tek tek işaretlenir,
3. Her yetenek için **Önizle** düğmesi, bir ekosistem uygulamasının alacağı yanıtı birebir gösterir.

Ayarlar veritabanında doğrulanarak saklanır; bilinmeyen yetenek adları yok sayılır.

## Gelecek sürümler için ayrılanlar

- **Kullanıcı onaylı işlemler** (ör. ekosistem uygulamasının bir profili etkinleştirmeyi *istemesi*):
  sürüm 1'de **uygulanmamıştır**. Eklenirse istek, Envora arayüzünde kullanıcıya önizlemeyle gösterilecek
  ve yalnızca kullanıcı onaylarsa uygulanacaktır; istemci hiçbir zaman doğrudan ortam değiştiremeyecektir.
- Parametreli yöntemler, olay bildirimleri ve kimlik doğrulama (ör. istemci imza denetimi) sürüm 2 için
  değerlendirilecektir.

Kaynak: [`src-tauri/src/iep/mod.rs`](../src-tauri/src/iep/mod.rs)
