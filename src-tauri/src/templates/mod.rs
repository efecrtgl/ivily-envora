//! Ortam şablonları: yerleşik öneriler ve kullanıcı şablonları.
//!
//! Şablonlar gerçek gizli değer içeremez ve hiçbir komut çalıştırmaz. Bir şablonu uygulamak,
//! kullanıcının gözden geçirip kaydedeceği bir profil/proje taslağı oluşturmaktan ibarettir.

pub mod builtin;
pub mod model;

use crate::db::Db;
use crate::env::model::name_key;
use crate::env::validate::{validate_name, validate_value};
use crate::envset;
use crate::error::{AppError, AppResult};
use crate::pathmgr::PathMod;
use crate::profiles::model::ProfilePath;
use crate::security::classify;
use model::*;
use rusqlite::{params, OptionalExtension};
use std::collections::HashSet;

const MAX_VARS: usize = 200;

fn localized(b: &builtin::Builtin, en: bool) -> Template {
    let pick = |pair: (&'static str, &'static str)| if en { pair.1 } else { pair.0 }.to_string();
    Template {
        id: format!("builtin:{}", b.id),
        builtin: true,
        name: pick(b.name),
        description: pick(b.description),
        category: b.category.to_string(),
        vars: b
            .vars
            .iter()
            .map(|v| TemplateVar {
                name: v.name.into(),
                value: v.value.into(),
                kind: None,
                scope: v.scope,
                role: v.role,
                note: pick(v.note),
            })
            .collect(),
        paths: b
            .paths
            .iter()
            .map(|p| ProfilePath {
                path: PathMod {
                    entry: p.entry.into(),
                    action: p.action,
                },
                scope: p.scope,
            })
            .collect(),
        updated_at: None,
    }
}

/// Şablonların gizli değer içermediğini ve geçerli olduğunu doğrular.
pub fn validate(input: &TemplateInput) -> AppResult<TemplateInput> {
    let name = input.name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err(AppError::validation("invalid_template_name", ""));
    }
    if input.vars.len() > MAX_VARS {
        return Err(AppError::validation("too_many_vars", ""));
    }
    let mut seen = HashSet::new();
    for v in &input.vars {
        validate_name(&v.name)?;
        validate_value(&v.value)?;
        if !seen.insert(name_key(&v.name)) {
            return Err(AppError::validation("duplicate_var", v.name.clone()));
        }
        // Gizli görünen bir değer veya hassas addaki dolu değer kabul edilmez.
        if classify::value_reason(&v.value).is_some()
            || (classify::name_reason(&v.name).is_some() && !v.value.trim().is_empty())
        {
            return Err(AppError::validation("template_secret", v.name.clone()));
        }
    }
    Ok(TemplateInput {
        name: name.to_string(),
        description: input.description.trim().to_string(),
        category: input.category.trim().to_string(),
        vars: input.vars.clone(),
        paths: envset::clean_paths(&input.paths)?,
    })
}

pub fn list(db: &Db, en: bool) -> AppResult<Vec<Template>> {
    let mut out: Vec<Template> = builtin::BUILTINS.iter().map(|b| localized(b, en)).collect();
    out.extend(user_templates(db)?);
    Ok(out)
}

