//! PATH analizi: girdi durumları, yinelenenler, uzunluk sınırları ve yürütülebilir dosya çakışmaları.
//!
//! Analiz yalnızca okur; hiçbir girdiyi kaldırmaz veya sıralamaz.

use super::entries::{key, split};
use crate::env::effective::{self, EnvMap};
use crate::env::inspect::expand_references;
use crate::env::model::Scope;
use crate::env::provider::EnvProvider;
use crate::env::ValueKind;
use crate::error::AppResult;
use crate::profiles::validate::is_relative_path;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// Windows'ta tek bir ortam değişkeninin en fazla uzunluğu.
pub const MAX_ENV_LEN: usize = 32_767;
/// Bazı eski araçların (ör. eski kurulum programları, `setx`) sorun yaşadığı eşik.
pub const LEGACY_LEN: usize = 2_047;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Flag {
    /// Girdi boş (`;;`).
    Empty,
    /// Dizin bulunamadı.
    Missing,
    /// Yol bir dosyayı gösteriyor, dizini değil.
    NotDirectory,
    /// Göreli yol.
    Relative,
    /// Aynı kapsamda daha önce geçiyor.
    Duplicate,
    /// Diğer kalıcı kapsamda da var (kullanıcı ↔ sistem).
    DuplicateOtherScope,
    /// Çözülemeyen `%AD%` başvurusu.
    UnresolvedReference,
    /// Tırnak içeriyor; bazı araçlar tırnaklı girdileri anlamaz.
    Quoted,
    /// Başında/sonunda boşluk var.
    Whitespace,
    /// Ağ yolu (UNC); erişim yavaş olabilir veya bağlantı yoksa komut aramayı geciktirir.
    Network,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryReport {
    pub index: usize,
    pub raw: String,
    /// Genişletilmiş yol (başvuru yoksa ham değerle aynı).
    pub expanded: String,
    pub flags: Vec<Flag>,
    /// Aynı girdinin ilk geçtiği sıra (yinelenenler için).
    pub duplicate_of: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathReport {
    pub scope: Scope,
    /// Kayıttaki adın yazımı (ör. `Path`); değişken yoksa `None`.
    pub name: Option<String>,
    pub kind: Option<ValueKind>,
    pub value: String,
    pub length: usize,
    pub entries: Vec<EntryReport>,
    pub counts: BTreeMap<String, usize>,
    pub read_only: bool,
}

fn flag_name(f: Flag) -> String {
    serde_json::to_value(f)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

fn analyze_entries(
    value: &str,
    expand: bool,
    env: &EnvMap,
    other_keys: &[String],
) -> Vec<EntryReport> {
    let mut first: HashMap<String, usize> = HashMap::new();
    split(value)
        .into_iter()
        .enumerate()
        .map(|(i, raw)| {
            let mut flags = Vec::new();
            let trimmed = raw.trim();
            let expanded = if expand {
                expand_references(trimmed, &effective::lookup(env))
            } else {
                trimmed.to_string()
            };
            let mut duplicate_of = None;
            if trimmed.is_empty() {
                flags.push(Flag::Empty);
            } else {
                if raw != trimmed {
                    flags.push(Flag::Whitespace);
                }
                if trimmed.contains('"') {
                    flags.push(Flag::Quoted);
                }
                if expanded.contains('%') && expanded.split('%').count() >= 3 {
                    flags.push(Flag::UnresolvedReference);
                } else if is_relative_path(&expanded) {
                    flags.push(Flag::Relative);
                } else {
                    let clean = expanded.trim_matches('"');
                    if clean.starts_with("\\\\") {
                        flags.push(Flag::Network);
                    } else {
                        let p = Path::new(clean);
                        if p.is_file() {
                            flags.push(Flag::NotDirectory);
                        } else if !p.is_dir() {
                            flags.push(Flag::Missing);
                        }
                    }
                }
                let k = key(&expanded);
                match first.get(&k) {
                    Some(&j) => {
                        flags.push(Flag::Duplicate);
                        duplicate_of = Some(j);
                    }
                    None => {
                        first.insert(k.clone(), i);
                    }
                }
                if other_keys.contains(&k) {
                    flags.push(Flag::DuplicateOtherScope);
                }
            }
            EntryReport {
                index: i,
                raw,
                expanded,
                flags,
                duplicate_of,
            }
        })
        .collect()
}

fn expanded_keys(value: &str, expand: bool, env: &EnvMap) -> Vec<String> {
    split(value)
        .iter()
        .filter(|e| !e.trim().is_empty())
        .map(|e| {
            let x = if expand {
                expand_references(e.trim(), &effective::lookup(env))
            } else {
                e.trim().to_string()
            };
            key(&x)
        })
        .collect()
}

/// Bir kapsamın PATH raporunu üretir.
pub fn analyze(p: &dyn EnvProvider, scope: Scope) -> AppResult<PathReport> {
    let env = effective::fresh_environment(p)?;
    let get = |s: Scope| p.get(s, "Path");
    let var = get(scope)?;
    let (name, kind, value) = match &var {
        Some(v) => (Some(v.name.clone()), Some(v.kind), v.value.clone()),
        None => (None, None, String::new()),
    };
    let expand = scope == Scope::Process || kind == Some(ValueKind::Expand) || value.contains('%');
    let other_keys = match scope {
        Scope::User | Scope::Machine => {
            let other = if scope == Scope::User {
                Scope::Machine
            } else {
                Scope::User
            };
            match get(other)? {
                Some(o) => expanded_keys(&o.value, true, &env),
                None => Vec::new(),
            }
        }
        Scope::Process => Vec::new(),
    };
    let entries = analyze_entries(&value, expand, &env, &other_keys);
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for e in &entries {
        for f in &e.flags {
            *counts.entry(flag_name(*f)).or_default() += 1;
        }
    }
    Ok(PathReport {
        scope,
        name,
        kind,
        length: value.encode_utf16().count(),
        value,
        entries,
        counts,
        read_only: scope == Scope::Process,
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub dir: String,
    pub scope: Scope,
    /// Microsoft Store uygulama yürütme takma adı (ör. WindowsApps\python.exe).
    pub app_alias: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Conflict {
    /// Yürütülebilir dosya adı (ör. `python.exe`).
    pub name: String,
    /// PATH sırasına göre konumlar; ilki komut çalıştırıldığında kullanılan kopyadır.
    pub locations: Vec<Location>,
}

const EXEC_EXTS: &[&str] = &["exe", "com", "cmd", "bat", "ps1"];

/// Birleşik PATH'te (sistem + kullanıcı) aynı adlı yürütülebilir dosyaları bulur.
/// Dizinler yalnızca listelenir; hiçbir dosya çalıştırılmaz. Sistem dizinlerindeki
/// yüzlerce dosya nedeniyle yalnızca en az bir kopyası sistem dışı bir dizinde olan adlar raporlanır.
pub fn conflicts(p: &dyn EnvProvider) -> AppResult<Vec<Conflict>> {
    let env = effective::fresh_environment(p)?;
    let system_root = env
        .get("SYSTEMROOT")
        .map(|(_, v)| key(v))
        .unwrap_or_else(|| "c:\\windows".into());
    let mut ordered: Vec<(String, Scope)> = Vec::new();
    for scope in [Scope::Machine, Scope::User] {
        if let Some(v) = p.get(scope, "Path")? {
            for e in split(&v.value) {
                let x = expand_references(e.trim(), &effective::lookup(&env));
                let x = x.trim_matches('"').to_string();
                if !x.is_empty() && !ordered.iter().any(|(d, _)| key(d) == key(&x)) {
                    ordered.push((x, scope));
                }
            }
        }
    }
    let mut map: BTreeMap<String, Vec<Location>> = BTreeMap::new();
    for (dir, scope) in &ordered {
        let Ok(rd) = std::fs::read_dir(dir) else {
            continue;
        };
        let app_alias = key(dir).ends_with("\\microsoft\\windowsapps");
        for e in rd.flatten() {
            let path = e.path();
            let Some(ext) = path.extension().and_then(|x| x.to_str()) else {
                continue;
            };
            if !EXEC_EXTS.iter().any(|x| x.eq_ignore_ascii_case(ext)) {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            map.entry(stem.to_lowercase()).or_default().push(Location {
                dir: dir.clone(),
                scope: *scope,
                app_alias,
            });
        }
    }
    let mut out: Vec<Conflict> = map
        .into_iter()
        .filter(|(_, locs)| {
            let mut dirs: Vec<String> = locs.iter().map(|l| key(&l.dir)).collect();
            dirs.dedup();
            dirs.len() > 1 && locs.iter().any(|l| !key(&l.dir).starts_with(&system_root))
        })
        .map(|(name, mut locations)| {
            locations.dedup_by(|a, b| key(&a.dir) == key(&b.dir));
            Conflict { name, locations }
        })
        .collect();
    out.sort_by_key(|c| (!c.locations[0].app_alias, c.name.clone()));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::provider::MemoryProvider;

    #[test]
    fn flags_each_problem_without_changing_anything() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().to_string_lossy().to_string();
        let file = dir.path().join("f.txt");
        std::fs::write(&file, "").unwrap();
        let p = MemoryProvider::new();
        p.seed(Scope::Process, "HOME_DIR", &real, ValueKind::String);
        let value = format!(
            "{real};;{real}\\;Z:\\envora\\missing;relative\\bin;%HOME_DIR%;%NOPE%\\x;\"{real}\";{};\\\\server\\share; {real}",
            file.to_string_lossy()
        );
        p.seed(Scope::User, "Path", &value, ValueKind::Expand);
        p.seed(Scope::Machine, "Path", &real, ValueKind::Expand);
        let r = analyze(&p, Scope::User).unwrap();
        let f = |i: usize| r.entries[i].flags.clone();
        assert!(f(0).contains(&Flag::DuplicateOtherScope));
        assert_eq!(f(1), vec![Flag::Empty]);
        assert!(f(2).contains(&Flag::Duplicate) && r.entries[2].duplicate_of == Some(0));
        assert!(f(3).contains(&Flag::Missing));
        assert!(f(4).contains(&Flag::Relative));
        assert!(
            f(5).contains(&Flag::Duplicate),
            "genişletilmiş hali aynı dizin"
        );
        assert!(f(6).contains(&Flag::UnresolvedReference));
        assert!(f(7).contains(&Flag::Quoted));
        assert!(f(8).contains(&Flag::NotDirectory));
        assert!(f(9).contains(&Flag::Network));
        assert!(f(10).contains(&Flag::Whitespace));
        assert_eq!(r.counts.get("empty"), Some(&1));
        assert_eq!(
            p.get(Scope::User, "Path").unwrap().unwrap().value,
            value,
            "analiz değeri değiştirmemeli"
        );
    }

    #[test]
    fn missing_path_variable() {
        let p = MemoryProvider::new();
        let r = analyze(&p, Scope::User).unwrap();
        assert!(r.name.is_none() && r.entries.is_empty());
    }

    #[test]
    fn finds_shadowed_executables() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        std::fs::write(a.path().join("tool.exe"), "").unwrap();
        std::fs::write(b.path().join("tool.exe"), "").unwrap();
        std::fs::write(b.path().join("only.exe"), "").unwrap();
        let p = MemoryProvider::new();
        p.seed(
            Scope::Machine,
            "Path",
            &a.path().to_string_lossy(),
            ValueKind::Expand,
        );
        p.seed(
            Scope::User,
            "Path",
            &b.path().to_string_lossy(),
            ValueKind::Expand,
        );
        let c = conflicts(&p).unwrap();
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].name, "tool");
        assert_eq!(
            c[0].locations[0].scope,
            Scope::Machine,
            "sistem PATH önce aranır"
        );
    }
}
