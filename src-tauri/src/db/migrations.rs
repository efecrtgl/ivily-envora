//! Sürümlü şema geçişleri.
//!
//! Her geçiş `PRAGMA user_version` değerini bir artırır ve tek bir işlem (transaction) içinde
//! uygulanır; yarıda kalan bir geçiş veritabanını tutarsız bırakmaz. Var olan geçişler
//! **asla değiştirilmez**; şema değişiklikleri yeni bir geçiş olarak listenin sonuna eklenir.

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "settings_and_activity",
        sql: r#"
        CREATE TABLE settings (
            key        TEXT PRIMARY KEY,
            value      TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Etkinlik kaydı. Ortam değişkeni DEĞERLERİ bu tabloya hiçbir zaman yazılmaz;
        -- yalnızca işlem türü, hedef adı ve sayısal özetler tutulur.
        CREATE TABLE activity (
            id       INTEGER PRIMARY KEY,
            ts       TEXT NOT NULL,
            category TEXT NOT NULL,
            action   TEXT NOT NULL,
            target   TEXT,
            status   TEXT NOT NULL,
            details  TEXT
        );
        CREATE INDEX idx_activity_ts ON activity(ts DESC);
        CREATE INDEX idx_activity_category ON activity(category, ts DESC);
    "#,
    },
    Migration {
        version: 2,
        name: "sensitivity_overrides",
        sql: r#"
        -- Kullanıcının hassasiyet sınıflandırmasını geçersiz kıldığı değişken adları.
        -- name_key büyük harfe çevrilmiş addır (Windows adları büyük/küçük harfe duyarsızdır).
        CREATE TABLE sensitivity_overrides (
            name_key   TEXT PRIMARY KEY,
            name       TEXT NOT NULL,
            sensitive  INTEGER NOT NULL CHECK (sensitive IN (0, 1)),
            updated_at TEXT NOT NULL
        );
    "#,
    },
    Migration {
        version: 3,
        name: "snapshots",
        sql: r#"
        CREATE TABLE snapshots (
            id          INTEGER PRIMARY KEY,
            name        TEXT NOT NULL,
            description TEXT,
            category    TEXT NOT NULL DEFAULT 'manual',      -- manual | auto
            source      TEXT NOT NULL DEFAULT 'environment', -- environment | profile | project
            created_at  TEXT NOT NULL,
            item_count  INTEGER NOT NULL DEFAULT 0,
            has_secrets INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX idx_snapshots_created ON snapshots(created_at DESC);

        -- Hassas değerler value_enc (DPAPI) sütununda; value_plain yalnızca hassas olmayanlar için.
        -- present = 0: değişken anlık görüntü anında yoktu (geri yüklemede silinir).
        CREATE TABLE snapshot_items (
            snapshot_id INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
            scope       TEXT NOT NULL,
            name        TEXT NOT NULL,
            kind        TEXT NOT NULL,
            present     INTEGER NOT NULL DEFAULT 1,
            sensitive   INTEGER NOT NULL DEFAULT 0,
            value_plain TEXT,
            value_enc   BLOB,
            PRIMARY KEY (snapshot_id, scope, name)
        );
    "#,
    },
];

pub fn latest_version() -> i64 {
    MIGRATIONS.last().map(|m| m.version).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_are_sequential() {
        for (i, m) in MIGRATIONS.iter().enumerate() {
            assert_eq!(m.version, i as i64 + 1, "geçiş {} sıra dışı", m.name);
        }
    }
}
