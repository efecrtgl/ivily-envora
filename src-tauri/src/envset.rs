//! Ortam kümesi depolaması: profiller ve projeler için ortak değişken/PATH saklama mantığı.
//!
//! Her iki sahip türü de aynı sütun yapısına sahip ayrı tablolar kullanır
//! (`profile_vars`/`profile_paths`, `project_vars`/`project_paths`). Tablo adları sabittir;
//! kullanıcı girdisi SQL metnine hiçbir zaman eklenmez.

use crate::db::Db;
use crate::env::inspect::Overrides;
use crate::env::model::name_key;
use crate::env::validate::{validate_name, validate_value};
use crate::env::{Scope, ValueKind};
use crate::error::{AppError, AppResult};
use crate::pathmgr::{entries, PathAction, PathMod};
use crate::profiles::model::{ProfilePath, ProfileVar, VarAction, VarInput};
use crate::security::{classify, vault};
use rusqlite::{params, Transaction};
use std::collections::HashSet;

pub const MAX_TAGS: usize = 20;
pub const MAX_TAG_LEN: usize = 40;
pub const MAX_VARS: usize = 500;
pub const MAX_PATHS: usize = 200;
pub const MAX_PATH_ENTRY: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    Profile,
    Project,
}

impl Owner {
    fn vars_table(self) -> &'static str {
        match self {
            Owner::Profile => "profile_vars",
            Owner::Project => "project_vars",
        }
    }
    fn paths_table(self) -> &'static str {
        match self {
            Owner::Profile => "profile_paths",
            Owner::Project => "project_paths",
        }
    }
    fn owner_col(self) -> &'static str {
        match self {
            Owner::Profile => "profile_id",
            Owner::Project => "project_id",
        }
    }
}

fn action_str(a: VarAction) -> &'static str {
    match a {
        VarAction::Set => "set",
        VarAction::Unset => "unset",
    }
}

fn path_action_str(a: PathAction) -> &'static str {
    match a {
        PathAction::Prepend => "prepend",
        PathAction::Append => "append",
        PathAction::Remove => "remove",
    }
}

pub fn clean_tags(tags: &[String]) -> AppResult<Vec<String>> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for t in tags {
        let t = t.trim();
        if t.is_empty() {
            continue;
        }
        if t.chars().count() > MAX_TAG_LEN || t.chars().any(char::is_control) {
            return Err(AppError::validation(
                "invalid_tag",
                t.chars().take(MAX_TAG_LEN).collect::<String>(),
            ));
        }
        if seen.insert(t.to_lowercase()) {
            out.push(t.to_string());
        }
    }
    if out.len() > MAX_TAGS {
        return Err(AppError::validation(
            "too_many_tags",
            format!("en fazla {MAX_TAGS}"),
        ));
    }
    Ok(out)
}

pub fn clean_paths(paths: &[ProfilePath]) -> AppResult<Vec<ProfilePath>> {
    if paths.len() > MAX_PATHS {
        return Err(AppError::validation(
            "too_many_paths",
            format!("en fazla {MAX_PATHS}"),
        ));
    }
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for p in paths {
        let e = p.path.entry.trim();
        if e.is_empty() {
            return Err(AppError::validation("invalid_path_entry", ""));
        }
        if e.contains(';') || e.contains('\0') || e.chars().count() > MAX_PATH_ENTRY {
            return Err(AppError::validation(
                "invalid_path_entry",
                e.chars().take(80).collect::<String>(),
            ));
        }
        if !seen.insert((p.scope, entries::key(e))) {
            return Err(AppError::validation("duplicate_path_entry", e.to_string()));
        }
        out.push(ProfilePath {
            path: PathMod {
                entry: e.to_string(),
                action: p.path.action,
            },
            scope: p.scope,
        });
    }
    Ok(out)
}

/// Girdiyi doğrular ve tam (değerleri çözülmüş) değişken listesine dönüştürür.
/// `existing`, güncellemede `keep` ile korunan değerlerin kaynağıdır.
pub fn resolve_vars(
    input: &[VarInput],
    existing: &[ProfileVar],
    overrides: &Overrides,
) -> AppResult<Vec<ProfileVar>> {
    if input.len() > MAX_VARS {
        return Err(AppError::validation(
            "too_many_vars",
            format!("en fazla {MAX_VARS}"),
        ));
    }
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(input.len());
    for v in input {
        let name = v.name.trim().to_string();
        validate_name(&name)?;
        if !seen.insert(name_key(&name)) {
            return Err(AppError::validation("duplicate_var", name));
        }
        if v.kind == Some(ValueKind::Other) {
            return Err(AppError::validation("invalid_kind", name));
        }
        let value = match v.action {
            VarAction::Unset => None,
            VarAction::Set => {
                if v.keep {
                    let source = v.previous_name.as_deref().unwrap_or(&name);
                    let prev = existing
                        .iter()
                        .find(|e| name_key(&e.name) == name_key(source))
                        .and_then(|e| e.value.clone())
                        .ok_or_else(|| AppError::validation("missing_value", name.clone()))?;
                    Some(prev)
                } else {
                    let val = v
                        .value
                        .clone()
                        .ok_or_else(|| AppError::validation("missing_value", name.clone()))?;
                    validate_value(&val)?;
                    Some(val)
                }
            }
        };
        let sensitive = v.sensitive.unwrap_or_else(|| {
            classify::resolve(
                &name,
                value.as_deref().unwrap_or(""),
                overrides.get(&name_key(&name)).copied(),
            )
            .sensitive
        });
        out.push(ProfileVar {
            name,
            value,
            sensitive,
            kind: v.kind,
            scope: v.scope,
            action: v.action,
        });
    }
    Ok(out)
}

