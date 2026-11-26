//! Yerel etkinlik kaydı.
//!
//! Kayıtlar işlem türü, hedef adı (ör. değişken adı veya profil adı) ve sayısal özetler içerir.
//! Ortam değişkeni **değerleri hiçbir zaman** kaydedilmez. `details` içinde değer taşıyabilecek
//! anahtarlar (`value`, `oldValue`, `newValue`, `values`) bulunursa kayıt reddedilir.

use crate::db::Db;
use crate::error::{AppError, AppResult};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Failed,
    Warning,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::Failed => "failed",
            Status::Warning => "warning",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: i64,
    pub ts: String,
    pub category: String,
    pub action: String,
    pub target: Option<String>,
    pub status: String,
    pub details: Option<Value>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Query {
    pub search: Option<String>,
    /// Hedef adına tam eşleşme (ör. bir projenin geçmişi).
    pub target: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub limit: Option<u32>,
    pub before_id: Option<i64>,
    /// `desc` (varsayılan) | `asc`
    pub order: Option<String>,
}

const FORBIDDEN_KEYS: &[&str] = &[
    "value",
    "values",
    "oldValue",
    "newValue",
    "old_value",
    "new_value",
    "secret",
];

fn contains_value_key(v: &Value) -> bool {
    match v {
        Value::Object(map) => map
            .iter()
            .any(|(k, v)| FORBIDDEN_KEYS.contains(&k.as_str()) || contains_value_key(v)),
        Value::Array(xs) => xs.iter().any(contains_value_key),
        _ => false,
    }
}

/// Bir etkinlik kaydı ekler. `details` değer içermemelidir.
pub fn record(
    db: &Db,
    category: &str,
    action: &str,
    target: Option<&str>,
    status: Status,
    details: Option<Value>,
) -> AppResult<()> {
    if let Some(d) = &details {
        if contains_value_key(d) {
            return Err(AppError::Internal(
                "etkinlik ayrıntıları değer alanı içeremez".into(),
            ));
        }
    }
    let ts = chrono::Utc::now().to_rfc3339();
    let details = details.map(|d| d.to_string());
    db.with(|c| {
        c.execute(
            "INSERT INTO activity(ts, category, action, target, status, details) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![ts, category, action, target, status.as_str(), details],
        )?;
        Ok(())
    })
}

/// Etkinlik kaydı başarısız olsa bile asıl işlemi bozmayan yardımcı.
pub fn record_quiet(
    db: &Db,
    category: &str,
    action: &str,
    target: Option<&str>,
    status: Status,
    details: Option<Value>,
) {
    let _ = record(db, category, action, target, status, details);
}

pub fn list(db: &Db, q: &Query) -> AppResult<Vec<Entry>> {
    let limit = q.limit.unwrap_or(200).min(1000) as i64;
    let asc = q.order.as_deref() == Some("asc");
    let mut sql = String::from(
        "SELECT id, ts, category, action, target, status, details FROM activity WHERE 1=1",
    );
    let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(cat) = q.category.as_ref().filter(|s| !s.is_empty()) {
        sql.push_str(" AND category = ?");
        args.push(Box::new(cat.clone()));
    }
    if let Some(tg) = q.target.as_ref().filter(|s| !s.is_empty()) {
        sql.push_str(" AND target = ?");
        args.push(Box::new(tg.clone()));
    }
    if let Some(st) = q.status.as_ref().filter(|s| !s.is_empty()) {
        sql.push_str(" AND status = ?");
        args.push(Box::new(st.clone()));
    }
    if let Some(s) = q
        .search
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        sql.push_str(" AND (action LIKE ? ESCAPE '\\' OR target LIKE ? ESCAPE '\\' OR category LIKE ? ESCAPE '\\')");
        let pat = format!(
            "%{}%",
            s.replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_")
        );
        for _ in 0..3 {
            args.push(Box::new(pat.clone()));
        }
    }
    if let Some(b) = q.before_id {
        sql.push_str(if asc { " AND id > ?" } else { " AND id < ?" });
        args.push(Box::new(b));
    }
    sql.push_str(if asc {
        " ORDER BY id ASC LIMIT ?"
    } else {
        " ORDER BY id DESC LIMIT ?"
    });
    args.push(Box::new(limit));

    db.with(|c| {
        let mut stmt = c.prepare(&sql)?;
        let rows = stmt.query_map(
            rusqlite::params_from_iter(args.iter().map(|a| a.as_ref())),
            |r| {
                let details: Option<String> = r.get(6)?;
                Ok(Entry {
                    id: r.get(0)?,
                    ts: r.get(1)?,
                    category: r.get(2)?,
                    action: r.get(3)?,
                    target: r.get(4)?,
                    status: r.get(5)?,
                    details: details.and_then(|d| serde_json::from_str(&d).ok()),
                })
            },
        )?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    })
}

pub fn clear(db: &Db) -> AppResult<usize> {
    db.with(|c| Ok(c.execute("DELETE FROM activity", [])?))
}

/// Saklama süresini aşan kayıtları siler.
pub fn prune(db: &Db, retention_days: u32) -> AppResult<usize> {
    let cutoff = (chrono::Utc::now() - chrono::Duration::days(retention_days as i64)).to_rfc3339();
    db.with(|c| Ok(c.execute("DELETE FROM activity WHERE ts < ?1", [cutoff])?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn records_and_filters() {
        let db = Db::open_in_memory().unwrap();
        record(
            &db,
            "variable",
            "set",
            Some("JAVA_HOME"),
            Status::Ok,
            Some(json!({"scope": "user"})),
        )
        .unwrap();
        record(&db, "path", "reorder", None, Status::Failed, None).unwrap();
        assert_eq!(list(&db, &Query::default()).unwrap().len(), 2);
        let q = Query {
            category: Some("path".into()),
            ..Default::default()
        };
        let r = list(&db, &q).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].status, "failed");
        let q = Query {
            search: Some("java".into()),
            ..Default::default()
        };
        assert_eq!(list(&db, &q).unwrap().len(), 1);
        let q = Query {
            search: Some("%".into()),
            ..Default::default()
        };
        assert_eq!(
            list(&db, &q).unwrap().len(),
            0,
            "LIKE joker karakterleri kaçışlanmalı"
        );
    }

    #[test]
    fn rejects_details_with_values() {
        let db = Db::open_in_memory().unwrap();
        let bad = json!({"changes": [{"name": "TOKEN", "newValue": "ghp_x"}]});
        assert!(record(&db, "variable", "set", Some("TOKEN"), Status::Ok, Some(bad)).is_err());
        assert!(list(&db, &Query::default()).unwrap().is_empty());
    }

    #[test]
    fn prune_and_clear() {
        let db = Db::open_in_memory().unwrap();
        db.with(|c| {
            c.execute(
                "INSERT INTO activity(ts, category, action, status) VALUES ('2000-01-01T00:00:00+00:00', 'x', 'old', 'ok')",
                [],
            )?;
            Ok(())
        })
        .unwrap();
        record(&db, "x", "new", None, Status::Ok, None).unwrap();
        assert_eq!(prune(&db, 30).unwrap(), 1);
        assert_eq!(clear(&db).unwrap(), 1);
    }
}
