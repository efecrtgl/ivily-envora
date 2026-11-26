//! Ortam karşılaştırma motoru.
//!
//! Karşılaştırma arka uçta gerçek değerlerle yapılır; sonuçta hassas değerler maskelenir
//! (yalnızca "değişti/aynı" bilgisi döner, değer dönmez).

use crate::db::Db;
use crate::env::effective;
use crate::env::inspect::{view_of, Overrides};
use crate::env::model::name_key;
use crate::env::provider::EnvProvider;
use crate::env::{Scope, ValueKind};
use crate::error::AppResult;
use crate::profiles::model::{ProfilePath, ProfileVar, VarAction};
use crate::profiles::store as profiles;
use crate::projects::store as projects;
use crate::security::classify;
use crate::snapshots::store as snapshots;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Source {
    /// Bir kapsamın güncel değerleri.
    Scope {
        scope: Scope,
    },
    /// Yeni başlatılan bir işlemin göreceği birleşik ortam (yaklaşık).
    Effective,
    Profile {
        id: i64,
    },
    /// Projenin tanımı: ilişkili profil + projeye özgü değişkenler (proje önceliklidir).
    Project {
        id: i64,
    },
    Snapshot {
        id: i64,
    },
    /// Arka uçta ayrıştırılmış ve doğrulanmış girdi (ör. içe aktarma önizlemesi).
    #[serde(skip)]
    Inline {
        label: String,
        items: Vec<Item>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Item {
    pub name: String,
    /// `None`: değişken kaldırılıyor ("unset") ya da anlık görüntüde yoktu.
    pub value: Option<String>,
    pub kind: Option<ValueKind>,
    pub sensitive: bool,
}

pub type Items = BTreeMap<String, Item>;

fn add(map: &mut Items, item: Item) {
    map.insert(name_key(&item.name), item);
}

fn from_vars(vars: &[ProfileVar], paths: &[ProfilePath]) -> Items {
    let mut m = Items::new();
    for v in vars {
        add(
            &mut m,
            Item {
                name: v.name.clone(),
                value: if v.action == VarAction::Set {
                    v.value.clone()
                } else {
                    None
                },
                kind: v.kind,
                sensitive: v.sensitive,
            },
        );
    }
    if !paths.is_empty() {
        // PATH değişiklikleri karşılaştırılabilir bir metne dönüştürülür.
        let desc: Vec<String> = paths
            .iter()
            .map(|p| {
                let a = match p.path.action {
                    crate::pathmgr::PathAction::Prepend => "+^",
                    crate::pathmgr::PathAction::Append => "+$",
                    crate::pathmgr::PathAction::Remove => "-",
                };
                format!("{a}{}", p.path.entry)
            })
            .collect();
        add(
            &mut m,
            Item {
                name: "PATH (değişiklikler)".into(),
                value: Some(desc.join(";")),
                kind: None,
                sensitive: false,
            },
        );
    }
    m
}

pub fn load(db: &Db, p: &dyn EnvProvider, overrides: &Overrides, src: &Source) -> AppResult<Items> {
    Ok(match src {
        Source::Scope { scope } => {
            let mut m = Items::new();
            for v in p.list(*scope)? {
                let sensitive = view_of(&v, overrides).sensitive;
                add(
                    &mut m,
                    Item {
                        name: v.name,
                        value: Some(v.value),
                        kind: Some(v.kind),
                        sensitive,
                    },
                );
            }
            m
        }
        Source::Effective => {
            let mut m = Items::new();
            for (k, (name, value)) in effective::fresh_environment(p)? {
                let sensitive =
                    classify::resolve(&name, &value, overrides.get(&k).copied()).sensitive;
                add(
                    &mut m,
                    Item {
                        name,
                        value: Some(value),
                        kind: None,
                        sensitive,
                    },
                );
            }
            m
        }
        Source::Profile { id } => {
            let f = profiles::load_full(db, *id)?;
            from_vars(&f.vars, &f.paths)
        }
        Source::Project { id } => {
            let pr = projects::load_full(db, *id)?;
            let mut m = match pr.profile_id {
                Some(pid) => {
                    let f = profiles::load_full(db, pid)?;
                    from_vars(&f.vars, &f.paths)
                }
                None => Items::new(),
            };
            for (k, v) in from_vars(&pr.vars, &pr.paths) {
                m.insert(k, v);
            }
            m
        }
        Source::Snapshot { id } => {
            let mut m = Items::new();
            let items = snapshots::items(db, *id)?;
            let multi = items
                .iter()
                .map(|i| i.scope)
                .collect::<std::collections::HashSet<_>>()
                .len()
                > 1;
            for i in items {
                // Birden çok kapsam içeren anlık görüntülerde adlar kapsamla ayrılır.
                let name = if multi {
                    format!("{} [{}]", i.name, i.scope.as_str())
                } else {
                    i.name.clone()
                };
                add(
                    &mut m,
                    Item {
                        name,
                        value: if i.present { i.value } else { None },
                        kind: Some(i.kind),
                        sensitive: i.sensitive,
                    },
                );
            }
            m
        }
        Source::Inline { items, .. } => {
            let mut m = Items::new();
            for i in items {
                add(&mut m, i.clone());
            }
            m
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Added,
    Removed,
    Changed,
    Unchanged,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub name: String,
    pub status: Status,
    /// Hassas değerlerde `None`.
    pub left: Option<String>,
    pub right: Option<String>,
    pub left_present: bool,
    pub right_present: bool,
    pub sensitive: bool,
    pub kind_changed: bool,
    pub is_path: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparison {
    pub rows: Vec<Row>,
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
    pub unchanged: usize,
    pub sensitive: usize,
}

/// "Sol → sağ" karşılaştırması: `added` sağda olup solda olmayan, `removed` solda olup sağda olmayandır.
pub fn compare(left: &Items, right: &Items) -> Comparison {
    let mut keys: Vec<&String> = left.keys().chain(right.keys()).collect();
    keys.sort();
    keys.dedup();
    let mut rows = Vec::with_capacity(keys.len());
    for k in keys {
        let l = left.get(k);
        let r = right.get(k);
        let lv = l.and_then(|i| i.value.as_ref());
        let rv = r.and_then(|i| i.value.as_ref());
        let status = match (lv, rv) {
            (None, None) => Status::Unchanged,
            (None, Some(_)) => Status::Added,
            (Some(_), None) => Status::Removed,
            (Some(a), Some(b)) if a == b => Status::Unchanged,
            _ => Status::Changed,
        };
        let sensitive = l.is_some_and(|i| i.sensitive) || r.is_some_and(|i| i.sensitive);
        let kind_changed = matches!((l.and_then(|i| i.kind), r.and_then(|i| i.kind)), (Some(a), Some(b)) if a != b);
        let status = if status == Status::Unchanged && kind_changed {
            Status::Changed
        } else {
            status
        };
        let name = r.or(l).map(|i| i.name.clone()).unwrap_or_default();
        rows.push(Row {
            is_path: name_key(&name) == "PATH",
            name,
            status,
            left: if sensitive { None } else { lv.cloned() },
            right: if sensitive { None } else { rv.cloned() },
            left_present: lv.is_some(),
            right_present: rv.is_some(),
            sensitive,
            kind_changed,
        });
    }
    let count = |s: Status| rows.iter().filter(|r| r.status == s).count();
    Comparison {
        added: count(Status::Added),
        removed: count(Status::Removed),
        changed: count(Status::Changed),
        unchanged: count(Status::Unchanged),
        sensitive: rows
            .iter()
            .filter(|r| r.sensitive && r.status != Status::Unchanged)
            .count(),
        rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::provider::MemoryProvider;
    use crate::profiles::model::{ProfileInput, VarInput};

    fn item(name: &str, value: Option<&str>, sensitive: bool) -> Item {
        Item {
            name: name.into(),
            value: value.map(String::from),
            kind: Some(ValueKind::String),
            sensitive,
        }
    }

    fn items(v: Vec<Item>) -> Items {
        let mut m = Items::new();
        for i in v {
            add(&mut m, i);
        }
        m
    }

    #[test]
    fn classifies_rows_and_masks_secrets() {
        let l = items(vec![
            item("A", Some("1"), false),
            item("B", Some("x"), false),
            item("TOKEN", Some("old"), true),
            item("Same", Some("s"), false),
        ]);
        let r = items(vec![
            item("a", Some("2"), false),
            item("C", Some("new"), false),
            item("TOKEN", Some("new-secret"), true),
            item("SAME", Some("s"), false),
        ]);
        let c = compare(&l, &r);
        let row = |n: &str| {
            c.rows
                .iter()
                .find(|x| x.name.eq_ignore_ascii_case(n))
                .unwrap()
        };
        assert_eq!(
            row("A").status,
            Status::Changed,
            "adlar büyük/küçük harfe duyarsız eşleşir"
        );
        assert_eq!(row("B").status, Status::Removed);
        assert_eq!(row("C").status, Status::Added);
        assert_eq!(row("SAME").status, Status::Unchanged);
        let t = row("TOKEN");
        assert!(t.status == Status::Changed && t.left.is_none() && t.right.is_none());
        assert!(!serde_json::to_string(&c).unwrap().contains("secret"));
        assert_eq!(
            (c.added, c.removed, c.changed, c.unchanged, c.sensitive),
            (1, 1, 2, 1, 1)
        );
    }

    #[test]
    fn kind_change_counts_as_change() {
        let mut a = item("X", Some("v"), false);
        let mut b = a.clone();
        a.kind = Some(ValueKind::String);
        b.kind = Some(ValueKind::Expand);
        let c = compare(&items(vec![a]), &items(vec![b]));
        assert_eq!(c.rows[0].status, Status::Changed);
        assert!(c.rows[0].kind_changed);
    }

    #[test]
    fn loads_scopes_and_profiles() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        p.seed(Scope::User, "JAVA_HOME", "C:\\jdk17", ValueKind::String);
        p.seed(Scope::Machine, "JAVA_HOME", "C:\\jdk21", ValueKind::String);
        let o = Overrides::new();
        let u = load(&db, &p, &o, &Source::Scope { scope: Scope::User }).unwrap();
        let m = load(
            &db,
            &p,
            &o,
            &Source::Scope {
                scope: Scope::Machine,
            },
        )
        .unwrap();
        assert_eq!(compare(&u, &m).changed, 1);

        let input = ProfileInput {
            name: "P".into(),
            description: String::new(),
            category: String::new(),
            tags: vec![],
            metadata: None,
            vars: vec![VarInput {
                name: "JAVA_HOME".into(),
                value: None,
                keep: false,
                previous_name: None,
                sensitive: None,
                kind: None,
                scope: Scope::User,
                action: VarAction::Unset,
            }],
            paths: vec![],
        };
        let id = profiles::create(&db, &input, &o).unwrap();
        let pr = load(&db, &p, &o, &Source::Profile { id }).unwrap();
        assert_eq!(
            compare(&u, &pr).removed,
            1,
            "profilde kaldırılan değişken 'removed' olarak görünür"
        );
    }
}
