//! Profillerin kalıcı ortama uygulanması (etkinleştirme) ve geri alınması (devre dışı bırakma).
//!
//! Etkinleştirme, profili her kalıcı kapsam için ayrı bir değişiklik planına dönüştürür; planlar
//! normal önizleme → yedek → uygulama akışından geçer. Planlar değer içermez (`fromProfile`),
//! yalnızca hesaplanan yeni PATH değeri planda yer alır.
//!
//! İşlem kapsamı tercihli değişkenler kalıcı olarak **yazılmaz**; yalnızca proje başlatmada kullanılır.

use super::model::{Profile, VarAction};
use crate::db::Db;
use crate::env::model::name_key;
use crate::env::plan::{ChangePlan, Op};
use crate::env::provider::EnvProvider;
use crate::env::{Scope, ValueKind};
use crate::error::{AppError, AppResult};
use crate::pathmgr::{entries, PathMod};
use crate::snapshots::store as snapshots;
use rusqlite::{params, OptionalExtension};

/// Profilin kalıcı kapsamlara uygulanma planları (boş kapsamlar atlanır).
pub fn build_plans(p: &dyn EnvProvider, profile: &Profile) -> AppResult<Vec<ChangePlan>> {
    let mut plans = Vec::new();
    for scope in Scope::PERSISTENT {
        let mut ops = Vec::new();
        for v in profile.vars.iter().filter(|v| v.scope == scope) {
            if name_key(&v.name) == "PATH" && profile.paths.iter().any(|x| x.scope == scope) {
                return Err(AppError::validation("path_conflict", v.name.clone()));
            }
            ops.push(match v.action {
                VarAction::Set => Op::FromProfile {
                    name: v.name.clone(),
                    profile_id: profile.id,
                },
                VarAction::Unset => Op::Delete {
                    name: v.name.clone(),
                },
            });
        }
        let mods: Vec<PathMod> = profile
            .paths
            .iter()
            .filter(|x| x.scope == scope)
            .map(|x| x.path.clone())
            .collect();
        if !mods.is_empty() {
            let current = p.get(scope, "Path")?;
            let before = current
                .as_ref()
                .map(|v| v.value.clone())
                .unwrap_or_default();
            let after = entries::apply_mods(&before, &mods);
            if after != before {
                let name = current
                    .as_ref()
                    .map(|v| v.name.clone())
                    .unwrap_or_else(|| "Path".into());
                let kind = current
                    .map(|v| v.kind)
                    .filter(|k| *k != ValueKind::Other)
                    .unwrap_or(ValueKind::Expand);
                ops.push(Op::Set {
                    name,
                    value: Some(after),
                    kind: Some(kind),
                });
            }
        }
        if !ops.is_empty() {
            plans.push(ChangePlan { scope, ops });
        }
    }
    Ok(plans)
}

