# Veritabanı

Ivily Envora, uygulama verisini gömülü bir SQLite veritabanında tutar (`rusqlite`, `bundled` özelliğiyle
derlenir; sistemde ayrıca SQLite kurulu olması gerekmez).

| Öğe | Değer |
|-----|-------|
| Konum | `%LOCALAPPDATA%\IvilyEnvora\envora.db` |
| Günlük kipi | WAL (`envora.db-wal`, `envora.db-shm` dosyaları normaldir) |
| Yabancı anahtarlar | Açık (`PRAGMA foreign_keys = ON`) |
| Güvenli silme | Açık (`PRAGMA secure_delete = ON`) — silinen sayfalar sıfırlanır |
| Şema sürümü | `PRAGMA user_version` |

Kaynak: [`src-tauri/src/db/`](../src-tauri/src/db/)

## Geçişler

Geçişler `src-tauri/src/db/migrations.rs` içindeki `MIGRATIONS` dizisinde tutulur.

- Her geçiş tek bir işlem (transaction) içinde uygulanır; başarısız olursa veritabanı önceki sürümde kalır.
- Yayımlanmış bir geçiş **asla değiştirilmez**. Şema değişikliği yeni bir geçiş olarak eklenir.
- Uygulama, kendisinden daha yeni bir şema sürümüne sahip veritabanını **açmayı reddeder** (eski bir sürüme
  geri dönüldüğünde veri bozulmasın diye). Bu durumda uygulama geçici bellek içi veritabanıyla açılır ve
  arayüzde bir uyarı gösterir; disk üzerindeki dosyaya dokunulmaz.

| Sürüm | Ad | İçerik |
|-------|----|--------|
| 1 | `settings_and_activity` | `settings`, `activity` tabloları |
| 2 | `sensitivity_overrides` | Kullanıcının hassasiyet geçersiz kılmaları |
| 3 | `snapshots` | `snapshots`, `snapshot_items` (hassas değerler DPAPI ile şifreli) |
| 4 | `profiles` | `profiles`, `profile_vars`, `profile_paths`, `profile_revisions`, `profile_activations` |
| 5 | `projects` | `projects`, `project_vars`, `project_paths` |
| 6 | `templates` | Kullanıcı şablonları (gizli değer içermez) |

## Tablolar

### `settings`

Anahtar–değer deposu. Şu an tek kayıt vardır: `app` anahtarı altında tüm ayarlar JSON olarak durur.
Ayarlar yalnızca kullanıcı bir değeri değiştirdiğinde yazılır.

Okuma toleranslıdır: eksik alanlar varsayılanla doldurulur, geçersiz bir alan yalnızca kendisi varsayılana
döner ve arayüzde hangi alanların düzeltildiği gösterilir. Böylece sürüm yükseltmeleri ayarları sıfırlamaz.
Yazma ise katıdır: geçersiz değer içeren bir güncelleme reddedilir.

### `activity`

| Sütun | Açıklama |
|-------|----------|
| `ts` | RFC 3339 zaman damgası (UTC) |
| `category` | `variable`, `path`, `profile`, `backup`, `settings` … |
| `action` | İşlem türü (`set`, `delete`, `reorder`, `database_backup` …) |
| `target` | Hedefin adı (değişken adı, profil adı); değer değil |
| `status` | `ok` \| `failed` \| `warning` |
| `details` | Sayısal özetler içeren JSON (ör. `{"scope":"user","count":3}`) |

**Değer içermez.** `activity::record`, `details` içinde `value`, `values`, `oldValue`, `newValue` veya
`secret` anahtarı bulunan kayıtları reddeder; bu kural birim testleriyle doğrulanır.
Saklama süresini (`activity.retentionDays`) aşan kayıtlar uygulama açılışında silinir.

### Diğer tablolar

| Tablo | İçerik |
|-------|--------|
| `sensitivity_overrides` | Büyük harfli ad → hassas mı (kullanıcı seçimi) |
| `snapshots` / `snapshot_items` | Anlık görüntü başlığı ve öğeleri; `present = 0` "yoktu" demektir; hassas değerler `value_enc` |
| `profiles` / `profile_vars` / `profile_paths` | Profil ve içeriği; hassas değerler `value_enc` |
| `profile_revisions` | Her düzenlemeden önceki tam profil (JSON; gizli değer içeriyorsa `data_enc`), profil başına en fazla 50 |
| `profile_activations` | Kalıcı uygulamalar: kapsam, uygulama öncesi yedek (`snapshot_id`), geri alınma zamanı |
| `projects` / `project_vars` / `project_paths` | Proje, başlatma yapılandırması (JSON) ve projeye özgü değişkenler |
| `templates` | Kullanıcı şablonları (değişkenler ve PATH JSON olarak) |

## Yedekleme ve bütünlük

- **Yedekleme:** Ayarlar → Veri → "Veritabanını yedekle". `VACUUM INTO` ile tutarlı bir kopya oluşturulur;
  var olan bir dosyanın üzerine yazılmaz.
- **Bütünlük:** Aynı bölümde `PRAGMA integrity_check` sonucu gösterilir.
- **Geri yükleme:** Ayarlar → Veri → "Yedekten geri yükle". Dosya salt okunur açılıp doğrulanır
  (bütünlük, şema sürümü), veri klasörüne `restore-pending.db` olarak hazırlanır ve uygulama yeniden
  başlatıldığında, veritabanı açılmadan önce yerine konur. Mevcut dosya `envora.db.bak-<zaman>` olarak
  saklanır, silinmez (`src-tauri/src/db/restore.rs`).

## Hassas veriler

Hassas olarak sınıflandırılan değerler (sonraki aşamalarda eklenen profil, proje ve anlık görüntü tablolarında)
veritabanına yazılmadan önce Windows DPAPI ile şifrelenir. Ayrıntılar: [GUVENLIK.md](GUVENLIK.md).
