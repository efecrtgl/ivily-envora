//! Profil deposu: CRUD, doğrulama, revizyonlar.

use super::model::*;
use crate::db::Db;
use crate::env::inspect::Overrides;
use crate::env::model::name_key;
use crate::env::validate::{validate_name, validate_value};
use crate::env::{Scope, ValueKind};
use crate::error::{AppError, AppResult};
use crate::pathmgr::{entries, PathAction, PathMod};
use crate::security::{classify, vault};
use rusqlite::{params, OptionalExtension, Transaction};
use std::collections::HashSet;

pub const MAX_NAME: usize = 100;
pub const MAX_TAGS: usize = 20;
pub const MAX_TAG_LEN: usize = 40;
pub const MAX_VARS: usize = 500;
pub const MAX_PATHS: usize = 200;
pub const MAX_PATH_ENTRY: usize = 1024;
/// Saklanan en fazla revizyon sayısı (profil başına).
pub const MAX_REVISIONS: i64 = 50;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
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

pub fn validate_profile_name(name: &str) -> AppResult<String> {
    let n = name.trim();
    if n.is_empty() {
        return Err(AppError::validation("invalid_profile_name", ""));
    }
    if n.chars().count() > MAX_NAME {
        return Err(AppError::validation(
            "invalid_profile_name",
            format!("en fazla {MAX_NAME} karakter"),
        ));
    }
    if n.chars().any(char::is_control) {
        return Err(AppError::validation("invalid_profile_name", ""));
    }
    Ok(n.to_string())
}

