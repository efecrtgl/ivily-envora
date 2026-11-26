//! İçe aktarma: ayrıştırma → önizleme (maskeli) → kullanıcının seçtiği hedefe uygulama.
//!
//! Ayrıştırılan içerik `ImportCache` içinde rastgele olmayan ancak tahmin edilmesi gereksiz,
//! süreye bağlı bir belirteçle tutulur. Değerler ön uca gönderilmez; değişkenleri ortama
//! uygulayan planlar `fromImport` işlemleriyle değerleri arka uçta bu önbellekten alır.

use super::*;
use crate::db::Db;
use crate::env::inspect::Overrides;
use crate::env::model::name_key;
use crate::env::plan::{ChangePlan, Op};
use crate::env::provider::EnvProvider;
use crate::env::validate::{validate_name, validate_value};
use crate::pathmgr::{entries, PathAction};
use crate::profiles::model::{ProfileInput, ProfilePath, VarInput};
use crate::profiles::store as profiles;
use crate::projects::model::{CustomCommand, ProjectInput};
use crate::projects::store as projects;
use crate::security::classify;
use crate::snapshots::store::{self as snapshots, NewSnapshot, SnapshotItem};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum Bundle {
    Variables(Vec<VarDoc>),
    Profile(ProfileDoc),
    Project(ProjectDoc),
    Snapshot(SnapshotDoc),
    Path(PathDoc),
}

