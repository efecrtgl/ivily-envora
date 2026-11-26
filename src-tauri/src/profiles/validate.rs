//! Profil doğrulaması: profili değiştirmeden bulguları raporlar.

use super::model::{Profile, VarAction};
use crate::env::inspect::expand_references;
use crate::env::model::name_key;
use crate::env::plan::is_critical;
use crate::env::provider::EnvProvider;
use crate::env::validate::{lint, Issue};
use crate::env::{Scope, ValueKind};
use crate::error::AppResult;
use crate::pathmgr::PathAction;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub severity: Severity,
    /// i18n anahtarı: `profileCheck.<code>`
    pub code: String,
    pub target: String,
}

fn f(severity: Severity, code: &str, target: &str) -> Finding {
    Finding {
        severity,
        code: code.to_string(),
        target: target.to_string(),
    }
}

/// `%AD%` başvurularındaki adları döndürür.
pub fn references(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let parts: Vec<&str> = value.split('%').collect();
    let mut i = 1;
    while i + 1 < parts.len() {
        let name = parts[i];
        if !name.is_empty() && !name.contains(' ') {
            out.push(name.to_string());
            i += 2;
        } else {
            i += 1;
        }
    }
    out
}

/// Dizin var mı? `%VAR%` başvuruları ortama göre genişletilir.
fn dir_exists(entry: &str, lookup: &dyn Fn(&str) -> Option<String>) -> Option<bool> {
    let expanded = expand_references(entry, lookup);
    if expanded.contains('%') {
        return None; // çözülemeyen başvuru: bilinmiyor
    }
    Some(std::path::Path::new(expanded.trim().trim_matches('"')).is_dir())
}

pub fn is_relative_path(entry: &str) -> bool {
    let e = entry.trim().trim_matches('"');
    if e.starts_with('%') || e.starts_with("\\\\") {
        return false;
    }
    let b = e.as_bytes();
    !(b.len() >= 3 && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/'))
}

pub fn check(p: &dyn EnvProvider, profile: &Profile) -> AppResult<Vec<Finding>> {
    let mut out = Vec::new();
    let mut known: HashSet<String> = HashSet::new();
    for scope in Scope::ALL {
        for v in p.list(scope)? {
            known.insert(name_key(&v.name));
        }
    }
    for v in &profile.vars {
        known.insert(name_key(&v.name));
    }
    let lookup_env = |n: &str| -> Option<String> {
        for scope in [Scope::User, Scope::Machine, Scope::Process] {
            if let Ok(Some(v)) = p.get(scope, n) {
                return Some(v.value);
            }
        }
        None
    };

    for v in &profile.vars {
        if v.action == VarAction::Unset {
            if v.scope == Scope::Process {
                out.push(f(Severity::Info, "unsetProcess", &v.name));
            }
            continue;
        }
        let value = v.value.as_deref().unwrap_or("");
        let kind = v.kind.unwrap_or(ValueKind::String);
        for issue in lint(&v.name, value, kind) {
            // Tür seçilmemişse (otomatik) uygulamada genişletilebilir seçilir; bu uyarı geçersiz olur.
            if issue == Issue::UnexpandedReference && v.kind.is_none() {
                continue;
            }
            let code = serde_json::to_value(issue)
                .ok()
                .and_then(|x| x.as_str().map(String::from))
                .unwrap_or_default();
            out.push(Finding {
                severity: Severity::Warning,
                code: format!("issue.{code}"),
                target: v.name.clone(),
            });
        }
        if !v.sensitive {
            for r in references(value) {
                if !known.contains(&name_key(&r)) {
                    out.push(f(
                        Severity::Warning,
                        "undefinedReference",
                        &format!("{} → %{r}%", v.name),
                    ));
                }
            }
        }
        if v.scope == Scope::Machine && is_critical(&v.name) {
            out.push(f(Severity::Warning, "critical", &v.name));
        }
    }

    for x in &profile.paths {
        let e = &x.path.entry;
        if x.path.action == PathAction::Remove {
            continue;
        }
        if is_relative_path(e) {
            out.push(f(Severity::Warning, "pathRelative", e));
        } else if dir_exists(e, &lookup_env) == Some(false) {
            out.push(f(Severity::Warning, "pathMissing", e));
        }
    }
    if profile.vars.iter().any(|v| name_key(&v.name) == "PATH") && !profile.paths.is_empty() {
        out.push(f(Severity::Error, "pathConflict", "PATH"));
    }
    if profile.vars.is_empty() && profile.paths.is_empty() {
        out.push(f(Severity::Info, "empty", &profile.name));
    }
    out.sort_by_key(|f| std::cmp::Reverse(f.severity));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::provider::MemoryProvider;
    use crate::pathmgr::PathMod;
    use crate::profiles::model::*;

    #[test]
    fn reference_parsing() {
        assert_eq!(references("%A%\\%B%"), vec!["A", "B"]);
        assert_eq!(references("100% sure %X%"), vec!["X"]);
        assert!(references("no refs").is_empty());
    }

    #[test]
    fn relative_detection() {
        assert!(!is_relative_path("C:\\x"));
        assert!(!is_relative_path("%USERPROFILE%\\bin"));
        assert!(!is_relative_path("\\\\server\\share"));
        assert!(is_relative_path("bin"));
        assert!(is_relative_path(".\\node_modules\\.bin"));
    }

    #[test]
    fn findings() {
        let p = MemoryProvider::new();
        p.seed(
            Scope::User,
            "USERPROFILE",
            "C:\\Users\\u",
            ValueKind::String,
        );
        let profile = Profile {
            id: 1,
            name: "t".into(),
            description: String::new(),
            category: String::new(),
            tags: vec![],
            status: Status::Active,
            version: 1,
            metadata: serde_json::json!({}),
            created_at: String::new(),
            updated_at: String::new(),
            vars: vec![
                ProfileVar {
                    name: "A".into(),
                    value: Some("%NOPE%\\x".into()),
                    sensitive: false,
                    kind: None,
                    scope: Scope::User,
                    action: VarAction::Set,
                },
                ProfileVar {
                    name: "B".into(),
                    value: Some(" padded".into()),
                    sensitive: false,
                    kind: None,
                    scope: Scope::User,
                    action: VarAction::Set,
                },
                ProfileVar {
                    name: "Path".into(),
                    value: Some("x".into()),
                    sensitive: false,
                    kind: None,
                    scope: Scope::Machine,
                    action: VarAction::Set,
                },
            ],
            paths: vec![
                ProfilePath {
                    path: PathMod {
                        entry: "relative\\bin".into(),
                        action: PathAction::Append,
                    },
                    scope: Scope::User,
                },
                ProfilePath {
                    path: PathMod {
                        entry: "Z:\\definitely\\missing\\envora".into(),
                        action: PathAction::Append,
                    },
                    scope: Scope::User,
                },
            ],
        };
        let codes: Vec<String> = check(&p, &profile)
            .unwrap()
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert_eq!(codes[0], "pathConflict", "hatalar önce sıralanmalı");
        for c in [
            "undefinedReference",
            "issue.valueWhitespace",
            "critical",
            "pathRelative",
            "pathMissing",
        ] {
            assert!(codes.iter().any(|x| x == c), "{c} bekleniyordu: {codes:?}");
        }
    }
}