pub fn record_activation(
    db: &Db,
    profile_id: i64,
    scope: Scope,
    snapshot_id: Option<i64>,
) -> AppResult<()> {
    let name: String = db.with(|c| {
        c.query_row(
            "SELECT name FROM profiles WHERE id = ?1",
            [profile_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("profile {profile_id}")))
    })?;
    db.tx(|tx| {
        // Aynı kapsamdaki önceki etkin kayıt, yenisiyle değiştirilir (yedek zinciri en eski hali korur).
        let previous: Option<Option<i64>> = tx
            .query_row(
                "SELECT snapshot_id FROM profile_activations WHERE profile_id = ?1 AND scope = ?2 AND deactivated_at IS NULL
                 ORDER BY id DESC LIMIT 1",
                params![profile_id, scope.as_str()],
                |r| r.get(0),
            )
            .optional()?;
        let snapshot = match previous {
            // Profil zaten etkinse ilk etkinleştirmenin yedeğini koru: devre dışı bırakma en baştaki hale döner.
            Some(Some(prev)) => Some(prev),
            _ => snapshot_id,
        };
        tx.execute(
            "UPDATE profile_activations SET deactivated_at = ?3 WHERE profile_id = ?1 AND scope = ?2 AND deactivated_at IS NULL",
            params![profile_id, scope.as_str(), chrono::Utc::now().to_rfc3339()],
        )?;
        tx.execute(
            "INSERT INTO profile_activations(profile_id, profile_name, scope, snapshot_id, applied_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![profile_id, name, scope.as_str(), snapshot, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(())
    })
}

/// Etkin bir uygulamanın yedeğinden geri alma planı üretir (değer içermez: `fromSnapshot`).
pub fn deactivation_plan(db: &Db, profile_id: i64, scope: Scope) -> AppResult<ChangePlan> {
    let snapshot_id: Option<i64> = db.with(|c| {
        Ok(c.query_row(
            "SELECT snapshot_id FROM profile_activations WHERE profile_id = ?1 AND scope = ?2 AND deactivated_at IS NULL
             ORDER BY id DESC LIMIT 1",
            params![profile_id, scope.as_str()],
            |r| r.get::<_, Option<i64>>(0),
        )
        .optional()?
        .flatten())
    })?;
    let snapshot_id =
        snapshot_id.ok_or_else(|| AppError::validation("no_backup_for_activation", ""))?;
    let items = snapshots::items(db, snapshot_id)?;
    let ops: Vec<Op> = items
        .iter()
        .filter(|i| i.scope == scope)
        .map(|i| Op::FromSnapshot {
            name: i.name.clone(),
            snapshot_id,
        })
        .collect();
    if ops.is_empty() {
        return Err(AppError::validation("no_backup_for_activation", ""));
    }
    Ok(ChangePlan { scope, ops })
}

pub fn mark_deactivated(db: &Db, profile_id: i64, scope: Scope) -> AppResult<()> {
    db.with(|c| {
        c.execute(
            "UPDATE profile_activations SET deactivated_at = ?3 WHERE profile_id = ?1 AND scope = ?2 AND deactivated_at IS NULL",
            params![profile_id, scope.as_str(), chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(())
    })
}

/// Kaydı tutmayı bırak (yedek olmadan, ortamı değiştirmeden). Ör. kullanıcı değişiklikleri elle geri aldıysa.
pub fn forget_activation(db: &Db, profile_id: i64, scope: Scope) -> AppResult<()> {
    mark_deactivated(db, profile_id, scope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::apply::{apply, ApplyOptions};
    use crate::env::inspect::Overrides;
    use crate::env::provider::MemoryProvider;
    use crate::pathmgr::PathAction;
    use crate::profiles::model::*;
    use crate::profiles::store;
    use crate::sources::DbSources;

    fn vin(name: &str, value: &str, scope: Scope, action: VarAction) -> VarInput {
        VarInput {
            name: name.into(),
            value: Some(value.into()),
            keep: false,
            previous_name: None,
            sensitive: None,
            kind: None,
            scope,
            action,
        }
    }

    #[test]
    fn activate_then_deactivate_round_trip() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        p.seed(
            Scope::User,
            "Path",
            "C:\\Windows;C:\\old-node",
            ValueKind::Expand,
        );
        p.seed(Scope::User, "NODE_ENV", "production", ValueKind::String);
        p.seed(Scope::User, "LEGACY", "x", ValueKind::String);
        let input = ProfileInput {
            name: "Node".into(),
            description: String::new(),
            category: String::new(),
            tags: vec![],
            metadata: None,
            vars: vec![
                vin("NODE_ENV", "development", Scope::User, VarAction::Set),
                vin("NPM_TOKEN", "npm-secret", Scope::User, VarAction::Set),
                vin("LEGACY", "", Scope::User, VarAction::Unset),
                vin("ONLY_FOR_LAUNCH", "1", Scope::Process, VarAction::Set),
            ],
            paths: vec![
                ProfilePath {
                    path: PathMod {
                        entry: "C:\\node".into(),
                        action: PathAction::Prepend,
                    },
                    scope: Scope::User,
                },
                ProfilePath {
                    path: PathMod {
                        entry: "C:\\old-node".into(),
                        action: PathAction::Remove,
                    },
                    scope: Scope::User,
                },
            ],
        };
        let id = store::create(&db, &input, &Overrides::new()).unwrap();
        let profile = store::load_full(&db, id).unwrap();
        let plans = build_plans(&p, &profile).unwrap();
        assert_eq!(
            plans.len(),
            1,
            "yalnızca kullanıcı kapsamı; işlem kapsamı kalıcı yazılmaz"
        );
        let json = serde_json::to_string(&plans).unwrap();
        assert!(!json.contains("npm-secret"), "plan gizli değer içermemeli");

        let src = DbSources::new(&db);
        let opts = ApplyOptions {
            backup: true,
            max_auto_snapshots: 50,
            label: "Node",
        };
        let no_elev = |_: &[crate::env::plan::Resolved]| -> AppResult<usize> { unreachable!() };
        let out = apply(&db, &p, &plans[0], &Overrides::new(), &opts, &src, &no_elev).unwrap();
        record_activation(&db, id, Scope::User, out.snapshot_id).unwrap();

        assert_eq!(
            p.get(Scope::User, "NPM_TOKEN").unwrap().unwrap().value,
            "npm-secret"
        );
        assert!(p.get(Scope::User, "LEGACY").unwrap().is_none());
        assert!(p.get(Scope::User, "ONLY_FOR_LAUNCH").unwrap().is_none());
        assert_eq!(
            p.get(Scope::User, "Path").unwrap().unwrap().value,
            "C:\\node;C:\\Windows"
        );
        assert_eq!(
            store::list(&db, false).unwrap()[0].active_scopes,
            vec![Scope::User]
        );

        // Devre dışı bırakma: önceki hale tam dönüş.
        let back = deactivation_plan(&db, id, Scope::User).unwrap();
        assert!(
            !serde_json::to_string(&back).unwrap().contains("production"),
            "geri alma planı değer içermemeli"
        );
        apply(
            &db,
            &p,
            &back,
            &Overrides::new(),
            &opts,
            &DbSources::new(&db),
            &no_elev,
        )
        .unwrap();
        mark_deactivated(&db, id, Scope::User).unwrap();
        assert_eq!(
            p.get(Scope::User, "NODE_ENV").unwrap().unwrap().value,
            "production"
        );
        assert!(p.get(Scope::User, "NPM_TOKEN").unwrap().is_none());
        assert_eq!(p.get(Scope::User, "LEGACY").unwrap().unwrap().value, "x");
        assert_eq!(
            p.get(Scope::User, "Path").unwrap().unwrap().value,
            "C:\\Windows;C:\\old-node"
        );
        assert!(store::list(&db, false).unwrap()[0].active_scopes.is_empty());
    }

    #[test]
    fn path_var_and_path_mods_conflict() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        let input = ProfileInput {
            name: "X".into(),
            description: String::new(),
            category: String::new(),
            tags: vec![],
            metadata: None,
            vars: vec![vin("PATH", "C:\\a", Scope::User, VarAction::Set)],
            paths: vec![ProfilePath {
                path: PathMod {
                    entry: "C:\\b".into(),
                    action: PathAction::Append,
                },
                scope: Scope::User,
            }],
        };
        let id = store::create(&db, &input, &Overrides::new()).unwrap();
        assert!(build_plans(&p, &store::load_full(&db, id).unwrap()).is_err());
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivationRecord {
    pub id: i64,
    pub scope: Scope,
    pub snapshot_id: Option<i64>,
    pub applied_at: String,
    pub deactivated_at: Option<String>,
}

pub fn history(db: &Db, profile_id: i64) -> AppResult<Vec<ActivationRecord>> {
    db.with(|c| {
        let mut st = c.prepare(
            "SELECT id, scope, snapshot_id, applied_at, deactivated_at FROM profile_activations
             WHERE profile_id = ?1 ORDER BY id DESC LIMIT 100",
        )?;
        let rows = st.query_map([profile_id], |r| {
            let scope: String = r.get(1)?;
            Ok(ActivationRecord {
                id: r.get(0)?,
                scope: Scope::parse(&scope).unwrap_or(Scope::User),
                snapshot_id: r.get(2)?,
                applied_at: r.get(3)?,
                deactivated_at: r.get(4)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    })
}