impl Bundle {
    pub fn kind(&self) -> DocKind {
        match self {
            Bundle::Variables(_) => DocKind::Variables,
            Bundle::Profile(_) => DocKind::Profile,
            Bundle::Project(_) => DocKind::Project,
            Bundle::Snapshot(_) => DocKind::Snapshot,
            Bundle::Path(_) => DocKind::Path,
        }
    }
    fn vars(&self) -> &[VarDoc] {
        match self {
            Bundle::Variables(v) => v,
            Bundle::Profile(p) => &p.vars,
            Bundle::Project(p) => &p.vars,
            _ => &[],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportWarning {
    pub line: Option<usize>,
    /// i18n: `importer.warn.<code>`
    pub code: String,
    pub target: Option<String>,
}

fn warn(line: Option<usize>, code: &str, target: Option<&str>) -> ImportWarning {
    ImportWarning {
        line,
        code: code.to_string(),
        target: target.map(String::from),
    }
}

/// Metni biçimine göre ayrıştırır. Hiçbir içerik çalıştırılmaz veya genişletilmez.
pub fn parse(text: &str, format: FileFormat) -> AppResult<(Bundle, Vec<ImportWarning>)> {
    let mut warnings = Vec::new();
    let bundle = match format {
        FileFormat::Json => {
            let doc: Document = serde_json::from_str(text)?;
            if doc.format != FORMAT {
                return Err(AppError::validation("not_envora_document", ""));
            }
            if doc.version > VERSION {
                return Err(AppError::validation(
                    "unsupported_version",
                    doc.version.to_string(),
                ));
            }
            match doc.kind {
                DocKind::Variables => {
                    #[derive(Deserialize)]
                    struct D {
                        variables: Vec<VarDoc>,
                    }
                    Bundle::Variables(serde_json::from_value::<D>(doc.data)?.variables)
                }
                DocKind::Profile => Bundle::Profile(serde_json::from_value(doc.data)?),
                DocKind::Project => Bundle::Project(serde_json::from_value(doc.data)?),
                DocKind::Snapshot => Bundle::Snapshot(serde_json::from_value(doc.data)?),
                DocKind::Path => Bundle::Path(serde_json::from_value(doc.data)?),
            }
        }
        FileFormat::Csv => {
            let rows = csv::parse(text)?;
            let mut it = rows.into_iter();
            let header: Vec<String> = it
                .next()
                .unwrap_or_default()
                .into_iter()
                .map(|h| h.trim().to_lowercase())
                .collect();
            let col = |n: &str| header.iter().position(|h| h == n);
            let (Some(ni), Some(vi)) = (col("name"), col("value")) else {
                return Err(AppError::validation("invalid_csv", "name,value"));
            };
            let (si, ki) = (col("scope"), col("kind"));
            let mut vars = Vec::new();
            for (i, r) in it.enumerate() {
                let get = |c: Option<usize>| c.and_then(|c| r.get(c)).map(|s| s.trim().to_string());
                let Some(name) = get(Some(ni)).filter(|s| !s.is_empty()) else {
                    warnings.push(warn(Some(i + 2), "noName", None));
                    continue;
                };
                vars.push(VarDoc {
                    name,
                    value: r.get(vi).cloned(),
                    scope: get(si).and_then(|s| Scope::parse(&s)),
                    kind: get(ki).and_then(|k| ValueKind::parse(&k)),
                    sensitive: false,
                    excluded: false,
                    action: VarAction::Set,
                });
            }
            Bundle::Variables(vars)
        }
        FileFormat::Env => {
            let p = dotenv::parse(text);
            warnings.extend(
                p.warnings
                    .into_iter()
                    .map(|w| warn(Some(w.line), w.code, None)),
            );
            Bundle::Variables(
                p.entries
                    .into_iter()
                    .map(|e| VarDoc {
                        name: e.name,
                        value: Some(e.value),
                        scope: None,
                        kind: None,
                        sensitive: false,
                        excluded: false,
                        action: VarAction::Set,
                    })
                    .collect(),
            )
        }
        FileFormat::Text => Bundle::Path(PathDoc {
            scope: None,
            entries: text
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .flat_map(|l| {
                    l.split(';')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                })
                .collect(),
        }),
    };
    // Yinelenen adlar: son tanım geçerli olur (yaygın .env davranışı).
    let mut seen = HashMap::new();
    for (i, v) in bundle.vars().iter().enumerate() {
        if let Some(prev) = seen.insert(name_key(&v.name), i) {
            let _ = prev;
            warnings.push(warn(None, "duplicateName", Some(&v.name)));
        }
    }
    Ok((bundle, warnings))
}

/// Yinelenen adlardan yalnızca sonuncusunu bırakır.
fn dedup_last(vars: &[VarDoc]) -> Vec<VarDoc> {
    let mut last: HashMap<String, usize> = HashMap::new();
    for (i, v) in vars.iter().enumerate() {
        last.insert(name_key(&v.name), i);
    }
    vars.iter()
        .enumerate()
        .filter(|(i, v)| last[&name_key(&v.name)] == *i)
        .map(|(_, v)| v.clone())
        .collect()
}

// ---------- Önbellek ----------

const TTL: Duration = Duration::from_secs(30 * 60);
const MAX_ENTRIES: usize = 8;

#[derive(Default)]
pub struct ImportCache {
    map: Mutex<HashMap<String, (Instant, Bundle)>>,
    seq: Mutex<u64>,
}

impl ImportCache {
    pub fn insert(&self, bundle: Bundle) -> String {
        let mut map = self.map.lock();
        map.retain(|_, (t, _)| t.elapsed() < TTL);
        if map.len() >= MAX_ENTRIES {
            if let Some(oldest) = map
                .iter()
                .min_by_key(|(_, (t, _))| *t)
                .map(|(k, _)| k.clone())
            {
                map.remove(&oldest);
            }
        }
        let mut seq = self.seq.lock();
        *seq += 1;
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let token = format!("imp-{:x}-{:x}", nanos, *seq);
        map.insert(token.clone(), (Instant::now(), bundle));
        token
    }

    pub fn get(&self, token: &str) -> AppResult<Bundle> {
        let map = self.map.lock();
        match map.get(token) {
            Some((t, b)) if t.elapsed() < TTL => Ok(b.clone()),
            _ => Err(AppError::validation("import_expired", "")),
        }
    }

    pub fn remove(&self, token: &str) {
        self.map.lock().remove(token);
    }
}

// ---------- Önizleme ----------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewVar {
    pub name: String,
    /// Hassas veya hariç tutulmuş değişkenlerde `None`.
    pub value: Option<String>,
    pub sensitive: bool,
    pub excluded: bool,
    pub action: VarAction,
    pub scope: Option<Scope>,
    /// Geçersizse hata kodu.
    pub invalid: Option<String>,
    /// Adın bulunduğu kalıcı kapsamlar (çakışma bilgisi).
    pub exists_in: Vec<Scope>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub token: String,
    pub format: FileFormat,
    pub kind: DocKind,
    pub name: Option<String>,
    pub vars: Vec<PreviewVar>,
    pub path_entries: Vec<String>,
    pub snapshot_items: usize,
    pub warnings: Vec<ImportWarning>,
    pub contains_secrets: bool,
    pub directory: Option<String>,
    pub directory_exists: bool,
    pub launch_custom: Option<CustomCommand>,
    pub profile_name: Option<String>,
    pub name_taken: bool,
}

pub fn preview(
    db: &Db,
    p: &dyn EnvProvider,
    overrides: &Overrides,
    token: String,
    format: FileFormat,
    bundle: &Bundle,
    warnings: Vec<ImportWarning>,
) -> AppResult<Preview> {
    let mut existing: HashMap<String, Vec<Scope>> = HashMap::new();
    for scope in Scope::PERSISTENT {
        for v in p.list(scope)? {
            existing.entry(name_key(&v.name)).or_default().push(scope);
        }
    }
    let vars: Vec<PreviewVar> = dedup_last(bundle.vars())
        .into_iter()
        .map(|v| {
            let invalid = validate_name(&v.name)
                .err()
                .or_else(|| v.value.as_deref().and_then(|x| validate_value(x).err()))
                .map(|e| e.code().to_string());
            let sensitive = v.sensitive
                || classify::resolve(
                    &v.name,
                    v.value.as_deref().unwrap_or(""),
                    overrides.get(&name_key(&v.name)).copied(),
                )
                .sensitive;
            PreviewVar {
                exists_in: existing
                    .get(&name_key(&v.name))
                    .cloned()
                    .unwrap_or_default(),
                value: if sensitive || v.excluded {
                    None
                } else {
                    v.value.clone()
                },
                sensitive,
                excluded: v.excluded || (v.action == VarAction::Set && v.value.is_none()),
                action: v.action,
                scope: v.scope,
                invalid,
                name: v.name,
            }
        })
        .collect();
    let contains_secrets = bundle.vars().iter().any(|v| v.value.is_some())
        && vars.iter().any(|v| v.sensitive && !v.excluded);
    let mut pv = Preview {
        token,
        format,
        kind: bundle.kind(),
        name: None,
        vars,
        path_entries: Vec::new(),
        snapshot_items: 0,
        warnings,
        contains_secrets,
        directory: None,
        directory_exists: false,
        launch_custom: None,
        profile_name: None,
        name_taken: false,
    };
    let taken = |table: &str, name: &str| -> AppResult<bool> {
        db.with(|c| {
            Ok(c.query_row(
                &format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE name = ?1 COLLATE NOCASE)"),
                [name],
                |r| r.get(0),
            )?)
        })
    };
    match bundle {
        Bundle::Profile(d) => {
            pv.name = Some(d.name.clone());
            pv.name_taken = taken("profiles", &d.name)?;
            pv.path_entries = d.paths.iter().map(|x| x.path.entry.clone()).collect();
        }
        Bundle::Project(d) => {
            pv.name = Some(d.name.clone());
            pv.name_taken = taken("projects", &d.name)?;
            pv.directory = Some(d.directory.clone());
            pv.directory_exists = Path::new(&d.directory).is_dir();
            pv.launch_custom = d.launch.custom.clone();
            pv.profile_name = d.profile_name.clone();
            pv.path_entries = d.paths.iter().map(|x| x.path.entry.clone()).collect();
        }
        Bundle::Snapshot(d) => {
            pv.name = Some(d.name.clone());
            pv.snapshot_items = d.items.len();
            pv.contains_secrets = d.items.iter().any(|i| i.sensitive && i.value.is_some());
        }
        Bundle::Path(d) => pv.path_entries = d.entries.clone(),
        Bundle::Variables(_) => {}
    }
    Ok(pv)
}

// ---------- Uygulama ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConflictPolicy {
    /// Var olan değişkenlerin üzerine yaz (önizleme ve onayla).
    Overwrite,
    /// Var olanları atla, yalnızca yenileri ekle.
    SkipExisting,
}

