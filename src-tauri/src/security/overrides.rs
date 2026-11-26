//! Kullanıcının hassasiyet sınıflandırması geçersiz kılmaları.

use crate::db::Db;
use crate::env::model::name_key;
use crate::error::AppResult;
use std::collections::HashMap;

/// `name_key` → hassas mı
pub fn all(db: &Db) -> AppResult<HashMap<String, bool>> {
    db.with(|c| {
        let mut stmt = c.prepare("SELECT name_key, sensitive FROM sensitivity_overrides")?;
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? != 0))
        })?;
        Ok(rows.collect::<Result<HashMap<_, _>, _>>()?)
    })
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverrideEntry {
    pub name: String,
    pub sensitive: bool,
    pub updated_at: String,
}

pub fn list(db: &Db) -> AppResult<Vec<OverrideEntry>> {
    db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT name, sensitive, updated_at FROM sensitivity_overrides ORDER BY name_key",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(OverrideEntry {
                name: r.get(0)?,
                sensitive: r.get::<_, i64>(1)? != 0,
                updated_at: r.get(2)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    })
}

/// `None` geçersiz kılmayı kaldırır ve otomatik algılamaya döner.
pub fn set(db: &Db, name: &str, sensitive: Option<bool>) -> AppResult<()> {
    let key = name_key(name);
    db.with(|c| {
        match sensitive {
            Some(s) => {
                c.execute(
                    "INSERT INTO sensitivity_overrides(name_key, name, sensitive, updated_at) VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(name_key) DO UPDATE SET name = excluded.name, sensitive = excluded.sensitive,
                     updated_at = excluded.updated_at",
                    rusqlite::params![key, name, s as i64, chrono::Utc::now().to_rfc3339()],
                )?;
            }
            None => {
                c.execute("DELETE FROM sensitivity_overrides WHERE name_key = ?1", [key])?;
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_clear_case_insensitive() {
        let db = Db::open_in_memory().unwrap();
        set(&db, "My_Token", Some(false)).unwrap();
        assert_eq!(all(&db).unwrap().get("MY_TOKEN"), Some(&false));
        set(&db, "MY_TOKEN", Some(true)).unwrap();
        assert_eq!(all(&db).unwrap().len(), 1);
        assert_eq!(all(&db).unwrap().get("MY_TOKEN"), Some(&true));
        set(&db, "my_token", None).unwrap();
        assert!(all(&db).unwrap().is_empty());
    }
}
