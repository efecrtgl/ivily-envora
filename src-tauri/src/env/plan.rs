//! Değişiklik planları: doğrulama, çözümleme ve önizleme.
//!
//! Kalıcı ortamdaki her değişiklik tek bir kapsama ait bir `ChangePlan` ile ifade edilir.
//! Başka bir kapsamdaki değişken hiçbir zaman sessizce değiştirilmez.

use super::inspect::{is_merged_name, Overrides};
use super::model::{name_key, Scope, ValueKind};
use super::provider::EnvProvider;
use super::validate::{has_percent_reference, lint, validate_name, validate_value, Issue};
use crate::error::{AppError, AppResult};
use crate::security::classify;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Op {
    /// Değeri ayarlar. `value: None` mevcut değeri korur (yalnızca türü değiştirir);
    /// `kind: None` mevcut türü korur, yeni değişkende `%AD%` içeriğine göre seçilir.
    #[serde(rename_all = "camelCase")]
    Set {
        name: String,
        value: Option<String>,
        kind: Option<ValueKind>,
    },
    #[serde(rename_all = "camelCase")]
    Delete { name: String },
    /// Değeri başka bir kapsamdan/addan kopyalar. Değer ön uca hiç gönderilmeden çoğaltma ve
    /// yeniden adlandırma yapılabilmesini sağlar.
    #[serde(rename_all = "camelCase")]
    Copy {
        name: String,
        source_scope: Scope,
        source_name: String,
    },
}