fn user_templates(db: &Db) -> AppResult<Vec<Template>> {
    db.with(|c| {
        let mut st = c.prepare("SELECT id, name, description, category, data, updated_at FROM templates ORDER BY name COLLATE NOCASE")?;
        let rows = st.query_map([], |r| {
            let data: String = r.get(4)?;
            let (vars, paths): (Vec<TemplateVar>, Vec<ProfilePath>) = serde_json::from_str(&data).unwrap_or_default();
            Ok(Template {
                id: format!("user:{}", r.get::<_, i64>(0)?),
                builtin: false,
                name: r.get(1)?,
                description: r.get(2)?,
                category: r.get(3)?,
                vars,
                paths,
                updated_at: r.get(5)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    })
}

pub fn get(db: &Db, id: &str, en: bool) -> AppResult<Template> {
    if let Some(b) = id.strip_prefix("builtin:") {
        return builtin::BUILTINS
            .iter()
            .find(|x| x.id == b)
            .map(|x| localized(x, en))
            .ok_or_else(|| AppError::NotFound(id.into()));
    }
    let n: i64 = id
        .strip_prefix("user:")
        .and_then(|x| x.parse().ok())
        .ok_or_else(|| AppError::NotFound(id.into()))?;
    user_templates(db)?
        .into_iter()
        .find(|t| t.id == format!("user:{n}"))
        .ok_or_else(|| AppError::NotFound(id.into()))
}

fn user_id(id: &str) -> AppResult<i64> {
    id.strip_prefix("user:")
        .and_then(|x| x.parse().ok())
        .ok_or_else(|| AppError::validation("builtin_readonly", ""))
}

pub fn create(db: &Db, input: &TemplateInput) -> AppResult<String> {
    let t = validate(input)?;
    let data = serde_json::to_string(&(&t.vars, &t.paths))
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let now = chrono::Utc::now().to_rfc3339();
    db.with(|c| {
        let exists: Option<i64> = c.query_row("SELECT id FROM templates WHERE name = ?1 COLLATE NOCASE", [&t.name], |r| r.get(0)).optional()?;
        if exists.is_some() {
            return Err(AppError::Conflict(t.name.clone()));
        }
        c.execute(
            "INSERT INTO templates(name, description, category, data, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            params![t.name, t.description, t.category, data, now],
        )?;
        Ok(format!("user:{}", c.last_insert_rowid()))
    })
}

pub fn update(db: &Db, id: &str, input: &TemplateInput) -> AppResult<()> {
    let n = user_id(id)?;
    let t = validate(input)?;
    let data = serde_json::to_string(&(&t.vars, &t.paths))
        .map_err(|e| AppError::Internal(e.to_string()))?;
    db.with(|c| {
        let other: Option<i64> = c.query_row("SELECT id FROM templates WHERE name = ?1 COLLATE NOCASE", [&t.name], |r| r.get(0)).optional()?;
        if other.is_some_and(|o| o != n) {
            return Err(AppError::Conflict(t.name.clone()));
        }
        let changed = c.execute(
            "UPDATE templates SET name = ?2, description = ?3, category = ?4, data = ?5, updated_at = ?6 WHERE id = ?1",
            params![n, t.name, t.description, t.category, data, chrono::Utc::now().to_rfc3339()],
        )?;
        if changed == 0 {
            return Err(AppError::NotFound(id.into()));
        }
        Ok(())
    })
}

pub fn delete(db: &Db, id: &str) -> AppResult<()> {
    let n = user_id(id)?;
    let changed = db.with(|c| Ok(c.execute("DELETE FROM templates WHERE id = ?1", [n])?))?;
    if changed == 0 {
        return Err(AppError::NotFound(id.into()));
    }
    Ok(())
}

/// Yerleşik veya kullanıcı şablonunu yeni bir kullanıcı şablonu olarak kopyalar.
pub fn duplicate(db: &Db, id: &str, name: &str, en: bool) -> AppResult<String> {
    let t = get(db, id, en)?;
    create(
        db,
        &TemplateInput {
            name: name.into(),
            description: t.description,
            category: t.category,
            vars: t.vars,
            paths: t.paths,
        },
    )
}

pub fn input_of(t: &Template) -> TemplateInput {
    TemplateInput {
        name: t.name.clone(),
        description: t.description.clone(),
        category: t.category.clone(),
        vars: t.vars.clone(),
        paths: t.paths.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::Scope;

    fn tv(name: &str, value: &str) -> TemplateVar {
        TemplateVar {
            name: name.into(),
            value: value.into(),
            kind: None,
            scope: Scope::User,
            role: Role::Optional,
            note: String::new(),
        }
    }

    #[test]
    fn builtins_are_valid_and_secret_free() {
        for b in builtin::BUILTINS {
            for en in [false, true] {
                let t = localized(b, en);
                assert!(validate(&input_of(&t)).is_ok(), "{} geçersiz", b.id);
                assert!(!t.name.is_empty() && !t.description.is_empty());
            }
        }
        let ids: HashSet<_> = builtin::BUILTINS.iter().map(|b| b.id).collect();
        assert_eq!(ids.len(), builtin::BUILTINS.len());
    }

    #[test]
    fn rejects_secrets_in_templates() {
        let mut i = TemplateInput {
            name: "T".into(),
            description: String::new(),
            category: String::new(),
            vars: vec![tv("API_TOKEN", "abc")],
            paths: vec![],
        };
        assert!(matches!(
            validate(&i),
            Err(AppError::Validation {
                code: "template_secret",
                ..
            })
        ));
        i.vars = vec![tv("API_TOKEN", "")];
        assert!(
            validate(&i).is_ok(),
            "hassas ad boş değerle (yer tutucu) kabul edilir"
        );
        i.vars = vec![tv("URL", "postgres://u:pw@h/db")];
        assert!(
            validate(&i).is_err(),
            "parola içeren bağlantı dizesi reddedilir"
        );
    }

    #[test]
    fn user_template_crud_and_builtin_protection() {
        let db = Db::open_in_memory().unwrap();
        let id = duplicate(&db, "builtin:node", "Benim Node", false).unwrap();
        let t = get(&db, &id, false).unwrap();
        assert!(!t.builtin && t.vars.iter().any(|v| v.name == "NODE_ENV"));
        let mut input = input_of(&t);
        input.vars.push(tv("EXTRA", "1"));
        update(&db, &id, &input).unwrap();
        assert_eq!(get(&db, &id, false).unwrap().vars.len(), t.vars.len() + 1);
        assert!(create(&db, &input).is_err(), "aynı ad");
        assert!(delete(&db, "builtin:node").is_err());
        assert!(update(&db, "builtin:node", &input).is_err());
        delete(&db, &id).unwrap();
        assert_eq!(list(&db, false).unwrap().len(), builtin::BUILTINS.len());
        assert_eq!(
            get(&db, "builtin:java", true).unwrap().name,
            "Java development"
        );
    }
}
