//! Proje deposu.

use super::model::*;
use crate::db::Db;
use crate::env::inspect::Overrides;
use crate::env::model::name_key;
use crate::envset::{self, Owner};
use crate::error::{AppError, AppResult};
use rusqlite::{params, OptionalExtension};
use std::path::{Path, PathBuf};

pub const MAX_NAME: usize = 100;
pub const MAX_ARGS: usize = 64;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub fn validate_name(name: &str) -> AppResult<String> {
    let n = name.trim();
    if n.is_empty() || n.chars().count() > MAX_NAME || n.chars().any(char::is_control) {
        return Err(AppError::validation("invalid_project_name", ""));
    }
    Ok(n.to_string())
}

/// Addan küçük harfli, ASCII bir tanımlayıcı üretir (ör. "Ivily Envora" → "ivily-envora").
pub fn slugify(name: &str) -> String {
    let map = |c: char| match c {
        'ç' | 'Ç' => 'c',
        'ğ' | 'Ğ' => 'g',
        'ı' | 'I' | 'İ' | 'i' => 'i',
        'ö' | 'Ö' => 'o',
        'ş' | 'Ş' => 's',
        'ü' | 'Ü' => 'u',
        c => c.to_ascii_lowercase(),
    };
    let mut out = String::new();
    for c in name.chars().map(map) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let out = out.trim_matches('-').chars().take(64).collect::<String>();
    if out.is_empty() {
        "project".into()
    } else {
        out
    }
}

pub fn validate_identifier(id: &str) -> AppResult<()> {
    let ok = !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !id.starts_with('-');
    if ok {
        Ok(())
    } else {
        Err(AppError::validation("invalid_identifier", id.to_string()))
    }
}

/// Proje dizinini doğrular: mutlak, var olan bir dizin olmalı. Kanonik yolu döndürür.
pub fn validate_directory(dir: &str) -> AppResult<PathBuf> {
    let p = Path::new(dir.trim());
    if dir.trim().is_empty() || !p.is_absolute() {
        return Err(AppError::validation("invalid_directory", dir.to_string()));
    }
    if !p.is_dir() {
        return Err(AppError::validation("directory_not_found", dir.to_string()));
    }
    let canon = std::fs::canonicalize(p)?;
    // `\\?\C:\x` biçimini kullanıcıya gösterilecek `C:\x` biçimine çevir.
    let s = canon.to_string_lossy();
    Ok(PathBuf::from(
        s.strip_prefix(r"\\?\").unwrap_or(&s).to_string(),
    ))
}

fn validate_launch(l: &LaunchConfig) -> AppResult<()> {
    if let Some(shell) = &l.shell {
        if !["pwsh", "powershell", "cmd"].contains(&shell.as_str()) {
            return Err(AppError::validation("invalid_shell", shell.clone()));
        }
    }
    if let Some(c) = &l.custom {
        if c.program.trim().is_empty() || c.program.contains('\0') || c.program.len() > 1024 {
            return Err(AppError::validation("invalid_program", ""));
        }
        if c.args.len() > MAX_ARGS || c.args.iter().any(|a| a.contains('\0') || a.len() > 4096) {
            return Err(AppError::validation("invalid_args", ""));
        }
    }
    Ok(())
}

fn taken(db: &Db, column: &str, value: &str, except: Option<i64>) -> AppResult<bool> {
    // `column` yalnızca bu dosyadaki sabit değerlerden gelir.
    db.with(|c| {
        let id: Option<i64> = c
            .query_row(
                &format!("SELECT id FROM projects WHERE {column} = ?1 COLLATE NOCASE"),
                [value],
                |r| r.get(0),
            )
            .optional()?;
        Ok(id.is_some_and(|id| Some(id) != except))
    })
}

fn unique_identifier(db: &Db, base: &str, except: Option<i64>) -> AppResult<String> {
    let mut candidate = base.to_string();
    let mut n = 2;
    while taken(db, "identifier", &candidate, except)? {
        candidate = format!("{}-{n}", base.chars().take(60).collect::<String>());
        n += 1;
    }
    Ok(candidate)
}

struct Clean {
    name: String,
    identifier: String,
    directory: String,
    tags: Vec<String>,
}

fn clean(db: &Db, input: &ProjectInput, except: Option<i64>) -> AppResult<Clean> {
    let name = validate_name(&input.name)?;
    if taken(db, "name", &name, except)? {
        return Err(AppError::Conflict(name));
    }
    let identifier = match input
        .identifier
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(id) => {
            validate_identifier(id)?;
            if taken(db, "identifier", id, except)? {
                return Err(AppError::Conflict(id.to_string()));
            }
            id.to_string()
        }
        None => unique_identifier(db, &slugify(&name), except)?,
    };
    let directory = validate_directory(&input.directory)?
        .to_string_lossy()
        .into_owned();
    validate_launch(&input.launch)?;
    if let Some(pid) = input.profile_id {
        let exists: bool = db.with(|c| {
            Ok(c.query_row(
                "SELECT EXISTS(SELECT 1 FROM profiles WHERE id = ?1)",
                [pid],
                |r| r.get(0),
            )?)
        })?;
        if !exists {
            return Err(AppError::NotFound(format!("profile {pid}")));
        }
    }
    Ok(Clean {
        name,
        identifier,
        directory,
        tags: envset::clean_tags(&input.tags)?,
    })
}