/// Seçilen değişkenleri hedef kapsama uygulayacak plan. Değerler planda bulunmaz (`fromImport`).
pub fn variables_plan(
    p: &dyn EnvProvider,
    token: &str,
    bundle: &Bundle,
    scope: Scope,
    names: Option<&[String]>,
    policy: ConflictPolicy,
) -> AppResult<ChangePlan> {
    let wanted: Option<std::collections::HashSet<String>> =
        names.map(|n| n.iter().map(|x| name_key(x)).collect());
    let existing: std::collections::HashSet<String> = p
        .list(scope)?
        .into_iter()
        .map(|v| name_key(&v.name))
        .collect();
    let ops: Vec<Op> = dedup_last(bundle.vars())
        .into_iter()
        .filter(|v| {
            wanted
                .as_ref()
                .is_none_or(|w| w.contains(&name_key(&v.name)))
        })
        .filter(|v| !v.excluded && validate_name(&v.name).is_ok())
        .filter(|v| policy == ConflictPolicy::Overwrite || !existing.contains(&name_key(&v.name)))
        .filter_map(|v| match (v.action, &v.value) {
            (VarAction::Unset, _) => Some(Op::Delete { name: v.name }),
            (VarAction::Set, Some(_)) => Some(Op::FromImport {
                name: v.name,
                token: token.to_string(),
            }),
            (VarAction::Set, None) => None,
        })
        .collect();
    if ops.is_empty() {
        return Err(AppError::validation("nothing_to_import", ""));
    }
    Ok(ChangePlan { scope, ops })
}

