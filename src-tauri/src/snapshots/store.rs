//! Anlık görüntü deposu.
//!
//! Hassas değerler `snapshot_items.value_enc` sütununda DPAPI ile şifreli tutulur;
//! `value_plain` yalnızca hassas olmayan değerler için doludur. Bir öğenin `present = 0`
//! olması, anlık görüntü alındığında o değişkenin **bulunmadığını** belirtir (geri yüklemede silinir).

use crate::db::Db;
use crate::env::{Scope, ValueKind};
use crate::error::{AppError, AppResult};
use crate::security::vault;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotItem {
    pub scope: Scope,
    pub name: String,
    pub kind: ValueKind,
    pub present: bool,
    pub sensitive: bool,
    /// Çözülmüş değer. Ön uca gönderilmeden önce hassas değerler maskelenir.
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotMeta {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    /// `manual` | `auto`
    pub category: String,
    /// `environment` | `profile` | `project`
    pub source: String,
    pub created_at: String,
    pub item_count: i64,
    pub has_secrets: bool,
    pub scopes: Vec<Scope>,
}

pub struct NewSnapshot<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub category: &'a str,
    pub source: &'a str,
}

fn parse_scope(s: &str) -> rusqlite::Result<Scope> {
    Scope::parse(s).ok_or_else(|| {
        rusqlite::Error::InvalidColumnType(0, "scope".into(), rusqlite::types::Type::Text)
    })
}

pub fn create(db: &Db, meta: NewSnapshot, items: &[SnapshotItem]) -> AppResult<i64> {
    if meta.name.trim().is_empty() {
        return Err(AppError::validation("invalid_snapshot_name", ""));
    }
    // Şifrelemeyi işlemden önce yap: DPAPI hatası veritabanını yarım bırakmasın.
    let mut rows = Vec::with_capacity(items.len());
    for it in items {
        let (plain, enc) = match (&it.value, it.sensitive) {
            (Some(v), true) => (None, Some(vault::protect_str(v)?)),
            (Some(v), false) => (Some(v.clone()), None),
            (None, _) => (None, None),
        };
        rows.push((it, plain, enc));
    }
    let has_secrets = items.iter().any(|i| i.sensitive && i.value.is_some());
    db.tx(|tx| {
        tx.execute(
            "INSERT INTO snapshots(name, description, category, source, created_at, item_count, has_secrets)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                meta.name.trim(),
                meta.description,
                meta.category,
                meta.source,
                chrono::Utc::now().to_rfc3339(),
                items.len() as i64,
                has_secrets as i64
            ],
        )?;
        let id = tx.last_insert_rowid();
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO snapshot_items(snapshot_id, scope, name, kind, present, sensitive, value_plain, value_enc)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        for (it, plain, enc) in &rows {
            stmt.execute(params![
                id,
                it.scope.as_str(),
                it.name,
                it.kind.as_str(),
                it.present as i64,
                it.sensitive as i64,
                plain,
                enc
            ])?;
        }
        Ok(id)
    })
}

fn row_meta(r: &rusqlite::Row) -> rusqlite::Result<SnapshotMeta> {
    let scopes: Option<String> = r.get(9)?;
    Ok(SnapshotMeta {
        id: r.get(0)?,
        name: r.get(1)?,
        description: r.get(2)?,
        category: r.get(3)?,
        source: r.get(4)?,
        created_at: r.get(5)?,
        item_count: r.get(6)?,
        has_secrets: r.get::<_, i64>(7)? != 0,
        scopes: scopes
            .unwrap_or_default()
            .split(',')
            .filter_map(|s| parse_scope(s).ok())
            .collect(),
    })
}

const META_SQL: &str = "SELECT s.id, s.name, s.description, s.category, s.source, s.created_at, s.item_count,
        s.has_secrets, 0, (SELECT group_concat(DISTINCT scope) FROM snapshot_items WHERE snapshot_id = s.id)
     FROM snapshots s";

pub fn list(db: &Db) -> AppResult<Vec<SnapshotMeta>> {
    db.with(|c| {
        let mut stmt = c.prepare(&format!("{META_SQL} ORDER BY s.created_at DESC, s.id DESC"))?;
        let rows = stmt.query_map([], row_meta)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    })
}

pub fn get_meta(db: &Db, id: i64) -> AppResult<SnapshotMeta> {
    db.with(|c| {
        c.query_row(&format!("{META_SQL} WHERE s.id = ?1"), [id], row_meta)
            .optional()?
            .ok_or_else(|| AppError::NotFound(format!("snapshot {id}")))
    })
}

/// Veritabanındaki ham öğe satırı (değer henüz çözülmemiş).
struct RawItem {
    scope: String,
    name: String,
    kind: String,
    present: bool,
    sensitive: bool,
    plain: Option<String>,
    enc: Option<Vec<u8>>,
}

