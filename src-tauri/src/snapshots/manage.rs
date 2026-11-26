//! Anlık görüntü oluşturma, ayrıntı, güncelleme ve geri yükleme planları.

use super::store::{self, NewSnapshot, SnapshotItem, SnapshotMeta};
use crate::db::Db;
use crate::env::inspect::{view_of, Overrides};
use crate::env::model::name_key;
use crate::env::plan::{ChangePlan, Op};
use crate::env::provider::EnvProvider;
use crate::env::{Scope, ValueKind};
use crate::error::{AppError, AppResult};
use crate::profiles::model::VarAction;
use crate::profiles::store as profiles;
use crate::projects::{launch, store as projects};
use crate::security::classify;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SnapshotSource {
    /// Kalıcı/işlem ortamı. `names` boşsa seçilen kapsamlardaki tüm değişkenler alınır.
    #[serde(rename_all = "camelCase")]
    Environment {
        scopes: Vec<Scope>,
        names: Option<Vec<String>>,
    },
    Profile {
        id: i64,
    },
    /// Projenin başlatma ortamı (işlem kapsamı olarak saklanır).
    Project {
        id: i64,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub source: SnapshotSource,
    #[serde(default)]
    pub exclude_sensitive: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResult {
    pub id: i64,
    pub items: usize,
    pub excluded_sensitive: usize,
}

pub fn create(
    db: &Db,
    p: &dyn EnvProvider,
    overrides: &Overrides,
    req: &CreateRequest,
) -> AppResult<CreateResult> {
    let name = req.name.trim();
    if name.is_empty() || name.chars().count() > 120 {
        return Err(AppError::validation("invalid_snapshot_name", ""));
    }
    let (source, mut items): (&str, Vec<SnapshotItem>) = match &req.source {
        SnapshotSource::Environment { scopes, names } => {
            if scopes.is_empty() {
                return Err(AppError::validation("no_scope_selected", ""));
            }
            let wanted: Option<HashSet<String>> = names
                .as_ref()
                .map(|n| n.iter().map(|x| name_key(x)).collect());
            let mut out = Vec::new();
            for &scope in scopes {
                for v in p.list(scope)? {
                    if wanted
                        .as_ref()
                        .is_some_and(|w| !w.contains(&name_key(&v.name)))
                    {
                        continue;
                    }
                    let sensitive = view_of(&v, overrides).sensitive;
                    out.push(SnapshotItem {
                        scope,
                        name: v.name,
                        kind: v.kind,
                        present: true,
                        sensitive,
                        value: Some(v.value),
                    });
                }
                // Seçilen adlardan bu kapsamda olmayanlar "yoktu" olarak kaydedilir; geri yüklemede silinir.
                if let (Some(w), true) = (&wanted, scope.is_persistent()) {
                    let present: HashSet<String> = out
                        .iter()
                        .filter(|i| i.scope == scope)
                        .map(|i| name_key(&i.name))
                        .collect();
                    for n in names.as_ref().into_iter().flatten() {
                        if w.contains(&name_key(n)) && !present.contains(&name_key(n)) {
                            out.push(SnapshotItem {
                                scope,
                                name: n.clone(),
                                kind: ValueKind::String,
                                present: false,
                                sensitive: false,
                                value: None,
                            });
                        }
                    }
                }
            }
            ("environment", out)
        }
        SnapshotSource::Profile { id } => {
            let f = profiles::load_full(db, *id)?;
            let out = f
                .vars
                .into_iter()
                .map(|v| SnapshotItem {
                    scope: v.scope,
                    kind: v.kind.unwrap_or(ValueKind::String),
                    present: v.action == VarAction::Set,
                    sensitive: v.sensitive,
                    value: if v.action == VarAction::Set {
                        v.value
                    } else {
                        None
                    },
                    name: v.name,
                })
                .collect();
            ("profile", out)
        }
        SnapshotSource::Project { id } => {
            let pr = projects::load_full(db, *id)?;
            let profile = match pr.profile_id {
                Some(pid) => Some(profiles::load_full(db, pid)?),
                None => None,
            };
            let env = launch::launch_env(
                p,
                &pr,
                profile
                    .as_ref()
                    .map(|f| (f.vars.as_slice(), f.paths.as_slice())),
            )?;
            let secret_names: HashSet<String> = pr
                .vars
                .iter()
                .chain(profile.iter().flat_map(|f| f.vars.iter()))
                .filter(|v| v.sensitive)
                .map(|v| name_key(&v.name))
                .collect();
            let out = env
                .into_iter()
                .map(|(k, (name, value))| {
                    let sensitive = secret_names.contains(&k)
                        || classify::resolve(&name, &value, overrides.get(&k).copied()).sensitive;
                    SnapshotItem {
                        scope: Scope::Process,
                        name,
                        kind: ValueKind::String,
                        present: true,
                        sensitive,
                        value: Some(value),
                    }
                })
                .collect();
            ("project", out)
        }
    };
    let before = items.len();
    if req.exclude_sensitive {
        items.retain(|i| !i.sensitive);
    }
    let excluded_sensitive = before - items.len();
    let id = store::create(
        db,
        NewSnapshot {
            name,
            description: req
                .description
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty()),
            category: "manual",
            source,
        },
        &items,
    )?;
    Ok(CreateResult {
        id,
        items: items.len(),
        excluded_sensitive,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Detail {
    pub meta: SnapshotMeta,
    /// Hassas değerler maskeli.
    pub items: Vec<SnapshotItem>,
}

pub fn detail(db: &Db, id: i64) -> AppResult<Detail> {
    let meta = store::get_meta(db, id)?;
    let items = store::items(db, id)?
        .into_iter()
        .map(|mut i| {
            if i.sensitive {
                i.value = None;
            }
            i
        })
        .collect();
    Ok(Detail { meta, items })
}

pub fn update_meta(db: &Db, id: i64, name: &str, description: Option<&str>) -> AppResult<()> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 120 {
        return Err(AppError::validation("invalid_snapshot_name", ""));
    }
    let n = db.with(|c| {
        Ok(c.execute(
            "UPDATE snapshots SET name = ?2, description = ?3 WHERE id = ?1",
            params![
                id,
                name,
                description.map(str::trim).filter(|s| !s.is_empty())
            ],
        )?)
    })?;
    if n == 0 {
        return Err(AppError::NotFound(format!("snapshot {id}")));
    }
    Ok(())
}

pub fn reveal(db: &Db, id: i64, scope: Scope, name: &str) -> AppResult<String> {
    store::items(db, id)?
        .into_iter()
        .find(|i| i.scope == scope && name_key(&i.name) == name_key(name))
        .and_then(|i| i.value)
        .ok_or_else(|| AppError::NotFound(name.to_string()))
}

/// Geri yükleme planları: kalıcı kapsam başına bir plan. `names` verilirse yalnızca o değişkenler.
/// Planlar değer içermez (`fromSnapshot`). İşlem kapsamındaki öğeler geri yüklenemez.
pub fn restore_plans(db: &Db, id: i64, names: Option<&[String]>) -> AppResult<Vec<ChangePlan>> {
    let items = store::items(db, id)?;
    let wanted: Option<HashSet<String>> = names.map(|n| n.iter().map(|x| name_key(x)).collect());
    let mut plans = Vec::new();
    for scope in Scope::PERSISTENT {
        let ops: Vec<Op> = items
            .iter()
            .filter(|i| i.scope == scope && i.kind != ValueKind::Other)
            .filter(|i| {
                wanted
                    .as_ref()
                    .is_none_or(|w| w.contains(&name_key(&i.name)))
            })
            .map(|i| Op::FromSnapshot {
                name: i.name.clone(),
                snapshot_id: id,
            })
            .collect();
        if !ops.is_empty() {
            plans.push(ChangePlan { scope, ops });
        }
    }
    if plans.is_empty() {
        return Err(AppError::validation("nothing_to_restore", ""));
    }
    Ok(plans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::apply::{apply, ApplyOptions};
    use crate::env::provider::MemoryProvider;
    use crate::sources::DbSources;

    fn env_req(names: Option<Vec<&str>>, exclude: bool) -> CreateRequest {
        CreateRequest {
            name: "test".into(),
            description: None,
            source: SnapshotSource::Environment {
                scopes: vec![Scope::User],
                names: names.map(|n| n.into_iter().map(String::from).collect()),
            },
            exclude_sensitive: exclude,
        }
    }

    #[test]
    fn create_with_selection_and_exclusion() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        p.seed(Scope::User, "A", "1", ValueKind::String);
        p.seed(Scope::User, "B", "2", ValueKind::String);
        p.seed(Scope::User, "API_TOKEN", "secret", ValueKind::String);
        let o = Overrides::new();
        let all = create(&db, &p, &o, &env_req(None, false)).unwrap();
        assert_eq!(all.items, 3);
        let safe = create(&db, &p, &o, &env_req(None, true)).unwrap();
        assert_eq!((safe.items, safe.excluded_sensitive), (2, 1));
        let sel = create(&db, &p, &o, &env_req(Some(vec!["a", "MISSING"]), false)).unwrap();
        let d = detail(&db, sel.id).unwrap();
        assert_eq!(d.items.len(), 2);
        assert!(
            !d.items
                .iter()
                .find(|i| i.name == "MISSING")
                .unwrap()
                .present
        );
        let full = detail(&db, all.id).unwrap();
        assert!(
            full.items
                .iter()
                .find(|i| i.name == "API_TOKEN")
                .unwrap()
                .value
                .is_none(),
            "ayrıntı maskeli"
        );
        assert_eq!(
            reveal(&db, all.id, Scope::User, "api_token").unwrap(),
            "secret"
        );
    }

    #[test]
    fn restore_round_trip_including_absent() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        p.seed(Scope::User, "A", "1", ValueKind::String);
        let o = Overrides::new();
        let snap = create(&db, &p, &o, &env_req(Some(vec!["A", "NEWER"]), false)).unwrap();
        p.set(Scope::User, "A", "changed", ValueKind::String)
            .unwrap();
        p.set(Scope::User, "NEWER", "x", ValueKind::String).unwrap();
        let plans = restore_plans(&db, snap.id, None).unwrap();
        assert_eq!(plans.len(), 1);
        let opts = ApplyOptions {
            backup: true,
            max_auto_snapshots: 50,
            label: "restore",
        };
        let no_elev = |_: &[crate::env::plan::Resolved]| -> AppResult<usize> { unreachable!() };
        apply(
            &db,
            &p,
            &plans[0],
            &o,
            &opts,
            &DbSources::new(&db),
            &no_elev,
        )
        .unwrap();
        assert_eq!(p.get(Scope::User, "A").unwrap().unwrap().value, "1");
        assert!(
            p.get(Scope::User, "NEWER").unwrap().is_none(),
            "anlık görüntüde olmayan değişken silinir"
        );
        // Seçimli geri yükleme
        assert_eq!(
            restore_plans(&db, snap.id, Some(&["A".into()])).unwrap()[0]
                .ops
                .len(),
            1
        );
        assert!(restore_plans(&db, snap.id, Some(&["X".into()])).is_err());
    }

    #[test]
    fn process_only_snapshots_cannot_be_restored() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        p.seed(Scope::Process, "A", "1", ValueKind::String);
        let req = CreateRequest {
            name: "p".into(),
            description: None,
            source: SnapshotSource::Environment {
                scopes: vec![Scope::Process],
                names: None,
            },
            exclude_sensitive: false,
        };
        let s = create(&db, &p, &Overrides::new(), &req).unwrap();
        assert!(matches!(
            restore_plans(&db, s.id, None),
            Err(AppError::Validation {
                code: "nothing_to_restore",
                ..
            })
        ));
    }
}