pub fn create(db: &Db, input: &ProjectInput, overrides: &Overrides) -> AppResult<i64> {
    let c = clean(db, input, None)?;
    let paths = envset::clean_paths(&input.paths)?;
    let rows = envset::encrypt_vars(envset::resolve_vars(&input.vars, &[], overrides)?)?;
    let launch =
        serde_json::to_string(&input.launch).map_err(|e| AppError::Internal(e.to_string()))?;
    let ts = now();
    db.tx(|tx| {
        tx.execute(
            "INSERT INTO projects(name, identifier, description, directory, profile_id, tags, launch, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
            params![
                c.name,
                c.identifier,
                input.description.trim(),
                c.directory,
                input.profile_id,
                serde_json::to_string(&c.tags).unwrap_or_else(|_| "[]".into()),
                launch,
                ts
            ],
        )?;
        let id = tx.last_insert_rowid();
        envset::write(tx, Owner::Project, id, &rows, &paths)?;
        Ok(id)
    })
}

pub fn update(db: &Db, id: i64, input: &ProjectInput, overrides: &Overrides) -> AppResult<()> {
    let current = load_full(db, id)?;
    let c = clean(db, input, Some(id))?;
    let paths = envset::clean_paths(&input.paths)?;
    let rows = envset::encrypt_vars(envset::resolve_vars(&input.vars, &current.vars, overrides)?)?;
    let launch =
        serde_json::to_string(&input.launch).map_err(|e| AppError::Internal(e.to_string()))?;
    db.tx(|tx| {
        tx.execute(
            "UPDATE projects SET name = ?2, identifier = ?3, description = ?4, directory = ?5, profile_id = ?6,
             tags = ?7, launch = ?8, updated_at = ?9 WHERE id = ?1",
            params![
                id,
                c.name,
                c.identifier,
                input.description.trim(),
                c.directory,
                input.profile_id,
                serde_json::to_string(&c.tags).unwrap_or_else(|_| "[]".into()),
                launch,
                now()
            ],
        )?;
        envset::write(tx, Owner::Project, id, &rows, &paths)?;
        Ok(())
    })
}

pub fn load_full(db: &Db, id: i64) -> AppResult<Project> {
    type Head = (
        String,
        String,
        String,
        String,
        Option<i64>,
        Option<String>,
        String,
        String,
        String,
        String,
    );
    let head: Option<Head> = db.with(|c| {
        Ok(c.query_row(
            "SELECT p.name, p.identifier, p.description, p.directory, p.profile_id, f.name, p.tags, p.launch,
                    p.created_at, p.updated_at
             FROM projects p LEFT JOIN profiles f ON f.id = p.profile_id WHERE p.id = ?1",
            [id],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                    r.get(8)?,
                    r.get(9)?,
                ))
            },
        )
        .optional()?)
    })?;
    let (
        name,
        identifier,
        description,
        directory,
        profile_id,
        profile_name,
        tags,
        launch,
        created_at,
        updated_at,
    ) = head.ok_or_else(|| AppError::NotFound(format!("project {id}")))?;
    Ok(Project {
        id,
        name,
        identifier,
        description,
        directory,
        profile_id,
        profile_name,
        tags: serde_json::from_str(&tags).unwrap_or_default(),
        launch: serde_json::from_str(&launch).unwrap_or_default(),
        created_at,
        updated_at,
        vars: envset::read_vars(db, Owner::Project, id)?,
        paths: envset::read_paths(db, Owner::Project, id)?,
    })
}

pub fn masked(mut p: Project) -> Project {
    envset::mask_vars(&mut p.vars);
    p
}

