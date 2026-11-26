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

## Yedekleme ve bütünlük

- **Yedekleme:** Ayarlar → Veri → "Veritabanını yedekle". `VACUUM INTO` ile tutarlı bir kopya oluşturulur;
  var olan bir dosyanın üzerine yazılmaz.
- **Bütünlük:** Aynı bölümde `PRAGMA integrity_check` sonucu gösterilir.
- **Geri yükleme:** Uygulamayı kapatın, `envora.db`, `envora.db-wal` ve `envora.db-shm` dosyalarını başka bir
  yere taşıyın ve yedek dosyasını `envora.db` adıyla bu klasöre kopyalayın.

## Hassas veriler

Hassas olarak sınıflandırılan değerler (sonraki aşamalarda eklenen profil, proje ve anlık görüntü tablolarında)
veritabanına yazılmadan önce Windows DPAPI ile şifrelenir. Ayrıntılar: [GUVENLIK.md](GUVENLIK.md).
