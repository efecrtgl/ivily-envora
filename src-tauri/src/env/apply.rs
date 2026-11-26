//! Planların uygulanması: yedek → uygulama → hata olursa geri alma.

use super::inspect::{view_of, Overrides};
use super::model::{name_key, EnvVar, Scope, ValueKind};
use super::plan::{resolve, ChangePlan, Resolved, ValueSource};
use super::provider::EnvProvider;
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::snapshots::store::{self, NewSnapshot, SnapshotItem};
use serde::Serialize;
use std::collections::HashMap;

/// Yalnızca gerçekten değişecek işlemleri bırakır.
pub fn effective_ops(
    p: &dyn EnvProvider,
    scope: Scope,
    ops: Vec<Resolved>,
) -> AppResult<Vec<Resolved>> {
    let current: HashMap<String, EnvVar> = p
        .list(scope)?
        .into_iter()
        .map(|v| (name_key(&v.name), v))
        .collect();
    Ok(ops
        .into_iter()
        .filter(|op| match op {
            Resolved::Delete { name } => current.contains_key(&name_key(name)),
            Resolved::Set { name, value, kind } => current
                .get(&name_key(name))
                .is_none_or(|v| &v.value != value || &v.kind != kind),
        })
        .collect())
}

/// İşlemleri sırayla uygular. Bir işlem başarısız olursa daha önce uygulananları eski
/// hallerine döndürür. Hata mesajı değer içermez.
pub fn apply_ops(p: &dyn EnvProvider, scope: Scope, ops: &[Resolved]) -> AppResult<usize> {
    let before: HashMap<String, Option<EnvVar>> = {
        let current: HashMap<String, EnvVar> = p
            .list(scope)?
            .into_iter()
            .map(|v| (name_key(&v.name), v))
            .collect();
        ops.iter()
            .map(|o| {
                (
                    name_key(o.name()),
                    current.get(&name_key(o.name())).cloned(),
                )
            })
            .collect()
    };
    for (i, op) in ops.iter().enumerate() {
        let res = match op {
            Resolved::Set { name, value, kind } => p.set(scope, name, value, *kind),
            Resolved::Delete { name } => p.delete(scope, name),
        };
        if let Err(e) = res {
            let mut rollback_failed = false;
            for done in ops[..i].iter().rev() {
                let r = match before.get(&name_key(done.name())).cloned().flatten() {
                    Some(v) => p.set(scope, &v.name, &v.value, v.kind),
                    None => p.delete(scope, done.name()),
                };
                rollback_failed |= r.is_err();
            }
            let note = if rollback_failed {
                "geri alma tamamlanamadı"
            } else {
                "önceki değişiklikler geri alındı"
            };
            return Err(match e {
                AppError::PermissionDenied(m) => {
                    AppError::PermissionDenied(format!("{}: {m} ({note})", op.name()))
                }
                other => AppError::Platform(format!("{}: {} ({note})", op.name(), other.code())),
            });
        }
    }
    Ok(ops.len())
}

/// Etkilenen değişkenlerin değişiklik öncesi halini anlık görüntü öğelerine dönüştürür.
pub fn backup_items(
    p: &dyn EnvProvider,
    scope: Scope,
    ops: &[Resolved],
    overrides: &Overrides,
) -> AppResult<Vec<SnapshotItem>> {
    let current: HashMap<String, EnvVar> = p
        .list(scope)?
        .into_iter()
        .map(|v| (name_key(&v.name), v))
        .collect();
    Ok(ops
        .iter()
        .map(|op| match current.get(&name_key(op.name())) {
            Some(v) => SnapshotItem {
                scope,
                name: v.name.clone(),
                kind: v.kind,
                present: true,
                sensitive: view_of(v, overrides).sensitive,
                value: Some(v.value.clone()),
            },
            None => SnapshotItem {
                scope,
                name: op.name().to_string(),
                kind: ValueKind::String,
                present: false,
                sensitive: false,
                value: None,
            },
        })
        .collect())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyOutcome {
    pub applied: usize,
    pub snapshot_id: Option<i64>,
    pub elevated: bool,
}

pub struct ApplyOptions<'a> {
    pub backup: bool,
    pub max_auto_snapshots: u32,
    /// Otomatik anlık görüntünün adı (ör. "Değişiklik öncesi: JAVA_HOME").
    pub label: &'a str,
}