fn clean_tags(tags: &[String]) -> AppResult<Vec<String>> {
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

fn clean_paths(paths: &[ProfilePath]) -> AppResult<Vec<ProfilePath>> {
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
fn resolve_vars(
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

/// Şifrelenmiş satırlar (işlemden önce hazırlanır; DPAPI hatası veritabanını yarım bırakmaz).
struct VarRow {
    var: ProfileVar,
    plain: Option<String>,
    enc: Option<Vec<u8>>,
}

fn encrypt_vars(vars: Vec<ProfileVar>) -> AppResult<Vec<VarRow>> {
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

fn write_children(
    tx: &Transaction,
    id: i64,
    rows: &[VarRow],
    paths: &[ProfilePath],
) -> AppResult<()> {
    tx.execute("DELETE FROM profile_vars WHERE profile_id = ?1", [id])?;
    tx.execute("DELETE FROM profile_paths WHERE profile_id = ?1", [id])?;
    let mut st = tx.prepare(
        "INSERT INTO profile_vars(profile_id, position, name, value_plain, value_enc, sensitive, kind, scope, action)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )?;
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
    let mut st = tx.prepare("INSERT INTO profile_paths(profile_id, position, entry, action, scope) VALUES (?1, ?2, ?3, ?4, ?5)")?;
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

fn name_taken(db: &Db, name: &str, except: Option<i64>) -> AppResult<bool> {
    db.with(|c| {
        let id: Option<i64> = c
            .query_row(
                "SELECT id FROM profiles WHERE name = ?1 COLLATE NOCASE",
                [name],
                |r| r.get(0),
            )
            .optional()?;
        Ok(id.is_some_and(|id| Some(id) != except))
    })
}

pub fn create(db: &Db, input: &ProfileInput, overrides: &Overrides) -> AppResult<i64> {
    let name = validate_profile_name(&input.name)?;
    if name_taken(db, &name, None)? {
        return Err(AppError::Conflict(name));
    }
    let tags = clean_tags(&input.tags)?;
    let paths = clean_paths(&input.paths)?;
    let rows = encrypt_vars(resolve_vars(&input.vars, &[], overrides)?)?;
    let metadata = input
        .metadata
        .clone()
        .unwrap_or_else(|| serde_json::json!({}));
    let ts = now();
    db.tx(|tx| {
        tx.execute(
            "INSERT INTO profiles(name, description, category, tags, metadata, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                name,
                input.description.trim(),
                input.category.trim(),
                serde_json::to_string(&tags).unwrap_or_else(|_| "[]".into()),
                metadata.to_string(),
                ts
            ],
        )?;
        let id = tx.last_insert_rowid();
        write_children(tx, id, &rows, &paths)?;
        Ok(id)
    })
}

fn parse_status(s: &str) -> Status {
    if s == "archived" {
        Status::Archived
    } else {
        Status::Active
    }
}

/// Profili tüm değerleriyle (hassaslar çözülmüş) yükler. Arayüze gönderilmeden önce `masked` uygulanmalı.
pub fn load_full(db: &Db, id: i64) -> AppResult<Profile> {
    type Head = (
        String,
        String,
        String,
        String,
        String,
        i64,
        String,
        String,
        String,
    );
    let head: Option<Head> = db.with(|c| {
        Ok(c.query_row(
            "SELECT name, description, category, tags, status, version, metadata, created_at, updated_at
             FROM profiles WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?)),
        )
        .optional()?)
    })?;
    let (name, description, category, tags, status, version, metadata, created_at, updated_at) =
        head.ok_or_else(|| AppError::NotFound(format!("profile {id}")))?;

    struct RawVar {
        name: String,
        plain: Option<String>,
        enc: Option<Vec<u8>>,
        sensitive: bool,
        kind: Option<String>,
        scope: String,
        action: String,
    }
    let raw: Vec<RawVar> = db.with(|c| {
        let mut st = c.prepare(
            "SELECT name, value_plain, value_enc, sensitive, kind, scope, action FROM profile_vars
             WHERE profile_id = ?1 ORDER BY position",
        )?;
        let rows = st.query_map([id], |r| {
            Ok(RawVar {
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
    let mut vars = Vec::with_capacity(raw.len());
    for r in raw {
        let value = match (r.plain, r.enc) {
            (_, Some(e)) => Some(vault::unprotect_str(&e)?),
            (p, None) => p,
        };
        vars.push(ProfileVar {
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
        });
    }
    let paths: Vec<ProfilePath> = db.with(|c| {
        let mut st = c.prepare("SELECT entry, action, scope FROM profile_paths WHERE profile_id = ?1 ORDER BY position")?;
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
    })?;
    Ok(Profile {
        id,
        name,
        description,
        category,
        tags: serde_json::from_str(&tags).unwrap_or_default(),
        status: parse_status(&status),
        version,
        metadata: serde_json::from_str(&metadata).unwrap_or_else(|_| serde_json::json!({})),
        created_at,
        updated_at,
        vars,
        paths,
    })
}

/// Hassas değerleri kaldırır (arayüze gönderilecek biçim).
pub fn masked(mut p: Profile) -> Profile {
    for v in &mut p.vars {
        if v.sensitive {
            v.value = None;
        }
    }
    p
}

fn save_revision(tx: &Transaction, p: &Profile, note: Option<&str>) -> AppResult<()> {
    let json = serde_json::to_string(p).map_err(|e| AppError::Internal(e.to_string()))?;
    let has_secrets = p.vars.iter().any(|v| v.sensitive && v.value.is_some());
    let (plain, enc) = if has_secrets {
        (None, Some(vault::protect_str(&json)?))
    } else {
        (Some(json), None)
    };
    tx.execute(
        "INSERT INTO profile_revisions(profile_id, version, created_at, note, var_count, data_plain, data_enc)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![p.id, p.version, now(), note, p.vars.len() as i64, plain, enc],
    )?;
    tx.execute(
        "DELETE FROM profile_revisions WHERE profile_id = ?1 AND id NOT IN
           (SELECT id FROM profile_revisions WHERE profile_id = ?1 ORDER BY version DESC, id DESC LIMIT ?2)",
        params![p.id, MAX_REVISIONS],
    )?;
    Ok(())
}

/// Profili günceller; önceki hali revizyon olarak saklanır. Yeni sürüm numarasını döndürür.
pub fn update(
    db: &Db,
    id: i64,
    input: &ProfileInput,
    note: Option<&str>,
    overrides: &Overrides,
) -> AppResult<i64> {
    let current = load_full(db, id)?;
    let name = validate_profile_name(&input.name)?;
    if name_taken(db, &name, Some(id))? {
        return Err(AppError::Conflict(name));
    }
    let tags = clean_tags(&input.tags)?;
    let paths = clean_paths(&input.paths)?;
    let rows = encrypt_vars(resolve_vars(&input.vars, &current.vars, overrides)?)?;
    let metadata = input
        .metadata
        .clone()
        .unwrap_or_else(|| current.metadata.clone());
    let version = current.version + 1;
    db.tx(|tx| {
        save_revision(tx, &current, note)?;
        tx.execute(
            "UPDATE profiles SET name = ?2, description = ?3, category = ?4, tags = ?5, metadata = ?6,
             version = ?7, updated_at = ?8 WHERE id = ?1",
            params![
                id,
                name,
                input.description.trim(),
                input.category.trim(),
                serde_json::to_string(&tags).unwrap_or_else(|_| "[]".into()),
                metadata.to_string(),
                version,
                now()
            ],
        )?;
        write_children(tx, id, &rows, &paths)?;
        Ok(version)
    })
}

/// Profil girdisini tam profilden üretir (çoğaltma ve revizyon geri yükleme için).
fn input_from(p: &Profile, name: &str) -> ProfileInput {
    ProfileInput {
        name: name.to_string(),
        description: p.description.clone(),
        category: p.category.clone(),
        tags: p.tags.clone(),
        metadata: Some(p.metadata.clone()),
        vars: p
            .vars
            .iter()
            .map(|v| VarInput {
                name: v.name.clone(),
                value: v.value.clone(),
                keep: false,
                previous_name: None,
                sensitive: Some(v.sensitive),
                kind: v.kind,
                scope: v.scope,
                action: v.action,
            })
            .collect(),
        paths: p.paths.clone(),
    }
}

pub fn duplicate(db: &Db, id: i64, new_name: &str) -> AppResult<i64> {
    let p = load_full(db, id)?;
    create(db, &input_from(&p, new_name), &Overrides::new())
}

pub fn rename(db: &Db, id: i64, new_name: &str) -> AppResult<()> {
    let name = validate_profile_name(new_name)?;
    if name_taken(db, &name, Some(id))? {
        return Err(AppError::Conflict(name));
    }
    let n = db.with(|c| {
        Ok(c.execute(
            "UPDATE profiles SET name = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, name, now()],
        )?)
    })?;
    if n == 0 {
        return Err(AppError::NotFound(format!("profile {id}")));
    }
    Ok(())
}

pub fn set_status(db: &Db, id: i64, status: Status) -> AppResult<()> {
    let n = db.with(|c| {
        Ok(c.execute(
            "UPDATE profiles SET status = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, status.as_str(), now()],
        )?)
    })?;
    if n == 0 {
        return Err(AppError::NotFound(format!("profile {id}")));
    }
    Ok(())
}

pub fn delete(db: &Db, id: i64) -> AppResult<()> {
    let n = db.with(|c| Ok(c.execute("DELETE FROM profiles WHERE id = ?1", [id])?))?;
    if n == 0 {
        return Err(AppError::NotFound(format!("profile {id}")));
    }
    Ok(())
}

pub fn list(db: &Db, include_archived: bool) -> AppResult<Vec<ProfileSummary>> {
    db.with(|c| {
        let mut st = c.prepare(
            "SELECT p.id, p.name, p.description, p.category, p.tags, p.status, p.version, p.updated_at,
               (SELECT COUNT(*) FROM profile_vars v WHERE v.profile_id = p.id),
               (SELECT COUNT(*) FROM profile_paths x WHERE x.profile_id = p.id),
               (SELECT COUNT(*) FROM profile_vars v WHERE v.profile_id = p.id AND v.sensitive = 1),
               (SELECT group_concat(DISTINCT a.scope) FROM profile_activations a
                  WHERE a.profile_id = p.id AND a.deactivated_at IS NULL)
             FROM profiles p WHERE (?1 OR p.status = 'active') ORDER BY p.name COLLATE NOCASE",
        )?;
        let rows = st.query_map([include_archived], |r| {
            let tags: String = r.get(4)?;
            let status: String = r.get(5)?;
            let scopes: Option<String> = r.get(11)?;
            Ok(ProfileSummary {
                id: r.get(0)?,
                name: r.get(1)?,
                description: r.get(2)?,
                category: r.get(3)?,
                tags: serde_json::from_str(&tags).unwrap_or_default(),
                status: parse_status(&status),
                version: r.get(6)?,
                updated_at: r.get(7)?,
                var_count: r.get(8)?,
                path_count: r.get(9)?,
                has_secrets: r.get::<_, i64>(10)? > 0,
                active_scopes: scopes.unwrap_or_default().split(',').filter_map(Scope::parse).collect(),
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    })
}

pub fn revisions(db: &Db, id: i64) -> AppResult<Vec<Revision>> {
    db.with(|c| {
        let mut st = c.prepare(
            "SELECT id, version, created_at, note, var_count FROM profile_revisions WHERE profile_id = ?1
             ORDER BY version DESC, id DESC",
        )?;
        let rows = st.query_map([id], |r| {
            Ok(Revision { id: r.get(0)?, version: r.get(1)?, created_at: r.get(2)?, note: r.get(3)?, var_count: r.get(4)? })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    })
}

fn load_revision(db: &Db, id: i64, rev_id: i64) -> AppResult<Profile> {
    let (plain, enc): (Option<String>, Option<Vec<u8>>) = db.with(|c| {
        c.query_row(
            "SELECT data_plain, data_enc FROM profile_revisions WHERE id = ?1 AND profile_id = ?2",
            params![rev_id, id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("revision {rev_id}")))
    })?;
    let json = match (plain, enc) {
        (_, Some(e)) => vault::unprotect_str(&e)?,
        (Some(p), None) => p,
        (None, None) => return Err(AppError::Database("boş revizyon".into())),
    };
    Ok(serde_json::from_str(&json)?)
}

/// Revizyon içeriğini (hassaslar maskeli) döndürür.
pub fn revision_masked(db: &Db, id: i64, rev_id: i64) -> AppResult<Profile> {
    Ok(masked(load_revision(db, id, rev_id)?))
}

/// Bir revizyonu geri yükler. Geri yükleme de yeni bir sürüm oluşturur (mevcut hal revizyona girer).
pub fn restore_revision(db: &Db, id: i64, rev_id: i64) -> AppResult<i64> {
    let rev = load_revision(db, id, rev_id)?;
    let current = load_full(db, id)?;
    // Ad çakışırsa mevcut adı koru.
    let name = if name_taken(db, &rev.name, Some(id))? {
        current.name.clone()
    } else {
        rev.name.clone()
    };
    let note = format!("v{} geri yüklendi", rev.version);
    update(
        db,
        id,
        &input_from(&rev, &name),
        Some(&note),
        &Overrides::new(),
    )
}

pub fn reveal_var(db: &Db, id: i64, name: &str) -> AppResult<String> {
    load_full(db, id)?
        .vars
        .into_iter()
        .find(|v| name_key(&v.name) == name_key(name))
        .and_then(|v| v.value)
        .ok_or_else(|| AppError::NotFound(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn var(name: &str, value: &str) -> VarInput {
        VarInput {
            name: name.into(),
            value: Some(value.into()),
            keep: false,
            previous_name: None,
            sensitive: None,
            kind: None,
            scope: Scope::User,
            action: VarAction::Set,
        }
    }

    pub fn input(name: &str, vars: Vec<VarInput>) -> ProfileInput {
        ProfileInput {
            name: name.into(),
            description: "d".into(),
            category: "dev".into(),
            tags: vec!["node".into(), " Node ".into(), "".into()],
            metadata: None,
            vars,
            paths: vec![ProfilePath {
                path: PathMod {
                    entry: "C:\\node".into(),
                    action: PathAction::Prepend,
                },
                scope: Scope::User,
            }],
        }
    }

    #[test]
    fn create_load_and_mask() {
        let db = Db::open_in_memory().unwrap();
        let id = create(
            &db,
            &input(
                "Node",
                vec![
                    var("NODE_ENV", "development"),
                    var("NPM_TOKEN", "npm-secret-value"),
                ],
            ),
            &Overrides::new(),
        )
        .unwrap();
        let p = load_full(&db, id).unwrap();
        assert_eq!(p.tags, vec!["node"], "etiketler kırpılıp tekilleştirilmeli");
        assert_eq!(p.vars.len(), 2);
        assert!(p.vars[1].sensitive);
        assert_eq!(p.vars[1].value.as_deref(), Some("npm-secret-value"));
        let m = masked(p);
        assert!(m.vars[1].value.is_none());
        assert!(!serde_json::to_string(&m).unwrap().contains("npm-secret"));
        // Veritabanında düz metin yok.
        let hits: i64 = db
            .with(|c| {
                Ok(c.query_row(
                    "SELECT COUNT(*) FROM profile_vars WHERE value_plain LIKE '%npm-secret%'",
                    [],
                    |r| r.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(hits, 0);
        let s = &list(&db, false).unwrap()[0];
        assert!(s.has_secrets && s.var_count == 2 && s.path_count == 1);
    }

    #[test]
    fn validation_rules() {
        let db = Db::open_in_memory().unwrap();
        let o = Overrides::new();
        assert!(create(&db, &input("  ", vec![]), &o).is_err());
        assert!(
            create(&db, &input("A", vec![var("X", "1"), var("x", "2")]), &o).is_err(),
            "aynı değişken iki kez"
        );
        assert!(create(&db, &input("A", vec![var("A=B", "1")]), &o).is_err());
        let mut bad_path = input("A", vec![]);
        bad_path.paths[0].path.entry = "C:\\a;C:\\b".into();
        assert!(
            create(&db, &bad_path, &o).is_err(),
            "PATH girdisi ';' içeremez"
        );
        create(&db, &input("Same", vec![]), &o).unwrap();
        assert!(matches!(
            create(&db, &input("same", vec![]), &o),
            Err(AppError::Conflict(_))
        ));
        let mut missing = var("Y", "");
        missing.value = None;
        assert!(create(&db, &input("B", vec![missing]), &o).is_err());
    }

    #[test]
    fn update_keeps_secret_without_frontend_value_and_records_revision() {
        let db = Db::open_in_memory().unwrap();
        let o = Overrides::new();
        let id = create(&db, &input("P", vec![var("API_TOKEN", "orig-secret")]), &o).unwrap();
        let mut keep = var("API_TOKEN_RENAMED", "");
        keep.value = None;
        keep.keep = true;
        keep.previous_name = Some("API_TOKEN".into());
        let v2 = update(
            &db,
            id,
            &input("P", vec![keep, var("EXTRA", "1")]),
            Some("ekleme"),
            &o,
        )
        .unwrap();
        assert_eq!(v2, 2);
        assert_eq!(
            reveal_var(&db, id, "api_token_renamed").unwrap(),
            "orig-secret"
        );
        let revs = revisions(&db, id).unwrap();
        assert_eq!(revs.len(), 1);
        assert_eq!(revs[0].version, 1);

        // Revizyon geri yükleme: eski hale döner, yeni sürüm üretir.
        let v3 = restore_revision(&db, id, revs[0].id).unwrap();
        assert_eq!(v3, 3);
        let p = load_full(&db, id).unwrap();
        assert_eq!(p.vars.len(), 1);
        assert_eq!(p.vars[0].name, "API_TOKEN");
        assert_eq!(p.vars[0].value.as_deref(), Some("orig-secret"));
        // Gizli değer içeren revizyon şifreli saklanır.
        let plain: i64 = db
            .with(|c| {
                Ok(c.query_row(
                    "SELECT COUNT(*) FROM profile_revisions WHERE data_plain LIKE '%orig-secret%'",
                    [],
                    |r| r.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(plain, 0);
    }

    #[test]
    fn duplicate_rename_archive_delete() {
        let db = Db::open_in_memory().unwrap();
        let o = Overrides::new();
        let id = create(&db, &input("Orig", vec![var("SECRET_X", "s")]), &o).unwrap();
        let copy = duplicate(&db, id, "Copy").unwrap();
        assert_eq!(reveal_var(&db, copy, "SECRET_X").unwrap(), "s");
        assert!(rename(&db, copy, "orig").is_err());
        rename(&db, copy, "Renamed").unwrap();
        set_status(&db, id, Status::Archived).unwrap();
        assert_eq!(list(&db, false).unwrap().len(), 1);
        assert_eq!(list(&db, true).unwrap().len(), 2);
        delete(&db, id).unwrap();
        assert!(load_full(&db, id).is_err());
    }
}
