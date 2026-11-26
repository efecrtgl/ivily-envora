use crate::activity::{self, Status};
use crate::env::plan::ChangePlan;
use crate::env::Scope;
use crate::error::{AppError, AppResult};
use crate::io::export::{self, ExportResult, Output};
use crate::io::import::{self, ConflictPolicy, CreateResult, PathMode, Preview};
use crate::io::{self as io_mod, FileFormat};
use crate::security::overrides;
use crate::state::AppState;
use serde::Deserialize;
use serde_json::json;
use std::path::{Path, PathBuf};
use tauri::State;

fn abs(path: &str) -> AppResult<PathBuf> {
    let p = PathBuf::from(path);
    if !p.is_absolute() {
        return Err(AppError::validation("invalid_path", ""));
    }
    Ok(p)
}

/// Dosyayı okur, ayrıştırır ve maskeli önizleme döndürür. İçerik çalıştırılmaz.
#[tauri::command]
pub fn import_preview(
    state: State<'_, AppState>,
    path: String,
    format: Option<FileFormat>,
) -> AppResult<Preview> {
    let p = abs(&path)?;
    let format = format.unwrap_or_else(|| io_mod::detect_format(&p));
    let text = io_mod::read_text(&p)?;
    let (bundle, warnings) = import::parse(&text, format)?;
    let token = state.imports.insert(bundle.clone());
    let o = overrides::all(&state.db)?;
    let pv = import::preview(
        &state.db,
        state.provider.as_ref(),
        &o,
        token,
        format,
        &bundle,
        warnings,
    )?;
    activity::record_quiet(
        &state.db,
        "import",
        "preview",
        None,
        Status::Ok,
        Some(json!({ "format": format, "kind": pv.kind, "vars": pv.vars.len() })),
    );
    Ok(pv)
}

#[tauri::command]
pub fn import_discard(state: State<'_, AppState>, token: String) {
    state.imports.remove(&token);
}

/// İçe aktarılan değişkenleri bir kapsama uygulayacak planı üretir (değer içermez).
#[tauri::command]
pub fn import_variables_plan(
    state: State<'_, AppState>,
    token: String,
    scope: Scope,
    names: Option<Vec<String>>,
    policy: ConflictPolicy,
) -> AppResult<ChangePlan> {
    let b = state.imports.get(&token)?;
    import::variables_plan(
        state.provider.as_ref(),
        &token,
        &b,
        scope,
        names.as_deref(),
        policy,
    )
}

#[tauri::command]
pub fn import_path_plan(
    state: State<'_, AppState>,
    token: String,
    scope: Scope,
    mode: PathMode,
) -> AppResult<ChangePlan> {
    let b = state.imports.get(&token)?;
    import::path_plan(state.provider.as_ref(), &b, scope, mode)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFromImport {
    pub token: String,
    pub name: String,
    #[serde(default)]
    pub directory: Option<String>,
    #[serde(default)]
    pub keep_launch: bool,
}

/// Profil, proje veya anlık görüntüyü içe aktarılan belgeden oluşturur.
#[tauri::command]
pub fn import_create(
    state: State<'_, AppState>,
    request: CreateFromImport,
) -> AppResult<CreateResult> {
    let b = state.imports.get(&request.token)?;
    let o = overrides::all(&state.db)?;
    let kind = b.kind();
    let r = match kind {
        io_mod::DocKind::Profile => import::create_profile(&state.db, &b, &request.name, &o)?,
        io_mod::DocKind::Project => {
            let dir = request
                .directory
                .as_deref()
                .ok_or_else(|| AppError::validation("invalid_directory", ""))?;
            import::create_project(&state.db, &b, &request.name, dir, request.keep_launch, &o)?
        }
        io_mod::DocKind::Snapshot => import::create_snapshot(&state.db, &b, &request.name)?,
        io_mod::DocKind::Template => {
            import::create_template(&state.db, &b, &request.name)?;
            CreateResult { id: 0, skipped: 0 }
        }
        _ => return Err(AppError::validation("wrong_kind", "")),
    };
    activity::record_quiet(
        &state.db,
        "import",
        "create",
        Some(request.name.trim()),
        Status::Ok,
        Some(json!({ "kind": kind, "skipped": r.skipped, "keepLaunch": request.keep_launch })),
    );
    state.imports.remove(&request.token);
    Ok(r)
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ExportRequest {
    #[serde(rename_all = "camelCase")]
    Variables {
        scopes: Vec<Scope>,
        names: Option<Vec<String>>,
        format: FileFormat,
    },
    Profile {
        id: i64,
    },
    Project {
        id: i64,
    },
    Snapshot {
        id: i64,
    },
    #[serde(rename_all = "camelCase")]
    Path {
        scope: Scope,
        format: FileFormat,
    },
    Template {
        id: String,
    },
}

/// Dışa aktarır ve kullanıcının seçtiği yola yazar. Hassas değerler yalnızca açıkça istenirse yazılır.
#[tauri::command]
pub fn export_write(
    state: State<'_, AppState>,
    request: ExportRequest,
    path: String,
    include_sensitive: bool,
) -> AppResult<ExportResult> {
    let dest = abs(&path)?;
    let o = overrides::all(&state.db)?;
    let p = state.provider.as_ref();
    let (kind, out): (&str, Output) = match &request {
        ExportRequest::Variables {
            scopes,
            names,
            format,
        } => (
            "variables",
            export::variables(p, &o, scopes, names.as_deref(), *format, include_sensitive)?,
        ),
        ExportRequest::Profile { id } => (
            "profile",
            export::profile(&state.db, *id, include_sensitive)?,
        ),
        ExportRequest::Project { id } => (
            "project",
            export::project(&state.db, *id, include_sensitive)?,
        ),
        ExportRequest::Snapshot { id } => (
            "snapshot",
            export::snapshot(&state.db, *id, include_sensitive)?,
        ),
        ExportRequest::Path { scope, format } => ("path", export::path(p, *scope, *format)?),
        ExportRequest::Template { id } => {
            let en = state.settings.read().general.language == "en";
            ("template", export::template(&state.db, id, en)?)
        }
    };
    io_mod::write_file(Path::new(&dest), &out.content)?;
    activity::record_quiet(
        &state.db,
        "export",
        kind,
        None,
        if out.result.contains_secrets {
            Status::Warning
        } else {
            Status::Ok
        },
        Some(
            json!({ "count": out.result.count, "excludedSensitive": out.result.excluded_sensitive, "containsSecrets": out.result.contains_secrets }),
        ),
    );
    Ok(out.result)
}