/// Sistem kapsamına bu süreçten yazılamıyorsa çağrılan yükseltilmiş uygulayıcı.
pub type Elevator<'a> = &'a dyn Fn(&[Resolved]) -> AppResult<usize>;

pub fn apply(
    db: &Db,
    p: &dyn EnvProvider,
    plan: &ChangePlan,
    overrides: &Overrides,
    opts: &ApplyOptions,
    src: &dyn ValueSource,
    elevate: Elevator,
) -> AppResult<ApplyOutcome> {
    let ops = effective_ops(p, plan.scope, resolve(p, plan, src)?)?;
    if ops.is_empty() {
        return Ok(ApplyOutcome {
            applied: 0,
            snapshot_id: None,
            elevated: false,
        });
    }
    let snapshot_id = if opts.backup {
        let items = backup_items(p, plan.scope, &ops, overrides)?;
        let id = store::create(
            db,
            NewSnapshot {
                name: opts.label,
                description: None,
                category: "auto",
                source: "environment",
            },
            &items,
        )?;
        let _ = store::prune_auto(db, opts.max_auto_snapshots);
        Some(id)
    } else {
        None
    };
    let elevated = plan.scope == Scope::Machine && !p.can_write_machine();
    let applied = if elevated {
        elevate(&ops)?
    } else {
        apply_ops(p, plan.scope, &ops)?
    };
    p.broadcast_change();
    Ok(ApplyOutcome {
        applied,
        snapshot_id,
        elevated,
    })
}

