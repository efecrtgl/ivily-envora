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

pub const MIGRATIONS: &[Migration] = &[Migration {
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
}];

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