/// Öğeleri döndürür; hassas değerler çözülür. Çağıran, ön uca göndermeden önce maskelemelidir.
pub fn items(db: &Db, id: i64) -> AppResult<Vec<SnapshotItem>> {
    let raw: Vec<RawItem> = db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT scope, name, kind, present, sensitive, value_plain, value_enc FROM snapshot_items
             WHERE snapshot_id = ?1 ORDER BY scope, name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([id], |r| {
            Ok(RawItem {
                scope: r.get(0)?,
                name: r.get(1)?,
                kind: r.get(2)?,
                present: r.get::<_, i64>(3)? != 0,
                sensitive: r.get::<_, i64>(4)? != 0,
                plain: r.get(5)?,
                enc: r.get(6)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    })?;
    raw.into_iter()
        .map(|r| {
            let value = match (r.plain, r.enc) {
                (_, Some(e)) => Some(vault::unprotect_str(&e)?),
                (p, None) => p,
            };
            Ok(SnapshotItem {
                scope: parse_scope(&r.scope)
                    .map_err(|_| AppError::Database("geçersiz kapsam".into()))?,
                name: r.name,
                kind: ValueKind::parse(&r.kind).unwrap_or(ValueKind::String),
                present: r.present,
                sensitive: r.sensitive,
                value,
            })
        })
        .collect()
}

pub fn delete(db: &Db, id: i64) -> AppResult<()> {
    let n = db.with(|c| Ok(c.execute("DELETE FROM snapshots WHERE id = ?1", [id])?))?;
    if n == 0 {
        return Err(AppError::NotFound(format!("snapshot {id}")));
    }
    Ok(())
}

/// En yeni `keep` otomatik anlık görüntü dışındakileri siler.
pub fn prune_auto(db: &Db, keep: u32) -> AppResult<usize> {
    db.with(|c| {
        Ok(c.execute(
            "DELETE FROM snapshots WHERE category = 'auto' AND id NOT IN
               (SELECT id FROM snapshots WHERE category = 'auto' ORDER BY created_at DESC, id DESC LIMIT ?1)",
            [keep as i64],
        )?)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str, value: Option<&str>, sensitive: bool) -> SnapshotItem {
        SnapshotItem {
            scope: Scope::User,
            name: name.into(),
            kind: ValueKind::String,
            present: value.is_some(),
            sensitive,
            value: value.map(String::from),
        }
    }

    #[test]
    fn round_trip_with_encrypted_secret() {
        let db = Db::open_in_memory().unwrap();
        let items_in = vec![
            item("A", Some("1"), false),
            item("TOKEN", Some("s3cret-value"), true),
            item("GONE", None, false),
        ];
        let id = create(
            &db,
            NewSnapshot {
                name: "t",
                description: None,
                category: "manual",
                source: "environment",
            },
            &items_in,
        )
        .unwrap();

        // Ham tabloda gizli değer düz metin olarak bulunmamalı.
        let plain_hits: i64 = db
            .with(|c| {
                Ok(c.query_row(
                    "SELECT COUNT(*) FROM snapshot_items WHERE value_plain LIKE '%s3cret%' OR CAST(value_enc AS TEXT) LIKE '%s3cret%'",
                    [],
                    |r| r.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(plain_hits, 0);

        let out = items(&db, id).unwrap();
        assert_eq!(out.len(), 3);
        assert_eq!(
            out.iter()
                .find(|i| i.name == "TOKEN")
                .unwrap()
                .value
                .as_deref(),
            Some("s3cret-value")
        );
        assert!(!out.iter().find(|i| i.name == "GONE").unwrap().present);
        let meta = get_meta(&db, id).unwrap();
        assert!(meta.has_secrets);
        assert_eq!(meta.item_count, 3);
        assert_eq!(meta.scopes, vec![Scope::User]);
    }

    #[test]
    fn prune_keeps_newest_auto_and_all_manual() {
        let db = Db::open_in_memory().unwrap();
        for i in 0..5 {
            create(
                &db,
                NewSnapshot {
                    name: &format!("a{i}"),
                    description: None,
                    category: "auto",
                    source: "environment",
                },
                &[],
            )
            .unwrap();
        }
        create(
            &db,
            NewSnapshot {
                name: "m",
                description: None,
                category: "manual",
                source: "environment",
            },
            &[],
        )
        .unwrap();
        assert_eq!(prune_auto(&db, 2).unwrap(), 3);
        let names: Vec<String> = list(&db).unwrap().into_iter().map(|m| m.name).collect();
        assert_eq!(names, vec!["m", "a4", "a3"]);
    }

    #[test]
    fn delete_cascades() {
        let db = Db::open_in_memory().unwrap();
        let id = create(
            &db,
            NewSnapshot {
                name: "x",
                description: None,
                category: "manual",
                source: "environment",
            },
            &[item("A", Some("1"), false)],
        )
        .unwrap();
        delete(&db, id).unwrap();
        let n: i64 = db
            .with(|c| Ok(c.query_row("SELECT COUNT(*) FROM snapshot_items", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(n, 0);
        assert!(delete(&db, id).is_err());
    }
}