/// Anlık görüntü öğelerinden geri yükleme işlemleri üretir (yalnızca verilen kapsam).
pub fn restore_ops(items: &[SnapshotItem], scope: Scope) -> Vec<Resolved> {
    items
        .iter()
        .filter(|i| i.scope == scope)
        .filter_map(|i| match (i.present, &i.value) {
            (true, Some(v)) if i.kind != ValueKind::Other => Some(Resolved::Set {
                name: i.name.clone(),
                value: v.clone(),
                kind: i.kind,
            }),
            (false, _) => Some(Resolved::Delete {
                name: i.name.clone(),
            }),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::plan::{NoSources, Op};
    use crate::env::provider::MemoryProvider;
    use crate::error::AppResult;

    fn opts() -> ApplyOptions<'static> {
        ApplyOptions {
            backup: true,
            max_auto_snapshots: 50,
            label: "test",
        }
    }

    fn no_elevate(_: &[Resolved]) -> AppResult<usize> {
        panic!("yükseltme beklenmiyordu")
    }

    #[test]
    fn applies_with_backup_and_restores() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        p.seed(Scope::User, "EDITOR", "vim", ValueKind::String);
        p.seed(Scope::User, "SECRET_KEY", "old-secret", ValueKind::String);
        let plan = ChangePlan {
            scope: Scope::User,
            ops: vec![
                Op::Set {
                    name: "EDITOR".into(),
                    value: Some("code".into()),
                    kind: None,
                },
                Op::Set {
                    name: "NEW".into(),
                    value: Some("1".into()),
                    kind: None,
                },
                Op::Set {
                    name: "SECRET_KEY".into(),
                    value: Some("new-secret".into()),
                    kind: None,
                },
            ],
        };
        let out = apply(
            &db,
            &p,
            &plan,
            &Overrides::new(),
            &opts(),
            &NoSources,
            &no_elevate,
        )
        .unwrap();
        assert_eq!(out.applied, 3);
        assert_eq!(p.get(Scope::User, "EDITOR").unwrap().unwrap().value, "code");

        // Yedek: eski değerler + "NEW yoktu"; gizli değer şifreli saklandı ve çözülebiliyor.
        let items = store::items(&db, out.snapshot_id.unwrap()).unwrap();
        assert!(!items.iter().find(|i| i.name == "NEW").unwrap().present);
        assert_eq!(
            items
                .iter()
                .find(|i| i.name == "SECRET_KEY")
                .unwrap()
                .value
                .as_deref(),
            Some("old-secret")
        );

        // Geri yükleme önceki hale döndürür.
        apply_ops(&p, Scope::User, &restore_ops(&items, Scope::User)).unwrap();
        assert_eq!(p.get(Scope::User, "EDITOR").unwrap().unwrap().value, "vim");
        assert!(p.get(Scope::User, "NEW").unwrap().is_none());
        assert_eq!(
            p.get(Scope::User, "SECRET_KEY").unwrap().unwrap().value,
            "old-secret"
        );
    }

    #[test]
    fn unchanged_plan_does_nothing() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        p.seed(Scope::User, "A", "1", ValueKind::String);
        let plan = ChangePlan {
            scope: Scope::User,
            ops: vec![Op::Set {
                name: "A".into(),
                value: Some("1".into()),
                kind: None,
            }],
        };
        let out = apply(
            &db,
            &p,
            &plan,
            &Overrides::new(),
            &opts(),
            &NoSources,
            &no_elevate,
        )
        .unwrap();
        assert_eq!(out.applied, 0);
        assert!(out.snapshot_id.is_none());
        assert!(store::list(&db).unwrap().is_empty());
    }

    /// İkinci yazmada başarısız olan sağlayıcı: geri alma test edilir.
    struct Flaky {
        inner: MemoryProvider,
        fail_on: &'static str,
    }
    impl EnvProvider for Flaky {
        fn list(&self, s: Scope) -> AppResult<Vec<EnvVar>> {
            self.inner.list(s)
        }
        fn set(&self, s: Scope, n: &str, v: &str, k: ValueKind) -> AppResult<()> {
            if n == self.fail_on {
                return Err(AppError::Io("disk".into()));
            }
            self.inner.set(s, n, v, k)
        }
        fn delete(&self, s: Scope, n: &str) -> AppResult<()> {
            self.inner.delete(s, n)
        }
        fn name(&self) -> &'static str {
            "flaky"
        }
    }

    #[test]
    fn failure_rolls_back_previous_ops() {
        let p = Flaky {
            inner: MemoryProvider::new(),
            fail_on: "B",
        };
        p.inner.seed(Scope::User, "A", "old", ValueKind::String);
        p.inner.seed(Scope::User, "C", "keep", ValueKind::String);
        let ops = vec![
            Resolved::Set {
                name: "A".into(),
                value: "new".into(),
                kind: ValueKind::String,
            },
            Resolved::Delete { name: "C".into() },
            Resolved::Set {
                name: "B".into(),
                value: "x".into(),
                kind: ValueKind::String,
            },
        ];
        let err = apply_ops(&p, Scope::User, &ops).unwrap_err();
        assert!(err.to_string().contains("geri alındı"));
        assert_eq!(p.get(Scope::User, "A").unwrap().unwrap().value, "old");
        assert_eq!(p.get(Scope::User, "C").unwrap().unwrap().value, "keep");
        assert!(
            !err.to_string().contains("new"),
            "hata mesajı değer içermemeli"
        );
    }

    #[test]
    fn machine_scope_uses_elevator_when_not_admin() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        let plan = ChangePlan {
            scope: Scope::Machine,
            ops: vec![Op::Set {
                name: "X".into(),
                value: Some("1".into()),
                kind: None,
            }],
        };
        let called = std::cell::Cell::new(0);
        let elevate = |ops: &[Resolved]| -> AppResult<usize> {
            called.set(ops.len());
            Ok(ops.len())
        };
        let out = apply(
            &db,
            &p,
            &plan,
            &Overrides::new(),
            &opts(),
            &NoSources,
            &elevate,
        )
        .unwrap();
        assert!(out.elevated);
        assert_eq!(called.get(), 1);
        // Yükseltme reddedilirse (UAC iptal) hata aynen döner.
        let cancel = |_: &[Resolved]| -> AppResult<usize> { Err(AppError::Cancelled) };
        assert!(matches!(
            apply(
                &db,
                &p,
                &plan,
                &Overrides::new(),
                &opts(),
                &NoSources,
                &cancel
            ),
            Err(AppError::Cancelled)
        ));
    }
}