/// `fromImport` işlemlerinin değerini önbellekten alır.
pub fn import_value(bundle: &Bundle, name: &str) -> AppResult<(String, Option<ValueKind>)> {
    dedup_last(bundle.vars())
        .into_iter()
        .find(|v| name_key(&v.name) == name_key(name) && !v.excluded)
        .and_then(|v| v.value.map(|x| (x, v.kind)))
        .ok_or_else(|| AppError::NotFound(name.to_string()))
}

fn var_inputs(vars: &[VarDoc], default_scope: Scope) -> (Vec<VarInput>, usize) {
    let mut skipped = 0;
    let inputs = dedup_last(vars)
        .into_iter()
        .filter(|v| {
            let keep = !(v.excluded || (v.action == VarAction::Set && v.value.is_none()));
            if !keep {
                skipped += 1;
            }
            keep
        })
        .map(|v| VarInput {
            name: v.name,
            value: v.value,
            keep: false,
            previous_name: None,
            sensitive: if v.sensitive { Some(true) } else { None },
            kind: v.kind,
            scope: v.scope.unwrap_or(default_scope),
            action: v.action,
        })
        .collect();
    (inputs, skipped)
}

fn paths(entries: &[PathDocEntry]) -> Vec<ProfilePath> {
    entries
        .iter()
        .map(|e| ProfilePath {
            path: e.path.clone(),
            scope: e.scope,
        })
        .collect()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResult {
    pub id: i64,
    /// Değeri olmadığı (hariç tutulduğu) için atlanan değişkenler.
    pub skipped: usize,
}

pub fn create_profile(
    db: &Db,
    bundle: &Bundle,
    name: &str,
    overrides: &Overrides,
) -> AppResult<CreateResult> {
    let Bundle::Profile(d) = bundle else {
        return Err(AppError::validation("wrong_kind", ""));
    };
    let (vars, skipped) = var_inputs(&d.vars, Scope::User);
    let input = ProfileInput {
        name: name.to_string(),
        description: d.description.clone(),
        category: d.category.clone(),
        tags: d.tags.clone(),
        metadata: Some(serde_json::json!({ "importedAt": chrono::Utc::now().to_rfc3339() })),
        vars,
        paths: paths(&d.paths),
    };
    Ok(CreateResult {
        id: profiles::create(db, &input, overrides)?,
        skipped,
    })
}

/// Proje içe aktarma. Özel başlatma komutu yalnızca `keep_launch` açıkça istenirse korunur.
pub fn create_project(
    db: &Db,
    bundle: &Bundle,
    name: &str,
    directory: &str,
    keep_launch: bool,
    overrides: &Overrides,
) -> AppResult<CreateResult> {
    let Bundle::Project(d) = bundle else {
        return Err(AppError::validation("wrong_kind", ""));
    };
    let (vars, skipped) = var_inputs(&d.vars, Scope::Process);
    let profile_id = match &d.profile_name {
        Some(n) => db.with(|c| {
            use rusqlite::OptionalExtension;
            Ok(c.query_row(
                "SELECT id FROM profiles WHERE name = ?1 COLLATE NOCASE",
                [n],
                |r| r.get(0),
            )
            .optional()?)
        })?,
        None => None,
    };
    let mut launch = d.launch.clone();
    if !keep_launch {
        launch.custom = None;
    }
    let input = ProjectInput {
        name: name.to_string(),
        identifier: None,
        description: d.description.clone(),
        directory: directory.to_string(),
        profile_id,
        tags: d.tags.clone(),
        launch,
        vars,
        paths: paths(&d.paths),
    };
    Ok(CreateResult {
        id: projects::create(db, &input, overrides)?,
        skipped,
    })
}

pub fn create_snapshot(db: &Db, bundle: &Bundle, name: &str) -> AppResult<CreateResult> {
    let Bundle::Snapshot(d) = bundle else {
        return Err(AppError::validation("wrong_kind", ""));
    };
    let mut skipped = 0;
    let mut items = Vec::new();
    for i in &d.items {
        if validate_name(&i.name).is_err()
            || i.value
                .as_deref()
                .is_some_and(|v| validate_value(v).is_err())
            || (i.present && i.value.is_none())
        {
            skipped += 1;
            continue;
        }
        items.push(SnapshotItem {
            scope: i.scope,
            name: i.name.clone(),
            kind: i.kind,
            present: i.present,
            sensitive: i.sensitive,
            value: i.value.clone(),
        });
    }
    let source = if ["environment", "profile", "project"].contains(&d.source.as_str()) {
        d.source.as_str()
    } else {
        "environment"
    };
    let desc = d.description.clone().unwrap_or_default();
    let id = snapshots::create(
        db,
        NewSnapshot {
            name,
            description: Some(desc.as_str()).filter(|s| !s.is_empty()),
            category: "manual",
            source,
        },
        &items,
    )?;
    Ok(CreateResult { id, skipped })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PathMode {
    /// Eksik girdileri sona ekle.
    AppendMissing,
    /// Eksik girdileri başa ekle.
    PrependMissing,
    /// PATH'i içe aktarılan liste ile değiştir.
    Replace,
}

pub fn path_plan(
    p: &dyn EnvProvider,
    bundle: &Bundle,
    scope: Scope,
    mode: PathMode,
) -> AppResult<ChangePlan> {
    let Bundle::Path(d) = bundle else {
        return Err(AppError::validation("wrong_kind", ""));
    };
    for e in &d.entries {
        if e.contains('\0') || e.contains(';') {
            return Err(AppError::validation("invalid_path_entry", e.clone()));
        }
    }
    let current = p.get(scope, "Path")?;
    let before = current
        .as_ref()
        .map(|v| v.value.clone())
        .unwrap_or_default();
    let after = match mode {
        PathMode::Replace => d.entries.join(";"),
        PathMode::AppendMissing | PathMode::PrependMissing => {
            let action = if mode == PathMode::AppendMissing {
                PathAction::Append
            } else {
                PathAction::Prepend
            };
            let existing: std::collections::HashSet<String> = entries::split(&before)
                .iter()
                .map(|e| entries::key(e))
                .collect();
            let mods: Vec<PathMod> = d
                .entries
                .iter()
                .filter(|e| !existing.contains(&entries::key(e)))
                .map(|e| PathMod {
                    entry: e.clone(),
                    action,
                })
                .collect();
            entries::apply_mods(&before, &mods)
        }
    };
    let name = current
        .as_ref()
        .map(|v| v.name.clone())
        .unwrap_or_else(|| "Path".into());
    let kind = current
        .map(|v| v.kind)
        .filter(|k| *k != ValueKind::Other)
        .unwrap_or(ValueKind::Expand);
    Ok(ChangePlan {
        scope,
        ops: vec![Op::Set {
            name,
            value: Some(after),
            kind: Some(kind),
        }],
    })
}

/// Karşılaştırma için değişkenleri satır öğelerine dönüştürür (hassaslar işaretli).
pub fn compare_items(bundle: &Bundle, overrides: &Overrides) -> Vec<crate::compare::Item> {
    match bundle {
        Bundle::Snapshot(d) => d
            .items
            .iter()
            .map(|i| crate::compare::Item {
                name: i.name.clone(),
                value: if i.present { i.value.clone() } else { None },
                kind: Some(i.kind),
                sensitive: i.sensitive,
                expanded: None,
            })
            .collect(),
        Bundle::Path(d) => vec![crate::compare::Item {
            name: "Path".into(),
            value: Some(d.entries.join(";")),
            kind: None,
            sensitive: false,
            expanded: None,
        }],
        _ => dedup_last(bundle.vars())
            .into_iter()
            .map(|v| crate::compare::Item {
                sensitive: v.sensitive
                    || classify::resolve(
                        &v.name,
                        v.value.as_deref().unwrap_or(""),
                        overrides.get(&name_key(&v.name)).copied(),
                    )
                    .sensitive,
                value: if v.action == VarAction::Set {
                    v.value
                } else {
                    None
                },
                kind: v.kind,
                name: v.name,
                expanded: None,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::apply::{apply, ApplyOptions};
    use crate::env::provider::MemoryProvider;
    use crate::sources::DbSources;

    #[test]
    fn env_import_previews_masked_and_applies_without_frontend_values() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        p.seed(Scope::User, "EXISTING", "old", ValueKind::String);
        let (bundle, warnings) = parse(
            "EXISTING=new\nAPI_KEY=k-123\nDUP=1\nDUP=2\nBAD NAME=x\n",
            FileFormat::Env,
        )
        .unwrap();
        assert!(warnings.iter().any(|w| w.code == "duplicateName"));
        let cache = ImportCache::default();
        let token = cache.insert(bundle.clone());
        let pv = preview(
            &db,
            &p,
            &Overrides::new(),
            token.clone(),
            FileFormat::Env,
            &bundle,
            warnings,
        )
        .unwrap();
        let key = pv.vars.iter().find(|v| v.name == "API_KEY").unwrap();
        assert!(key.sensitive && key.value.is_none());
        assert_eq!(
            pv.vars
                .iter()
                .find(|v| v.name == "DUP")
                .unwrap()
                .value
                .as_deref(),
            Some("2"),
            "son tanım geçerli"
        );
        assert_eq!(
            pv.vars
                .iter()
                .find(|v| v.name == "EXISTING")
                .unwrap()
                .exists_in,
            vec![Scope::User]
        );
        assert!(!serde_json::to_string(&pv).unwrap().contains("k-123"));

        // Var olanları atla
        let skip = variables_plan(
            &p,
            &token,
            &bundle,
            Scope::User,
            None,
            ConflictPolicy::SkipExisting,
        )
        .unwrap();
        assert!(!skip.ops.iter().any(|o| o.target() == "EXISTING"));
        assert!(
            !serde_json::to_string(&skip).unwrap().contains("k-123"),
            "plan değer içermez"
        );

        let plan = variables_plan(
            &p,
            &token,
            &bundle,
            Scope::User,
            Some(&["EXISTING".into(), "API_KEY".into()]),
            ConflictPolicy::Overwrite,
        )
        .unwrap();
        let src = DbSources::new(&db).with_imports(&cache);
        let opts = ApplyOptions {
            backup: true,
            max_auto_snapshots: 50,
            label: "import",
        };
        let no_elev = |_: &[crate::env::plan::Resolved]| -> AppResult<usize> { unreachable!() };
        apply(&db, &p, &plan, &Overrides::new(), &opts, &src, &no_elev).unwrap();
        assert_eq!(
            p.get(Scope::User, "EXISTING").unwrap().unwrap().value,
            "new"
        );
        assert_eq!(
            p.get(Scope::User, "API_KEY").unwrap().unwrap().value,
            "k-123"
        );
        assert!(
            p.get(Scope::User, "DUP").unwrap().is_none(),
            "seçilmeyen içe aktarılmaz"
        );
    }

    #[test]
    fn json_round_trip_profile_and_project() {
        let db = Db::open_in_memory().unwrap();
        let p = MemoryProvider::new();
        let o = Overrides::new();
        let dir = tempfile::tempdir().unwrap();
        let profile_in = ProfileInput {
            name: "Node".into(),
            description: "d".into(),
            category: "Node.js".into(),
            tags: vec!["web".into()],
            metadata: None,
            vars: vec![
                VarInput {
                    name: "NODE_ENV".into(),
                    value: Some("dev".into()),
                    keep: false,
                    previous_name: None,
                    sensitive: None,
                    kind: None,
                    scope: Scope::User,
                    action: VarAction::Set,
                },
                VarInput {
                    name: "NPM_TOKEN".into(),
                    value: Some("tok".into()),
                    keep: false,
                    previous_name: None,
                    sensitive: None,
                    kind: None,
                    scope: Scope::User,
                    action: VarAction::Set,
                },
            ],
            paths: vec![],
        };
        let id = profiles::create(&db, &profile_in, &o).unwrap();
        let out = super::super::export::profile(&db, id, false).unwrap();
        assert!(!out.content.contains("\"tok\""));
        let (bundle, _) = parse(&out.content, FileFormat::Json).unwrap();
        let cache = ImportCache::default();
        let pv = preview(
            &db,
            &p,
            &o,
            cache.insert(bundle.clone()),
            FileFormat::Json,
            &bundle,
            vec![],
        )
        .unwrap();
        assert!(pv.name_taken);
        let r = create_profile(&db, &bundle, "Node (2)", &o).unwrap();
        assert_eq!(r.skipped, 1, "hariç tutulan gizli değişken atlanır");

        // Proje: özel komut varsayılan olarak alınmaz
        let doc = ProjectDoc {
            name: "Web".into(),
            identifier: None,
            description: String::new(),
            directory: "Z:\\nowhere".into(),
            tags: vec![],
            launch: crate::projects::model::LaunchConfig {
                shell: None,
                custom: Some(CustomCommand {
                    program: "evil.exe".into(),
                    args: vec![],
                }),
            },
            profile_name: Some("node".into()),
            vars: vec![],
            paths: vec![],
        };
        let b = Bundle::Project(doc);
        assert!(
            create_project(&db, &b, "Web", "Z:\\nowhere", false, &o).is_err(),
            "dizin var olmalı"
        );
        let d = dir.path().to_string_lossy().to_string();
        let r = create_project(&db, &b, "Web", &d, false, &o).unwrap();
        let pr = projects::load_full(&db, r.id).unwrap();
        assert!(
            pr.launch.custom.is_none(),
            "içe aktarılan komut onaysız korunmaz"
        );
        assert_eq!(pr.profile_id, Some(id), "aynı adlı profile bağlanır");
    }

    #[test]
    fn rejects_foreign_and_future_documents() {
        assert!(parse(
            r#"{"format":"other","version":1,"kind":"profile","data":{}}"#,
            FileFormat::Json
        )
        .is_err());
        assert!(parse(
            r#"{"format":"ivily-envora","version":99,"kind":"profile","data":{}}"#,
            FileFormat::Json
        )
        .is_err());
        assert!(parse("not json", FileFormat::Json).is_err());
    }

    #[test]
    fn path_import_modes() {
        let p = MemoryProvider::new();
        p.seed(Scope::User, "Path", "C:\\a;C:\\b", ValueKind::Expand);
        let (b, _) = parse("C:\\b\n# yorum\nC:\\c;C:\\d\n", FileFormat::Text).unwrap();
        let val = |plan: ChangePlan| match &plan.ops[0] {
            Op::Set { value: Some(v), .. } => v.clone(),
            _ => unreachable!(),
        };
        assert_eq!(
            val(path_plan(&p, &b, Scope::User, PathMode::AppendMissing).unwrap()),
            "C:\\a;C:\\b;C:\\c;C:\\d"
        );
        assert_eq!(
            val(path_plan(&p, &b, Scope::User, PathMode::PrependMissing).unwrap()),
            "C:\\c;C:\\d;C:\\a;C:\\b"
        );
        assert_eq!(
            val(path_plan(&p, &b, Scope::User, PathMode::Replace).unwrap()),
            "C:\\b;C:\\c;C:\\d"
        );
    }

    #[test]
    fn csv_import_reads_columns() {
        let (b, _) = parse(
            "name,value,scope,kind\r\nA,1,machine,expand\r\n,x,,\r\n",
            FileFormat::Csv,
        )
        .unwrap();
        let Bundle::Variables(v) = b else { panic!() };
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].scope, Some(Scope::Machine));
        assert_eq!(v[0].kind, Some(ValueKind::Expand));
        assert!(parse("foo,bar\n1,2", FileFormat::Csv).is_err());
    }
}
