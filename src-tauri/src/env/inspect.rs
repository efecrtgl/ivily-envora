//! Ortam inceleme: ön uca gönderilen görünümler.
//!
//! Hassas değişkenlerin değerleri listelerde **gönderilmez** (`value: None, masked: true`).
//! Değeri görmek için ayrı bir `reveal` çağrısı gerekir ve bu çağrı etkinlik kaydına işlenir.

use super::category::{categorize, Category};
use super::model::{name_key, EnvVar, Scope, ValueKind};
use super::provider::EnvProvider;
use super::validate::{lint, Issue};
use crate::error::{AppError, AppResult};
use crate::security::classify::{self, Reason};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VarView {
    pub name: String,
    pub scope: Scope,
    pub kind: ValueKind,
    /// Hassas değişkenlerde `None`.
    pub value: Option<String>,
    pub masked: bool,
    pub sensitive: bool,
    pub sensitive_reason: Option<Reason>,
    pub overridden: bool,
    pub category: Category,
    pub issues: Vec<Issue>,
    /// İşlem kapsamı ve desteklenmeyen kayıt türleri salt okunurdur.
    pub read_only: bool,
}

pub type Overrides = HashMap<String, bool>;

pub fn view_of(v: &EnvVar, overrides: &Overrides) -> VarView {
    let s = classify::resolve(
        &v.name,
        &v.value,
        overrides.get(&name_key(&v.name)).copied(),
    );
    VarView {
        name: v.name.clone(),
        scope: v.scope,
        kind: v.kind,
        value: if s.sensitive {
            None
        } else {
            Some(v.value.clone())
        },
        masked: s.sensitive,
        sensitive: s.sensitive,
        sensitive_reason: s.reason,
        overridden: s.overridden,
        category: categorize(&v.name, s.sensitive),
        issues: lint(&v.name, &v.value, v.kind),
        read_only: v.scope == Scope::Process || v.kind == ValueKind::Other,
    }
}

