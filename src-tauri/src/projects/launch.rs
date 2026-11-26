//! Proje ortamının hesaplanması ve açık kullanıcı onayıyla süreç başlatma.
//!
//! - Ortam: güncel kalıcı ortam (bkz. `env::effective`) + ilişkili profil + projeye özgü değişkenler.
//!   Profildeki ve projedeki tüm değişkenler (kapsam tercihinden bağımsız) yalnızca başlatılan
//!   sürece uygulanır; kalıcı ortama hiçbir şey yazılmaz.
//! - Başlatma: yalnızca seçilen kabuk (pwsh/powershell/cmd) veya kullanıcının tanımladığı özel
//!   program çalıştırılır. Kabuk komut dizesi kullanılmaz; argümanlar ayrı geçirilir.
//!   Proje dizinindeki hiçbir betik otomatik çalıştırılmaz.

use super::model::{CustomCommand, Project};
use crate::env::effective::{self, EnvMap};
use crate::env::inspect::expand_references;
use crate::env::model::name_key;
use crate::env::provider::EnvProvider;
use crate::env::validate::has_percent_reference;
use crate::env::ValueKind;
use crate::error::{AppError, AppResult};
use crate::pathmgr::{entries, PathMod};
use crate::profiles::model::{ProfilePath, ProfileVar, VarAction};
use serde::Serialize;
use std::path::{Path, PathBuf};

fn apply_set(map: &mut EnvMap, vars: &[ProfileVar], paths: &[ProfilePath]) {
    for v in vars {
        match v.action {
            VarAction::Unset => {
                map.remove(&name_key(&v.name));
            }
            VarAction::Set => {
                let raw = v.value.clone().unwrap_or_default();
                let expand = match v.kind {
                    Some(ValueKind::Expand) => true,
                    Some(_) => false,
                    None => has_percent_reference(&raw),
                };
                let value = if expand {
                    expand_references(&raw, &effective::lookup(map))
                } else {
                    raw
                };
                effective::set(map, &v.name, value);
            }
        }
    }
    if !paths.is_empty() {
        // İşlem ortamında PATH girdileri Windows tarafından genişletilmez; burada genişletilir.
        let mods: Vec<PathMod> = paths
            .iter()
            .map(|p| PathMod {
                entry: expand_references(&p.path.entry, &effective::lookup(map)),
                action: p.path.action,
            })
            .collect();
        let current = map.get("PATH").map(|(_, v)| v.clone()).unwrap_or_default();
        effective::set(map, "Path", entries::apply_mods(&current, &mods));
    }
}

/// Projenin başlatma ortamı. `profile` ilişkili profilin tam (çözülmüş) halidir.
pub fn launch_env(
    p: &dyn EnvProvider,
    project: &Project,
    profile: Option<(&[ProfileVar], &[ProfilePath])>,
) -> AppResult<EnvMap> {
    let mut map = effective::fresh_environment(p)?;
    if let Some((vars, paths)) = profile {
        apply_set(&mut map, vars, paths);
    }
    apply_set(&mut map, &project.vars, &project.paths);
    Ok(map)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind {
    Added,
    Changed,
    Removed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvChange {
    pub name: String,
    pub kind: ChangeKind,
    pub sensitive: bool,
    /// Hassas olmayan değişkenlerde yeni değer.
    pub value: Option<String>,
    /// Değer profilden mi projeden mi geliyor?
    pub source: &'static str,
}

/// Başlatma ortamının kalıcı ortamdan farkı (hassas değerler maskeli).
pub fn diff(
    base: &EnvMap,
    launch: &EnvMap,
    profile_vars: &[ProfileVar],
    project: &Project,
) -> Vec<EnvChange> {
    let sensitive = |key: &str| -> bool {
        project
            .vars
            .iter()
            .chain(profile_vars.iter())
            .any(|v| name_key(&v.name) == key && v.sensitive)
    };
    let source = |key: &str| -> &'static str {
        if project.vars.iter().any(|v| name_key(&v.name) == key)
            || (key == "PATH" && !project.paths.is_empty())
        {
            "project"
        } else {
            "profile"
        }
    };
    let mut out = Vec::new();
    for (k, (name, v)) in launch {
        let kind = match base.get(k) {
            None => ChangeKind::Added,
            Some((_, old)) if old != v => ChangeKind::Changed,
            _ => continue,
        };
        let s = sensitive(k);
        out.push(EnvChange {
            name: name.clone(),
            kind,
            sensitive: s,
            value: if s { None } else { Some(v.clone()) },
            source: source(k),
        });
    }
    for (k, (name, _)) in base {
        if !launch.contains_key(k) {
            out.push(EnvChange {
                name: name.clone(),
                kind: ChangeKind::Removed,
                sensitive: sensitive(k),
                value: None,
                source: source(k),
            });
        }
    }
    out.sort_by_key(|c| c.name.to_uppercase());
    out
}

/// PATH ve PATHEXT'e göre bir programı bulur (yalnızca .exe/.com).
pub fn which(program: &str, env: &EnvMap) -> Option<PathBuf> {
    let p = Path::new(program);
    if p.is_absolute() {
        return p.is_file().then(|| p.to_path_buf());
    }
    if program.contains(['\\', '/']) {
        return None; // göreli yollar kabul edilmez
    }
    let path = env.get("PATH").map(|(_, v)| v.as_str()).unwrap_or("");
    let has_ext = p.extension().is_some();
    for dir in path.split(';').filter(|d| !d.trim().is_empty()) {
        let base = Path::new(dir.trim().trim_matches('"'));
        if has_ext {
            let c = base.join(program);
            if c.is_file() {
                return Some(c);
            }
        } else {
            for ext in ["exe", "com"] {
                let c = base.join(format!("{program}.{ext}"));
                if c.is_file() {
                    return Some(c);
                }
            }
        }
    }
    None
}

fn allowed_executable(p: &Path) -> bool {
    p.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("exe") || e.eq_ignore_ascii_case("com"))
}