impl Op {
    pub fn target(&self) -> &str {
        match self {
            Op::Set { name, .. } | Op::Delete { name } | Op::Copy { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePlan {
    pub scope: Scope,
    pub ops: Vec<Op>,
}

/// Değerleri çözümlenmiş, uygulanmaya hazır işlem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Resolved {
    Set {
        name: String,
        value: String,
        kind: ValueKind,
    },
    Delete {
        name: String,
    },
}

impl Resolved {
    pub fn name(&self) -> &str {
        match self {
            Resolved::Set { name, .. } | Resolved::Delete { name } => name,
        }
    }
}

pub const MAX_OPS: usize = 500;

/// Sistem kapsamında değiştirilmesi Windows'u etkileyebilecek adlar.
const CRITICAL: &[&str] = &[
    "PATH",
    "PATHEXT",
    "SYSTEMROOT",
    "WINDIR",
    "COMSPEC",
    "OS",
    "TEMP",
    "TMP",
    "PSMODULEPATH",
    "NUMBER_OF_PROCESSORS",
    "DRIVERDATA",
    "PROCESSOR_ARCHITECTURE",
];

pub fn is_critical(name: &str) -> bool {
    CRITICAL.contains(&name_key(name).as_str())
}

fn infer_kind(value: &str) -> ValueKind {
    if has_percent_reference(value) {
        ValueKind::Expand
    } else {
        ValueKind::String
    }
}

/// Planın yapısını doğrular (değerleri okumadan).
pub fn validate(plan: &ChangePlan) -> AppResult<()> {
    if plan.scope == Scope::Process {
        return Err(AppError::validation("invalid_scope_process", ""));
    }
    if plan.ops.is_empty() {
        return Err(AppError::validation("empty_plan", ""));
    }
    if plan.ops.len() > MAX_OPS {
        return Err(AppError::validation(
            "plan_too_large",
            format!("en fazla {MAX_OPS}"),
        ));
    }
    let mut seen = HashSet::new();
    for op in &plan.ops {
        validate_name(op.target())?;
        if !seen.insert(name_key(op.target())) {
            return Err(AppError::validation(
                "duplicate_in_plan",
                op.target().to_string(),
            ));
        }
        match op {
            Op::Set {
                value: Some(v),
                kind,
                ..
            } => {
                validate_value(v)?;
                if *kind == Some(ValueKind::Other) {
                    return Err(AppError::validation("invalid_kind", ""));
                }
            }
            Op::Set {
                kind: Some(ValueKind::Other),
                ..
            } => return Err(AppError::validation("invalid_kind", "")),
            Op::Copy { source_name, .. } => validate_name(source_name)?,
            _ => {}
        }
    }
    Ok(())
}

/// İşlemleri mevcut ortama göre çözümler.
pub fn resolve(p: &dyn EnvProvider, plan: &ChangePlan) -> AppResult<Vec<Resolved>> {
    validate(plan)?;
    let current: HashMap<String, (String, ValueKind)> = p
        .list(plan.scope)?
        .into_iter()
        .map(|v| (name_key(&v.name), (v.value, v.kind)))
        .collect();
    let mut out = Vec::with_capacity(plan.ops.len());
    for op in &plan.ops {
        out.push(match op {
            Op::Delete { name } => Resolved::Delete { name: name.clone() },
            Op::Set { name, value, kind } => {
                let existing = current.get(&name_key(name));
                let value = match (value, existing) {
                    (Some(v), _) => v.clone(),
                    (None, Some((v, _))) => v.clone(),
                    (None, None) => return Err(AppError::NotFound(name.clone())),
                };
                if let Some((_, ValueKind::Other)) = existing {
                    return Err(AppError::validation("invalid_kind", name.clone()));
                }
                let kind = kind
                    .or(existing.map(|(_, k)| *k))
                    .unwrap_or_else(|| infer_kind(&value));
                Resolved::Set {
                    name: name.clone(),
                    value,
                    kind,
                }
            }
            Op::Copy {
                name,
                source_scope,
                source_name,
            } => {
                let src = p
                    .get(*source_scope, source_name)?
                    .ok_or_else(|| AppError::NotFound(source_name.clone()))?;
                if src.kind == ValueKind::Other {
                    return Err(AppError::validation("invalid_kind", source_name.clone()));
                }
                let kind = if *source_scope == Scope::Process {
                    infer_kind(&src.value)
                } else {
                    src.kind
                };
                Resolved::Set {
                    name: name.clone(),
                    value: src.value,
                    kind,
                }
            }
        });
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Create,
    Update,
    Delete,
    Unchanged,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewItem {
    pub name: String,
    pub action: Action,
    pub kind_before: Option<ValueKind>,
    pub kind_after: Option<ValueKind>,
    /// Hassas değişkenlerde `None` (maskeli).
    pub before: Option<String>,
    pub after: Option<String>,
    pub sensitive: bool,
    pub issues: Vec<Issue>,
    /// Kapsam uyarıları: `shadowedByUser`, `shadowsMachine`, `critical`, `pathDelete`, `missing`
    pub warnings: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub scope: Scope,
    pub items: Vec<PreviewItem>,
    /// Değişecek öğe sayısı (değişmeyenler hariç).
    pub changes: usize,
    /// Bu süreç sistem kapsamına yazamıyorsa yükseltme (UAC) gerekir.
    pub requires_elevation: bool,
}

pub fn preview(
    p: &dyn EnvProvider,
    plan: &ChangePlan,
    overrides: &Overrides,
) -> AppResult<Preview> {
    let resolved = resolve(p, plan)?;
    let current: HashMap<String, (String, ValueKind)> = p
        .list(plan.scope)?
        .into_iter()
        .map(|v| (name_key(&v.name), (v.value, v.kind)))
        .collect();
    let other_scope = if plan.scope == Scope::User {
        Scope::Machine
    } else {
        Scope::User
    };
    let other: HashSet<String> = p
        .list(other_scope)?
        .into_iter()
        .map(|v| name_key(&v.name))
        .collect();

    let mut items = Vec::new();
    for r in &resolved {
        let key = name_key(r.name());
        let before = current.get(&key);
        let ov = overrides.get(&key).copied();
        let sens = |v: &str| classify::resolve(r.name(), v, ov).sensitive;
        let mut warnings = Vec::new();
        let (action, kind_after, after, issues) = match r {
            Resolved::Delete { .. } => {
                if before.is_none() {
                    warnings.push("missing");
                }
                if is_merged_name(r.name()) {
                    warnings.push("pathDelete");
                }
                (
                    if before.is_some() {
                        Action::Delete
                    } else {
                        Action::Unchanged
                    },
                    None,
                    None,
                    Vec::new(),
                )
            }
            Resolved::Set { value, kind, .. } => {
                let action = match before {
                    None => Action::Create,
                    Some((v, k)) if v == value && k == kind => Action::Unchanged,
                    Some(_) => Action::Update,
                };
                (
                    action,
                    Some(*kind),
                    Some(value.clone()),
                    lint(r.name(), value, *kind),
                )
            }
        };
        if action != Action::Unchanged && !is_merged_name(r.name()) && other.contains(&key) {
            warnings.push(if plan.scope == Scope::Machine {
                "shadowedByUser"
            } else {
                "shadowsMachine"
            });
        }
        if plan.scope == Scope::Machine && is_critical(r.name()) && action != Action::Unchanged {
            warnings.push("critical");
        }
        let sensitive = before.is_some_and(|(v, _)| sens(v)) || after.as_deref().is_some_and(sens);
        items.push(PreviewItem {
            name: before
                .and_then(|_| p.get(plan.scope, r.name()).ok().flatten().map(|v| v.name))
                .unwrap_or_else(|| r.name().to_string()),
            action,
            kind_before: before.map(|(_, k)| *k),
            kind_after,
            before: if sensitive {
                None
            } else {
                before.map(|(v, _)| v.clone())
            },
            after: if sensitive { None } else { after },
            sensitive,
            issues,
            warnings,
        });
    }
    let changes = items
        .iter()
        .filter(|i| i.action != Action::Unchanged)
        .count();
    Ok(Preview {
        scope: plan.scope,
        items,
        changes,
        requires_elevation: plan.scope == Scope::Machine && !p.can_write_machine(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::provider::MemoryProvider;

    fn provider() -> MemoryProvider {
        let p = MemoryProvider::new();
        p.seed(Scope::User, "EDITOR", "code", ValueKind::String);
        p.seed(Scope::User, "API_TOKEN", "tok-123456", ValueKind::String);
        p.seed(Scope::Machine, "JAVA_HOME", "C:\\jdk17", ValueKind::String);
        p.seed(Scope::Machine, "Path", "C:\\Windows", ValueKind::Expand);
        p
    }

    fn set(name: &str, value: &str) -> Op {
        Op::Set {
            name: name.into(),
            value: Some(value.into()),
            kind: None,
        }
    }

    #[test]
    fn rejects_invalid_plans() {
        let plan = |scope, ops| ChangePlan { scope, ops };
        assert!(validate(&plan(Scope::Process, vec![set("A", "1")])).is_err());
        assert!(validate(&plan(Scope::User, vec![])).is_err());
        assert!(
            validate(&plan(Scope::User, vec![set("A", "1"), set("a", "2")])).is_err(),
            "aynı ad iki kez"
        );
        assert!(validate(&plan(Scope::User, vec![set("A=B", "1")])).is_err());
        assert!(validate(&plan(Scope::User, vec![set("A", "x\0")])).is_err());
        let bad_kind = Op::Set {
            name: "A".into(),
            value: Some("1".into()),
            kind: Some(ValueKind::Other),
        };
        assert!(validate(&plan(Scope::User, vec![bad_kind])).is_err());
    }

    #[test]
    fn preview_actions_and_kinds() {
        let p = provider();
        let plan = ChangePlan {
            scope: Scope::User,
            ops: vec![
                set("EDITOR", "code"),
                set("NEW_DIR", "%USERPROFILE%\\x"),
                Op::Delete {
                    name: "NOPE".into(),
                },
                set("JAVA_HOME", "C:\\jdk21"),
            ],
        };
        let pv = preview(&p, &plan, &Overrides::new()).unwrap();
        let get = |n: &str| pv.items.iter().find(|i| i.name == n).unwrap();
        assert_eq!(get("EDITOR").action, Action::Unchanged);
        assert_eq!(get("NEW_DIR").action, Action::Create);
        assert_eq!(
            get("NEW_DIR").kind_after,
            Some(ValueKind::Expand),
            "%AD% içeren yeni değer genişletilebilir olmalı"
        );
        assert_eq!(get("NOPE").action, Action::Unchanged);
        assert!(get("NOPE").warnings.contains(&"missing"));
        assert!(get("JAVA_HOME").warnings.contains(&"shadowsMachine"));
        assert_eq!(pv.changes, 2);
        assert!(!pv.requires_elevation);
    }

    #[test]
    fn preview_masks_sensitive_values() {
        let p = provider();
        let plan = ChangePlan {
            scope: Scope::User,
            ops: vec![set("API_TOKEN", "tok-new-999")],
        };
        let pv = preview(&p, &plan, &Overrides::new()).unwrap();
        assert!(
            pv.items[0].sensitive && pv.items[0].before.is_none() && pv.items[0].after.is_none()
        );
        let json = serde_json::to_string(&pv).unwrap();
        assert!(!json.contains("tok-"), "önizleme gizli değer içermemeli");
    }

    #[test]
    fn machine_scope_requires_elevation_and_warns_critical() {
        let p = provider();
        let plan = ChangePlan {
            scope: Scope::Machine,
            ops: vec![Op::Delete {
                name: "PATH".into(),
            }],
        };
        let pv = preview(&p, &plan, &Overrides::new()).unwrap();
        assert!(pv.requires_elevation);
        assert!(pv.items[0].warnings.contains(&"critical"));
        assert!(pv.items[0].warnings.contains(&"pathDelete"));
        assert_eq!(pv.items[0].name, "Path", "mevcut yazım gösterilmeli");
    }

    #[test]
    fn copy_and_keep_value_resolve_without_frontend_values() {
        let p = provider();
        let plan = ChangePlan {
            scope: Scope::User,
            ops: vec![
                Op::Copy {
                    name: "API_TOKEN_2".into(),
                    source_scope: Scope::User,
                    source_name: "api_token".into(),
                },
                Op::Set {
                    name: "EDITOR".into(),
                    value: None,
                    kind: Some(ValueKind::Expand),
                },
            ],
        };
        let r = resolve(&p, &plan).unwrap();
        assert_eq!(
            r[0],
            Resolved::Set {
                name: "API_TOKEN_2".into(),
                value: "tok-123456".into(),
                kind: ValueKind::String
            }
        );
        assert_eq!(
            r[1],
            Resolved::Set {
                name: "EDITOR".into(),
                value: "code".into(),
                kind: ValueKind::Expand
            }
        );
        let missing = ChangePlan {
            scope: Scope::User,
            ops: vec![Op::Set {
                name: "X".into(),
                value: None,
                kind: None,
            }],
        };
        assert!(resolve(&p, &missing).is_err());
    }

    #[test]
    fn plan_json_shape() {
        let plan: ChangePlan = serde_json::from_str(
            r#"{"scope":"user","ops":[{"op":"set","name":"A","value":"1"},{"op":"delete","name":"B"},
               {"op":"copy","name":"C","sourceScope":"machine","sourceName":"D"}]}"#,
        )
        .unwrap();
        assert_eq!(plan.ops.len(), 3);
    }
}
