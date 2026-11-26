//! Geliştirici aracı keşfi.
//!
//! - Araçlar PATH ve PATHEXT sırasına göre bulunur (cmd'nin aradığı gibi); tüm konumlar listelenir.
//! - Sürüm, mümkün olduğunca **hiçbir şey çalıştırmadan** okunur: yürütülebilir dosyanın Windows
//!   sürüm kaynağından ya da npm tarzı `.cmd` araçlarda yanındaki `package.json` dosyasından.
//! - Yalnızca bu yollar sonuç vermezse ve ayarlarda izin verildiyse `--version` sorgusu çalıştırılır:
//!   yalnızca `.exe` dosyaları, sabit argümanlarla, zaman aşımıyla, pencere açmadan ve girdi vermeden.
//! - Microsoft Store uygulama takma adları (WindowsApps) **asla çalıştırılmaz**.
//! - Hiçbir şey indirilmez veya kurulmaz; PATH değiştirilmez.

mod version;

use crate::env::effective::{self, EnvMap};
use crate::env::provider::EnvProvider;
use crate::error::AppResult;
use crate::pathmgr::entries;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub struct ToolDef {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    /// Aranacak komut adı (uzantısız).
    pub command: &'static str,
    /// `--version` sorgusu argümanları (gerekirse).
    pub version_args: &'static [&'static str],
}