/// Kabuk adını yürütülebilir dosya yoluna çözer.
pub fn resolve_shell(shell: &str, env: &EnvMap) -> AppResult<PathBuf> {
    let system_root = env
        .get("SYSTEMROOT")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| "C:\\Windows".into());
    let candidate = match shell {
        "cmd" => Some(PathBuf::from(&system_root).join("System32").join("cmd.exe")),
        "powershell" => Some(
            PathBuf::from(&system_root).join(r"System32\WindowsPowerShell\v1.0\powershell.exe"),
        ),
        "pwsh" => which("pwsh", env).or_else(|| {
            let pf = env
                .get("PROGRAMFILES")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "C:\\Program Files".into());
            Some(PathBuf::from(pf).join(r"PowerShell\7\pwsh.exe"))
        }),
        _ => None,
    };
    match candidate {
        Some(p) if p.is_file() => Ok(p),
        _ => Err(AppError::validation("shell_not_found", shell.to_string())),
    }
}

/// Mevcut kabukları listeler.
pub fn available_shells(env: &EnvMap) -> Vec<&'static str> {
    ["pwsh", "powershell", "cmd"]
        .into_iter()
        .filter(|s| resolve_shell(s, env).is_ok())
        .collect()
}

pub enum Target<'a> {
    Shell(&'a str),
    Custom(&'a CustomCommand),
}

/// Çalıştırılacak programı ve argümanları çözer (başlatmadan). Önizleme ve testler için.
pub fn resolve_target(target: &Target, env: &EnvMap) -> AppResult<(PathBuf, Vec<String>)> {
    match target {
        Target::Shell(s) => Ok((resolve_shell(s, env)?, Vec::new())),
        Target::Custom(c) => {
            let program = which(c.program.trim(), env)
                .ok_or_else(|| AppError::validation("program_not_found", c.program.clone()))?;
            if !allowed_executable(&program) {
                return Err(AppError::validation(
                    "program_not_allowed",
                    c.program.clone(),
                ));
            }
            Ok((program, c.args.clone()))
        }
    }
}

/// Süreci yeni bir konsol penceresinde başlatır. Ortam tamamen verilen haritadır.
pub fn spawn(program: &Path, args: &[String], dir: &Path, env: &EnvMap) -> AppResult<u32> {
    if !dir.is_dir() {
        return Err(AppError::validation(
            "directory_not_found",
            dir.to_string_lossy().into_owned(),
        ));
    }
    let mut cmd = std::process::Command::new(program);
    cmd.args(args).current_dir(dir).env_clear();
    for (name, value) in env.values() {
        cmd.env(name, value);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
        cmd.creation_flags(CREATE_NEW_CONSOLE);
    }
    let child = cmd
        .spawn()
        .map_err(|e| AppError::Platform(format!("başlatılamadı: {e}")))?;
    Ok(child.id())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::provider::MemoryProvider;
    use crate::env::Scope;
    use crate::pathmgr::PathAction;
    use crate::projects::model::LaunchConfig;

    fn project(vars: Vec<ProfileVar>, paths: Vec<ProfilePath>) -> Project {
        Project {
            id: 1,
            name: "p".into(),
            identifier: "p".into(),
            description: String::new(),
            directory: String::new(),
            profile_id: None,
            profile_name: None,
            tags: vec![],
            launch: LaunchConfig::default(),
            created_at: String::new(),
            updated_at: String::new(),
            vars,
            paths,
        }
    }

    fn var(name: &str, value: &str, action: VarAction, sensitive: bool) -> ProfileVar {
        ProfileVar {
            name: name.into(),
            value: Some(value.into()),
            sensitive,
            kind: None,
            scope: Scope::Process,
            action,
        }
    }

    #[test]
    fn project_overrides_profile_and_expands() {
        let p = MemoryProvider::new();
        p.seed(
            Scope::Process,
            "USERPROFILE",
            "C:\\Users\\u",
            ValueKind::String,
        );
        p.seed(Scope::Machine, "Path", "C:\\Windows", ValueKind::Expand);
        p.seed(Scope::User, "REMOVE_ME", "x", ValueKind::String);
        let profile_vars = vec![
            var("NODE_ENV", "development", VarAction::Set, false),
            var("API_TOKEN", "t0k", VarAction::Set, true),
        ];
        let profile_paths = vec![ProfilePath {
            path: PathMod {
                entry: "%USERPROFILE%\\node".into(),
                action: PathAction::Prepend,
            },
            scope: Scope::User,
        }];
        let proj = project(
            vec![
                var("NODE_ENV", "test", VarAction::Set, false),
                var("REMOVE_ME", "", VarAction::Unset, false),
                var("DATA", "%USERPROFILE%\\data", VarAction::Set, false),
            ],
            vec![],
        );
        let base = effective::fresh_environment(&p).unwrap();
        let env = launch_env(&p, &proj, Some((&profile_vars, &profile_paths))).unwrap();
        let get = |k: &str| env.get(k).map(|(_, v)| v.as_str());
        assert_eq!(get("NODE_ENV"), Some("test"));
        assert_eq!(get("DATA"), Some("C:\\Users\\u\\data"));
        assert_eq!(
            get("PATH"),
            Some("C:\\Users\\u\\node;C:\\Windows"),
            "PATH girdileri genişletilir"
        );
        assert!(get("REMOVE_ME").is_none());

        let d = diff(&base, &env, &profile_vars, &proj);
        let token = d.iter().find(|c| c.name == "API_TOKEN").unwrap();
        assert!(token.sensitive && token.value.is_none());
        assert!(!serde_json::to_string(&d).unwrap().contains("t0k"));
        assert_eq!(
            d.iter().find(|c| c.name == "REMOVE_ME").unwrap().kind,
            ChangeKind::Removed
        );
        assert_eq!(
            d.iter().find(|c| c.name == "NODE_ENV").unwrap().source,
            "project"
        );
        assert_eq!(
            d.iter().find(|c| c.name == "API_TOKEN").unwrap().source,
            "profile"
        );
    }

    #[test]
    fn custom_programs_must_be_exe() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("tool.exe"), b"").unwrap();
        std::fs::write(dir.path().join("script.bat"), b"").unwrap();
        let mut env = EnvMap::new();
        effective::set(&mut env, "Path", dir.path().to_string_lossy().to_string());
        let ok = CustomCommand {
            program: "tool".into(),
            args: vec!["--x".into()],
        };
        assert!(resolve_target(&Target::Custom(&ok), &env).is_ok());
        let bat = CustomCommand {
            program: dir.path().join("script.bat").to_string_lossy().to_string(),
            args: vec![],
        };
        assert!(matches!(
            resolve_target(&Target::Custom(&bat), &env),
            Err(AppError::Validation {
                code: "program_not_allowed",
                ..
            })
        ));
        let rel = CustomCommand {
            program: ".\\tool.exe".into(),
            args: vec![],
        };
        assert!(
            resolve_target(&Target::Custom(&rel), &env).is_err(),
            "göreli yol kabul edilmez"
        );
        let missing = CustomCommand {
            program: "nope-envora".into(),
            args: vec![],
        };
        assert!(resolve_target(&Target::Custom(&missing), &env).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn shells_resolve_on_windows() {
        let p = crate::env::windows::WindowsProvider::new();
        let env = effective::fresh_environment(&p).unwrap();
        assert!(resolve_shell("cmd", &env).is_ok());
        assert!(available_shells(&env).contains(&"cmd"));
        assert!(resolve_shell("bash", &env).is_err());
    }
}