pub fn list_views(
    p: &dyn EnvProvider,
    scopes: &[Scope],
    overrides: &Overrides,
) -> AppResult<Vec<VarView>> {
    let mut out = Vec::new();
    for &scope in scopes {
        let mut vars = p.list(scope)?;
        vars.sort_by_key(|a| name_key(&a.name));
        out.extend(vars.iter().map(|v| view_of(v, overrides)));
    }
    Ok(out)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeCount {
    pub scope: Scope,
    pub total: usize,
    pub sensitive: usize,
    pub with_issues: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub scopes: Vec<ScopeCount>,
    /// Hem kullanıcı hem sistem kapsamında tanımlı (PATH dışı) adlar: kullanıcı değeri sistem değerini gölgeler.
    pub shadowed: Vec<String>,
    pub can_write_machine: bool,
    pub provider: &'static str,
}

/// Windows, PATH için kullanıcı ve sistem değerlerini birleştirir; diğer adlarda kullanıcı değeri kazanır.
pub fn is_merged_name(name: &str) -> bool {
    name_key(name) == "PATH"
}

pub fn overview(p: &dyn EnvProvider, overrides: &Overrides) -> AppResult<Overview> {
    let mut scopes = Vec::new();
    let mut names: HashMap<Scope, Vec<String>> = HashMap::new();
    for scope in Scope::ALL {
        let vars = p.list(scope)?;
        let views: Vec<VarView> = vars.iter().map(|v| view_of(v, overrides)).collect();
        scopes.push(ScopeCount {
            scope,
            total: views.len(),
            sensitive: views.iter().filter(|v| v.sensitive).count(),
            with_issues: views.iter().filter(|v| !v.issues.is_empty()).count(),
        });
        names.insert(scope, vars.into_iter().map(|v| v.name).collect());
    }
    let machine: std::collections::HashSet<String> = names
        .get(&Scope::Machine)
        .into_iter()
        .flatten()
        .map(|n| name_key(n))
        .collect();
    let mut shadowed: Vec<String> = names
        .get(&Scope::User)
        .into_iter()
        .flatten()
        .filter(|n| machine.contains(&name_key(n)) && !is_merged_name(n))
        .cloned()
        .collect();
    shadowed.sort();
    Ok(Overview {
        scopes,
        shadowed,
        can_write_machine: p.can_write_machine(),
        provider: p.name(),
    })
}

/// `%AD%` başvurularını verilen arama işleviyle genişletir. Bulunamayan veya hassas olan
/// başvurular olduğu gibi bırakılır (Windows da bulunamayanları değiştirmez).
pub fn expand_references(value: &str, lookup: &dyn Fn(&str) -> Option<String>) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('%') {
            Some(end) if end > 0 => {
                let name = &after[..end];
                match lookup(name) {
                    Some(v) => out.push_str(&v),
                    None => {
                        out.push('%');
                        out.push_str(name);
                        out.push('%');
                    }
                }
                rest = &after[end + 1..];
            }
            _ => {
                out.push('%');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeEntry {
    pub view: VarView,
    /// `REG_EXPAND_SZ` değerlerin genişletilmiş önizlemesi (hassas başvurular genişletilmez).
    pub expanded: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Detail {
    pub name: String,
    pub entries: Vec<ScopeEntry>,
    /// Yeni başlatılan bir işlemin göreceği kalıcı kapsam (kullanıcı > sistem; PATH birleştirilir).
    pub effective_scope: Option<Scope>,
    /// İşlem kapsamındaki değer kalıcı değerden farklı mı? (Envora başlatıldıktan sonra değişmiş olabilir.)
    pub process_differs: bool,
}

pub fn detail(p: &dyn EnvProvider, name: &str, overrides: &Overrides) -> AppResult<Detail> {
    let key = name_key(name);
    let mut all: HashMap<Scope, Vec<EnvVar>> = HashMap::new();
    for scope in Scope::ALL {
        all.insert(scope, p.list(scope)?);
    }
    let sensitive_names: std::collections::HashSet<String> = all
        .values()
        .flatten()
        .filter(|v| view_of(v, overrides).sensitive)
        .map(|v| name_key(&v.name))
        .collect();
    let lookup = |n: &str| -> Option<String> {
        let k = name_key(n);
        if sensitive_names.contains(&k) {
            return None;
        }
        for scope in [Scope::User, Scope::Machine, Scope::Process] {
            if let Some(v) = all[&scope].iter().find(|v| name_key(&v.name) == k) {
                return Some(v.value.clone());
            }
        }
        None
    };

    let mut entries = Vec::new();
    for scope in Scope::ALL {
        if let Some(v) = all[&scope].iter().find(|v| name_key(&v.name) == key) {
            let view = view_of(v, overrides);
            let expanded = if v.kind == ValueKind::Expand && !view.sensitive {
                Some(expand_references(&v.value, &lookup))
            } else {
                None
            };
            entries.push(ScopeEntry { view, expanded });
        }
    }
    if entries.is_empty() {
        return Err(AppError::NotFound(name.to_string()));
    }
    let has = |s: Scope| entries.iter().any(|e| e.view.scope == s);
    let effective_scope = if has(Scope::User) {
        Some(Scope::User)
    } else if has(Scope::Machine) {
        Some(Scope::Machine)
    } else {
        None
    };
    let process_differs = match (
        all[&Scope::Process]
            .iter()
            .find(|v| name_key(&v.name) == key),
        effective_scope.and_then(|s| all[&s].iter().find(|v| name_key(&v.name) == key)),
    ) {
        (Some(proc_v), Some(pers)) if !is_merged_name(name) => {
            let pers_value = if pers.kind == ValueKind::Expand {
                expand_references(&pers.value, &|n| std::env::var(n).ok())
            } else {
                pers.value.clone()
            };
            proc_v.value != pers_value
        }
        (Some(_), None) | (None, Some(_)) => true,
        _ => false,
    };
    Ok(Detail {
        name: entries[0].view.name.clone(),
        entries,
        effective_scope,
        process_differs,
    })
}

/// Hassas bir değeri açıkça kullanıcı isteğiyle döndürür.
pub fn reveal(p: &dyn EnvProvider, scope: Scope, name: &str) -> AppResult<String> {
    p.get(scope, name)?
        .map(|v| v.value)
        .ok_or_else(|| AppError::NotFound(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::provider::MemoryProvider;

    fn sample() -> MemoryProvider {
        let p = MemoryProvider::new();
        p.seed(Scope::Machine, "Path", "C:\\Windows", ValueKind::Expand);
        p.seed(Scope::User, "Path", "%USERPROFILE%\\bin", ValueKind::Expand);
        p.seed(
            Scope::User,
            "USERPROFILE",
            "C:\\Users\\u",
            ValueKind::String,
        );
        p.seed(
            Scope::User,
            "GITHUB_TOKEN",
            concat!("gh", "p_", "notarealtokenvalue0000000000"),
            ValueKind::String,
        );
        p.seed(Scope::User, "LEAK", "%GITHUB_TOKEN%", ValueKind::Expand);
        p.seed(Scope::User, "TEMP", "C:\\u\\tmp", ValueKind::Expand);
        p.seed(
            Scope::Machine,
            "TEMP",
            "C:\\Windows\\Temp",
            ValueKind::Expand,
        );
        p
    }

    #[test]
    fn sensitive_values_are_not_listed() {
        let p = sample();
        let views = list_views(&p, &Scope::ALL, &Overrides::new()).unwrap();
        let tok = views.iter().find(|v| v.name == "GITHUB_TOKEN").unwrap();
        assert!(tok.masked && tok.value.is_none());
        let json = serde_json::to_string(&views).unwrap();
        assert!(
            !json.contains("ghp_"),
            "liste JSON'u gizli değer içermemeli"
        );
    }

    #[test]
    fn override_unmasks() {
        let p = sample();
        let mut o = Overrides::new();
        o.insert("GITHUB_TOKEN".into(), false);
        let views = list_views(&p, &[Scope::User], &o).unwrap();
        assert!(views
            .iter()
            .find(|v| v.name == "GITHUB_TOKEN")
            .unwrap()
            .value
            .is_some());
    }

    #[test]
    fn expansion_never_reveals_sensitive_references() {
        let p = sample();
        let d = detail(&p, "LEAK", &Overrides::new()).unwrap();
        assert_eq!(d.entries[0].expanded.as_deref(), Some("%GITHUB_TOKEN%"));
        let d = detail(&p, "path", &Overrides::new()).unwrap();
        assert_eq!(d.entries.len(), 2);
        let user = d
            .entries
            .iter()
            .find(|e| e.view.scope == Scope::User)
            .unwrap();
        assert_eq!(user.expanded.as_deref(), Some("C:\\Users\\u\\bin"));
    }

    #[test]
    fn overview_counts_and_shadowing() {
        let p = sample();
        let o = overview(&p, &Overrides::new()).unwrap();
        let user = o.scopes.iter().find(|s| s.scope == Scope::User).unwrap();
        assert_eq!(user.total, 5);
        assert_eq!(user.sensitive, 1);
        assert_eq!(
            o.shadowed,
            vec!["TEMP".to_string()],
            "PATH birleştirilir, gölgeleme sayılmaz"
        );
    }

    #[test]
    fn expand_edge_cases() {
        let l = |n: &str| {
            if n == "A" {
                Some("1".to_string())
            } else {
                None
            }
        };
        assert_eq!(expand_references("%A%-%B%-100%", &l), "1-%B%-100%");
        assert_eq!(expand_references("%%A%", &l), "%1");
        assert_eq!(expand_references("no refs", &l), "no refs");
    }

    #[test]
    fn reveal_and_missing() {
        let p = sample();
        assert!(reveal(&p, Scope::User, "github_token")
            .unwrap()
            .starts_with("ghp_"));
        assert!(matches!(
            reveal(&p, Scope::Machine, "NOPE"),
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            detail(&p, "NOPE", &Overrides::new()),
            Err(AppError::NotFound(_))
        ));
    }
}