pub static TOOLS: &[ToolDef] = &[
    ToolDef {
        id: "node",
        name: "Node.js",
        category: "javascript",
        command: "node",
        version_args: &["--version"],
    },
    ToolDef {
        id: "npm",
        name: "npm",
        category: "javascript",
        command: "npm",
        version_args: &[],
    },
    ToolDef {
        id: "pnpm",
        name: "pnpm",
        category: "javascript",
        command: "pnpm",
        version_args: &[],
    },
    ToolDef {
        id: "yarn",
        name: "Yarn",
        category: "javascript",
        command: "yarn",
        version_args: &[],
    },
    ToolDef {
        id: "deno",
        name: "Deno",
        category: "javascript",
        command: "deno",
        version_args: &["--version"],
    },
    ToolDef {
        id: "bun",
        name: "Bun",
        category: "javascript",
        command: "bun",
        version_args: &["--version"],
    },
    ToolDef {
        id: "python",
        name: "Python",
        category: "python",
        command: "python",
        version_args: &["--version"],
    },
    ToolDef {
        id: "py",
        name: "Python Launcher (py)",
        category: "python",
        command: "py",
        version_args: &[],
    },
    ToolDef {
        id: "pip",
        name: "pip",
        category: "python",
        command: "pip",
        version_args: &["--version"],
    },
    ToolDef {
        id: "uv",
        name: "uv",
        category: "python",
        command: "uv",
        version_args: &["--version"],
    },
    ToolDef {
        id: "rustc",
        name: "Rust (rustc)",
        category: "rust",
        command: "rustc",
        version_args: &["--version"],
    },
    ToolDef {
        id: "cargo",
        name: "Cargo",
        category: "rust",
        command: "cargo",
        version_args: &["--version"],
    },
    ToolDef {
        id: "rustup",
        name: "rustup",
        category: "rust",
        command: "rustup",
        version_args: &["--version"],
    },
    ToolDef {
        id: "java",
        name: "Java",
        category: "java",
        command: "java",
        version_args: &["-version"],
    },
    ToolDef {
        id: "javac",
        name: "JDK (javac)",
        category: "java",
        command: "javac",
        version_args: &["-version"],
    },
    ToolDef {
        id: "mvn",
        name: "Maven",
        category: "java",
        command: "mvn",
        version_args: &[],
    },
    ToolDef {
        id: "gradle",
        name: "Gradle",
        category: "java",
        command: "gradle",
        version_args: &[],
    },
    ToolDef {
        id: "go",
        name: "Go",
        category: "go",
        command: "go",
        version_args: &["version"],
    },
    ToolDef {
        id: "dotnet",
        name: ".NET",
        category: "dotnet",
        command: "dotnet",
        version_args: &["--version"],
    },
    ToolDef {
        id: "php",
        name: "PHP",
        category: "php",
        command: "php",
        version_args: &["--version"],
    },
    ToolDef {
        id: "composer",
        name: "Composer",
        category: "php",
        command: "composer",
        version_args: &[],
    },
    ToolDef {
        id: "ruby",
        name: "Ruby",
        category: "ruby",
        command: "ruby",
        version_args: &["--version"],
    },
    ToolDef {
        id: "git",
        name: "Git",
        category: "vcs",
        command: "git",
        version_args: &["--version"],
    },
    ToolDef {
        id: "docker",
        name: "Docker",
        category: "containers",
        command: "docker",
        version_args: &["--version"],
    },
    ToolDef {
        id: "wsl",
        name: "WSL",
        category: "containers",
        command: "wsl",
        version_args: &[],
    },
    ToolDef {
        id: "pwsh",
        name: "PowerShell 7",
        category: "shell",
        command: "pwsh",
        version_args: &[],
    },
    ToolDef {
        id: "powershell",
        name: "Windows PowerShell",
        category: "shell",
        command: "powershell",
        version_args: &[],
    },
    ToolDef {
        id: "code",
        name: "Visual Studio Code",
        category: "editor",
        command: "code",
        version_args: &[],
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum VersionSource {
    /// Yürütülebilir dosyanın sürüm kaynağı (hiçbir şey çalıştırılmadı).
    FileResource,
    /// Komutun yanındaki package.json (hiçbir şey çalıştırılmadı).
    Package,
    /// `--version` sorgusu çalıştırıldı.
    Probe,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub path: String,
    pub app_alias: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub found: bool,
    /// PATH sırasına göre tüm konumlar; ilki komut yazıldığında çalışan kopyadır.
    pub locations: Vec<Location>,
    pub version: Option<String>,
    pub version_source: Option<VersionSource>,
    /// İlk konum bir Store takma adıysa (çalıştırılmadı).
    pub app_alias: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub tools: Vec<ToolInfo>,
    pub generated_at: String,
    pub duration_ms: u128,
    pub probed: usize,
}

fn is_app_alias(p: &Path) -> bool {
    entries::key(&p.to_string_lossy()).contains("\\microsoft\\windowsapps\\")
}

/// Komutu PATH ve PATHEXT sırasına göre arar; tüm eşleşmeleri döndürür.
pub fn which_all(command: &str, env: &EnvMap) -> Vec<PathBuf> {
    let path = env.get("PATH").map(|(_, v)| v.as_str()).unwrap_or("");
    let exts: Vec<String> = env
        .get("PATHEXT")
        .map(|(_, v)| {
            v.split(';')
                .filter(|e| !e.is_empty())
                .map(|e| e.to_lowercase())
                .collect()
        })
        .unwrap_or_else(|| vec![".com".into(), ".exe".into(), ".bat".into(), ".cmd".into()]);
    let mut out: Vec<PathBuf> = Vec::new();
    let mut seen_dirs = std::collections::HashSet::new();
    for dir in path
        .split(';')
        .map(|d| d.trim().trim_matches('"'))
        .filter(|d| !d.is_empty())
    {
        if !seen_dirs.insert(entries::key(dir)) {
            continue;
        }
        for ext in &exts {
            let c = Path::new(dir).join(format!("{command}{ext}"));
            if c.is_file() {
                out.push(c);
                break; // aynı dizinde PATHEXT sırasına göre ilk eşleşme çalışır
            }
        }
    }
    out
}

pub struct Options {
    pub probe: bool,
    pub timeout: Duration,
}

pub fn discover(p: &dyn EnvProvider, opts: &Options) -> AppResult<Report> {
    let started = std::time::Instant::now();
    let env = effective::fresh_environment(p)?;
    let mut probed = 0;
    let tools = TOOLS
        .iter()
        .map(|t| {
            let found = which_all(t.command, &env);
            let locations: Vec<Location> = found
                .iter()
                .map(|p| Location {
                    path: p.to_string_lossy().into_owned(),
                    app_alias: is_app_alias(p),
                })
                .collect();
            let first = found.first();
            let app_alias = first.is_some_and(|p| is_app_alias(p));
            let (version, version_source) = match first {
                None => (None, None),
                Some(_) if app_alias => (None, None),
                Some(path) => {
                    let ext = path
                        .extension()
                        .map(|e| e.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    if let Some(v) = version::from_package(path, t.command) {
                        (Some(v), Some(VersionSource::Package))
                    } else if let (true, Some(v)) = (ext == "exe", version::from_resource(path)) {
                        (Some(v), Some(VersionSource::FileResource))
                    } else if opts.probe && ext == "exe" && !t.version_args.is_empty() {
                        probed += 1;
                        match version::probe(path, t.version_args, &env, opts.timeout) {
                            Some(v) => (Some(v), Some(VersionSource::Probe)),
                            None => (None, None),
                        }
                    } else {
                        (None, None)
                    }
                }
            };
            ToolInfo {
                id: t.id,
                name: t.name,
                category: t.category,
                found: first.is_some(),
                locations,
                version,
                version_source,
                app_alias,
            }
        })
        .collect();
    Ok(Report {
        tools,
        generated_at: chrono::Utc::now().to_rfc3339(),
        duration_ms: started.elapsed().as_millis(),
        probed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::provider::MemoryProvider;
    use crate::env::{Scope, ValueKind};

    #[test]
    fn which_follows_path_and_pathext_order() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        std::fs::write(a.path().join("tool.cmd"), "").unwrap();
        std::fs::write(a.path().join("tool.exe"), "").unwrap();
        std::fs::write(b.path().join("tool.exe"), "").unwrap();
        let mut env = EnvMap::new();
        effective::set(
            &mut env,
            "Path",
            format!(
                "{};{};{}",
                a.path().display(),
                b.path().display(),
                a.path().display()
            ),
        );
        effective::set(&mut env, "PATHEXT", ".COM;.EXE;.BAT;.CMD".into());
        let r = which_all("tool", &env);
        assert_eq!(r.len(), 2, "aynı dizin iki kez sayılmaz");
        assert_eq!(r[0], a.path().join("tool.exe"), ".exe, .cmd'den önce gelir");
        assert_eq!(r[1], b.path().join("tool.exe"));
    }

    #[test]
    fn app_aliases_are_never_executed() {
        let root = tempfile::tempdir().unwrap();
        let alias = root.path().join("Microsoft").join("WindowsApps");
        std::fs::create_dir_all(&alias).unwrap();
        std::fs::write(alias.join("python.exe"), "not a real exe").unwrap();
        let p = MemoryProvider::new();
        p.seed(
            Scope::Machine,
            "Path",
            &alias.to_string_lossy(),
            ValueKind::Expand,
        );
        let r = discover(
            &p,
            &Options {
                probe: true,
                timeout: Duration::from_millis(500),
            },
        )
        .unwrap();
        let py = r.tools.iter().find(|t| t.id == "python").unwrap();
        assert!(py.found && py.app_alias);
        assert!(py.version.is_none());
        assert_eq!(r.probed, 0, "takma ad çalıştırılmamalı");
    }

    #[test]
    fn missing_tools_are_reported_as_not_found() {
        let p = MemoryProvider::new();
        let r = discover(
            &p,
            &Options {
                probe: false,
                timeout: Duration::from_millis(100),
            },
        )
        .unwrap();
        assert!(r.tools.iter().all(|t| !t.found));
        assert_eq!(r.tools.len(), TOOLS.len());
    }
}
