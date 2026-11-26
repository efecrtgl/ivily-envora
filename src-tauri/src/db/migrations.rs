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
    Migration {
        version: 4,
        name: "profiles",
        sql: r#"
        CREATE TABLE profiles (
            id          INTEGER PRIMARY KEY,
            name        TEXT NOT NULL UNIQUE COLLATE NOCASE,
            description TEXT NOT NULL DEFAULT '',
            category    TEXT NOT NULL DEFAULT '',
            tags        TEXT NOT NULL DEFAULT '[]',
            status      TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'archived')),
            version     INTEGER NOT NULL DEFAULT 1,
            metadata    TEXT NOT NULL DEFAULT '{}',
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );

        -- Hassas değerler value_enc (DPAPI) sütununda; value_plain yalnızca hassas olmayanlar için.
        CREATE TABLE profile_vars (
            profile_id  INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            position    INTEGER NOT NULL,
            name        TEXT NOT NULL,
            value_plain TEXT,
            value_enc   BLOB,
            sensitive   INTEGER NOT NULL DEFAULT 0,
            kind        TEXT,
            scope       TEXT NOT NULL DEFAULT 'user',
            action      TEXT NOT NULL DEFAULT 'set' CHECK (action IN ('set', 'unset')),
            PRIMARY KEY (profile_id, name COLLATE NOCASE)
        );

        CREATE TABLE profile_paths (
            profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            position   INTEGER NOT NULL,
            entry      TEXT NOT NULL,
            action     TEXT NOT NULL CHECK (action IN ('prepend', 'append', 'remove')),
            scope      TEXT NOT NULL DEFAULT 'user',
            PRIMARY KEY (profile_id, position)
        );

        -- Her düzenlemeden önceki tam profil (JSON). Hassas değer içeriyorsa data_enc (DPAPI).
        CREATE TABLE profile_revisions (
            id         INTEGER PRIMARY KEY,
            profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            version    INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            note       TEXT,
            var_count  INTEGER NOT NULL DEFAULT 0,
            data_plain TEXT,
            data_enc   BLOB
        );
        CREATE INDEX idx_profile_revisions ON profile_revisions(profile_id, version DESC);

        -- Kalıcı uygulamalar. snapshot_id, uygulama öncesi yedektir (devre dışı bırakmada kullanılır).
        CREATE TABLE profile_activations (
            id             INTEGER PRIMARY KEY,
            profile_id     INTEGER REFERENCES profiles(id) ON DELETE SET NULL,
            profile_name   TEXT NOT NULL,
            scope          TEXT NOT NULL,
            snapshot_id    INTEGER REFERENCES snapshots(id) ON DELETE SET NULL,
            applied_at     TEXT NOT NULL,
            deactivated_at TEXT
        );
        CREATE INDEX idx_profile_activations ON profile_activations(profile_id, deactivated_at);
    "#,
    },
    Migration {
        version: 5,
        name: "projects",
        sql: r#"
        CREATE TABLE projects (
            id          INTEGER PRIMARY KEY,
            name        TEXT NOT NULL UNIQUE COLLATE NOCASE,
            identifier  TEXT NOT NULL UNIQUE COLLATE NOCASE,
            description TEXT NOT NULL DEFAULT '',
            directory   TEXT NOT NULL,
            profile_id  INTEGER REFERENCES profiles(id) ON DELETE SET NULL,
            tags        TEXT NOT NULL DEFAULT '[]',
            launch      TEXT NOT NULL DEFAULT '{}',
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );

        -- profile_vars / profile_paths ile aynı yapı (bkz. envset.rs).
        CREATE TABLE project_vars (
            project_id  INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            position    INTEGER NOT NULL,
            name        TEXT NOT NULL,
            value_plain TEXT,
            value_enc   BLOB,
            sensitive   INTEGER NOT NULL DEFAULT 0,
            kind        TEXT,
            scope       TEXT NOT NULL DEFAULT 'process',
            action      TEXT NOT NULL DEFAULT 'set' CHECK (action IN ('set', 'unset')),
            PRIMARY KEY (project_id, name COLLATE NOCASE)
        );

        CREATE TABLE project_paths (
            project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            position   INTEGER NOT NULL,
            entry      TEXT NOT NULL,
            action     TEXT NOT NULL CHECK (action IN ('prepend', 'append', 'remove')),
            scope      TEXT NOT NULL DEFAULT 'process',
            PRIMARY KEY (project_id, position)
        );
    "#,
    },
    Migration {
        version: 6,
        name: "templates",
        sql: r#"
        -- Kullanıcı şablonları. Yerleşik şablonlar kodda tanımlıdır. Şablonlar gizli değer içermez.
        CREATE TABLE templates (
            id          INTEGER PRIMARY KEY,
            name        TEXT NOT NULL UNIQUE COLLATE NOCASE,
            description TEXT NOT NULL DEFAULT '',
            category    TEXT NOT NULL DEFAULT '',
            data        TEXT NOT NULL,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
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