/// Şifrelenmiş satır (işlemden önce hazırlanır; DPAPI hatası veritabanını yarım bırakmaz).
pub struct VarRow {
    var: ProfileVar,
    plain: Option<String>,
    enc: Option<Vec<u8>>,
}

pub fn encrypt_vars(vars: Vec<ProfileVar>) -> AppResult<Vec<VarRow>> {
    vars.into_iter()
        .map(|v| {
            let (plain, enc) = match (&v.value, v.sensitive) {
                (Some(val), true) => (None, Some(vault::protect_str(val)?)),
                (Some(val), false) => (Some(val.clone()), None),
                (None, _) => (None, None),
            };
            Ok(VarRow { var: v, plain, enc })
        })
        .collect()
}

/// Sahibin değişken ve PATH satırlarını tamamen değiştirir.
pub fn write(
    tx: &Transaction,
    owner: Owner,
    id: i64,
    rows: &[VarRow],
    paths: &[ProfilePath],
) -> AppResult<()> {
    let (vt, pt, col) = (owner.vars_table(), owner.paths_table(), owner.owner_col());
    tx.execute(&format!("DELETE FROM {vt} WHERE {col} = ?1"), [id])?;
    tx.execute(&format!("DELETE FROM {pt} WHERE {col} = ?1"), [id])?;
    let mut st = tx.prepare(&format!(
        "INSERT INTO {vt}({col}, position, name, value_plain, value_enc, sensitive, kind, scope, action)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
    ))?;
    for (i, r) in rows.iter().enumerate() {
        st.execute(params![
            id,
            i as i64,
            r.var.name,
            r.plain,
            r.enc,
            r.var.sensitive as i64,
            r.var.kind.map(|k| k.as_str()),
            r.var.scope.as_str(),
            action_str(r.var.action)
        ])?;
    }
    let mut st = tx.prepare(&format!(
        "INSERT INTO {pt}({col}, position, entry, action, scope) VALUES (?1, ?2, ?3, ?4, ?5)"
    ))?;
    for (i, p) in paths.iter().enumerate() {
        st.execute(params![
            id,
            i as i64,
            p.path.entry,
            path_action_str(p.path.action),
            p.scope.as_str()
        ])?;
    }
    Ok(())
}

/// Değişkenleri hassas değerler çözülmüş olarak okur.
pub fn read_vars(db: &Db, owner: Owner, id: i64) -> AppResult<Vec<ProfileVar>> {
    struct Raw {
        name: String,
        plain: Option<String>,
        enc: Option<Vec<u8>>,
        sensitive: bool,
        kind: Option<String>,
        scope: String,
        action: String,
    }
    let (vt, col) = (owner.vars_table(), owner.owner_col());
    let raw: Vec<Raw> = db.with(|c| {
        let mut st = c.prepare(&format!(
            "SELECT name, value_plain, value_enc, sensitive, kind, scope, action FROM {vt} WHERE {col} = ?1 ORDER BY position"
        ))?;
        let rows = st.query_map([id], |r| {
            Ok(Raw {
                name: r.get(0)?,
                plain: r.get(1)?,
                enc: r.get(2)?,
                sensitive: r.get::<_, i64>(3)? != 0,
                kind: r.get(4)?,
                scope: r.get(5)?,
                action: r.get(6)?,
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
            Ok(ProfileVar {
                name: r.name,
                value,
                sensitive: r.sensitive,
                kind: r.kind.as_deref().and_then(ValueKind::parse),
                scope: Scope::parse(&r.scope).unwrap_or(Scope::User),
                action: if r.action == "unset" {
                    VarAction::Unset
                } else {
                    VarAction::Set
                },
            })
        })
        .collect()
}

pub fn read_paths(db: &Db, owner: Owner, id: i64) -> AppResult<Vec<ProfilePath>> {
    let (pt, col) = (owner.paths_table(), owner.owner_col());
    db.with(|c| {
        let mut st = c.prepare(&format!(
            "SELECT entry, action, scope FROM {pt} WHERE {col} = ?1 ORDER BY position"
        ))?;
        let rows = st.query_map([id], |r| {
            let action: String = r.get(1)?;
            let scope: String = r.get(2)?;
            Ok(ProfilePath {
                path: PathMod {
                    entry: r.get(0)?,
                    action: match action.as_str() {
                        "prepend" => PathAction::Prepend,
                        "remove" => PathAction::Remove,
                        _ => PathAction::Append,
                    },
                },
                scope: Scope::parse(&scope).unwrap_or(Scope::User),
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    })
}

/// Hassas değerleri kaldırır (arayüze gönderilecek biçim).
pub fn mask_vars(vars: &mut [ProfileVar]) {
    for v in vars {
        if v.sensitive {
            v.value = None;
        }
    }
}