pub fn list(db: &Db) -> AppResult<Vec<ProjectSummary>> {
    db.with(|c| {
        let mut st = c.prepare(
            "SELECT p.id, p.name, p.identifier, p.description, p.directory, p.profile_id, f.name, p.tags, p.updated_at,
               (SELECT COUNT(*) FROM project_vars v WHERE v.project_id = p.id)
             FROM projects p LEFT JOIN profiles f ON f.id = p.profile_id ORDER BY p.name COLLATE NOCASE",
        )?;
        let rows = st.query_map([], |r| {
            let tags: String = r.get(7)?;
            let directory: String = r.get(4)?;
            Ok(ProjectSummary {
                id: r.get(0)?,
                name: r.get(1)?,
                identifier: r.get(2)?,
                description: r.get(3)?,
                directory_exists: Path::new(&directory).is_dir(),
                directory,
                profile_id: r.get(5)?,
                profile_name: r.get(6)?,
                tags: serde_json::from_str(&tags).unwrap_or_default(),
                updated_at: r.get(8)?,
                var_count: r.get(9)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    })
}

pub fn delete(db: &Db, id: i64) -> AppResult<()> {
    let n = db.with(|c| Ok(c.execute("DELETE FROM projects WHERE id = ?1", [id])?))?;
    if n == 0 {
        return Err(AppError::NotFound(format!("project {id}")));
    }
    Ok(())
}

pub fn reveal_var(db: &Db, id: i64, name: &str) -> AppResult<String> {
    load_full(db, id)?
        .vars
        .into_iter()
        .find(|v| name_key(&v.name) == name_key(name))
        .and_then(|v| v.value)
        .ok_or_else(|| AppError::NotFound(name.to_string()))
}

/// Proje kökünde bilinen işaret dosyalarının varlığını denetler. Dosyalar **okunmaz** ve
/// hiçbir şey çalıştırılmaz; yalnızca var olup olmadıklarına bakılır.
pub fn detect_markers(dir: &Path) -> Vec<&'static str> {
    const MARKERS: &[(&str, &str)] = &[
        ("package.json", "node"),
        ("pnpm-lock.yaml", "pnpm"),
        ("yarn.lock", "yarn"),
        ("pyproject.toml", "python"),
        ("requirements.txt", "python"),
        ("Cargo.toml", "rust"),
        ("go.mod", "go"),
        ("pom.xml", "java"),
        ("build.gradle", "java"),
        ("build.gradle.kts", "java"),
        ("composer.json", "php"),
        ("Gemfile", "ruby"),
        ("Dockerfile", "docker"),
        ("docker-compose.yml", "docker"),
        ("compose.yaml", "docker"),
        ("global.json", "dotnet"),
        (".env", "dotenv"),
    ];
    let mut out: Vec<&'static str> = Vec::new();
    for (file, tag) in MARKERS {
        if dir.join(file).is_file() && !out.contains(tag) {
            out.push(tag);
        }
    }
    let dotnet = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten().any(|e| {
                e.path().extension().is_some_and(|x| {
                    ["csproj", "sln", "fsproj"]
                        .iter()
                        .any(|k| x.eq_ignore_ascii_case(k))
                })
            })
        })
        .unwrap_or(false);
    if dotnet && !out.contains(&"dotnet") {
        out.push("dotnet");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::Scope;
    use crate::profiles::model::{VarAction, VarInput};

    fn input(name: &str, dir: &str) -> ProjectInput {
        ProjectInput {
            name: name.into(),
            identifier: None,
            description: String::new(),
            directory: dir.into(),
            profile_id: None,
            tags: vec![],
            launch: LaunchConfig::default(),
            vars: vec![VarInput {
                name: "DB_PASSWORD".into(),
                value: Some("p4ss".into()),
                keep: false,
                previous_name: None,
                sensitive: None,
                kind: None,
                scope: Scope::Process,
                action: VarAction::Set,
            }],
            paths: vec![],
        }
    }

    #[test]
    fn slugs() {
        assert_eq!(slugify("Ivily Envora"), "ivily-envora");
        assert_eq!(slugify("Şirket Ön Yüzü!"), "sirket-on-yuzu");
        assert_eq!(slugify("***"), "project");
        assert!(validate_identifier("a-b-1").is_ok());
        assert!(validate_identifier("A b").is_err());
        assert!(validate_identifier("-a").is_err());
    }

    #[test]
    fn create_validates_directory_and_identifier() {
        let db = Db::open_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path().to_string_lossy().to_string();
        let o = Overrides::new();
        assert!(create(&db, &input("A", "relative\\dir"), &o).is_err());
        assert!(create(&db, &input("A", "Z:\\definitely\\missing\\envora"), &o).is_err());
        let id = create(&db, &input("Web App", &d), &o).unwrap();
        let id2 = create(&db, &input("Web-App", &d), &o).unwrap();
        let p = load_full(&db, id).unwrap();
        assert_eq!(p.identifier, "web-app");
        assert_eq!(load_full(&db, id2).unwrap().identifier, "web-app-2");
        assert!(p.vars[0].sensitive);
        assert!(masked(p).vars[0].value.is_none());
        assert_eq!(reveal_var(&db, id, "db_password").unwrap(), "p4ss");
        assert!(matches!(
            create(&db, &input("web app", &d), &o),
            Err(AppError::Conflict(_))
        ));
        let s = &list(&db).unwrap()[0];
        assert!(s.directory_exists);
    }

    #[test]
    fn rejects_bad_launch_config() {
        let db = Db::open_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let mut i = input("X", &dir.path().to_string_lossy());
        i.launch.shell = Some("bash -c rm".into());
        assert!(create(&db, &i, &Overrides::new()).is_err());
    }

    #[test]
    fn markers_are_detected_without_reading() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("package.json"), "{}").unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        std::fs::write(dir.path().join("app.csproj"), "").unwrap();
        let m = detect_markers(dir.path());
        assert_eq!(m, vec!["node", "rust", "dotnet"]);
    }
}
