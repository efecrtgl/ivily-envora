use crate::activity::{self, Status};
use crate::env::effective;
use crate::error::{AppError, AppResult};
use crate::profiles::store as profiles;
use crate::projects::launch::{self, EnvChange, Target};
use crate::projects::model::{Project, ProjectInput, ProjectSummary};
use crate::projects::store;
use crate::security::overrides;
use crate::state::AppState;
use serde::Serialize;
use serde_json::json;
use std::path::Path;
use tauri::State;

#[tauri::command]
pub fn projects_list(state: State<'_, AppState>) -> AppResult<Vec<ProjectSummary>> {
    store::list(&state.db)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectView {
    pub project: Project,
    pub directory_exists: bool,
    pub markers: Vec<&'static str>,
}

#[tauri::command]
pub fn project_get(state: State<'_, AppState>, id: i64) -> AppResult<ProjectView> {
    let p = store::masked(store::load_full(&state.db, id)?);
    let dir = Path::new(&p.directory);
    Ok(ProjectView {
        directory_exists: dir.is_dir(),
        markers: store::detect_markers(dir),
        project: p,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirInfo {
    pub path: String,
    pub markers: Vec<&'static str>,
    pub suggested_name: String,
}

/// Seçilen dizini doğrular ve işaret dosyalarını (yalnızca varlık) bildirir.
#[tauri::command]
pub fn project_inspect_dir(path: String) -> AppResult<DirInfo> {
    let dir = store::validate_directory(&path)?;
    let suggested_name = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(DirInfo {
        markers: store::detect_markers(&dir),
        path: dir.to_string_lossy().into_owned(),
        suggested_name,
    })
}

#[tauri::command]
pub fn project_create(state: State<'_, AppState>, input: ProjectInput) -> AppResult<i64> {
    let o = overrides::all(&state.db)?;
    let id = store::create(&state.db, &input, &o)?;
    activity::record_quiet(
        &state.db,
        "project",
        "create",
        Some(input.name.trim()),
        Status::Ok,
        Some(json!({ "vars": input.vars.len() })),
    );
    Ok(id)
}

#[tauri::command]
pub fn project_update(state: State<'_, AppState>, id: i64, input: ProjectInput) -> AppResult<()> {
    let o = overrides::all(&state.db)?;
    store::update(&state.db, id, &input, &o)?;
    activity::record_quiet(
        &state.db,
        "project",
        "update",
        Some(input.name.trim()),
        Status::Ok,
        Some(json!({ "vars": input.vars.len() })),
    );
    Ok(())
}

#[tauri::command]
pub fn project_delete(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let name = store::load_full(&state.db, id)
        .map(|p| p.name)
        .unwrap_or_default();
    store::delete(&state.db, id)?;
    activity::record_quiet(
        &state.db,
        "project",
        "delete",
        Some(&name),
        Status::Ok,
        None,
    );
    Ok(())
}

#[tauri::command]
pub fn project_reveal_var(state: State<'_, AppState>, id: i64, name: String) -> AppResult<String> {
    let v = store::reveal_var(&state.db, id, &name)?;
    activity::record_quiet(
        &state.db,
        "security",
        "reveal",
        Some(&name),
        Status::Ok,
        Some(json!({ "project": id })),
    );
    Ok(v)
}

fn profile_parts(
    state: &AppState,
    p: &Project,
) -> AppResult<Option<crate::profiles::model::Profile>> {
    match p.profile_id {
        Some(pid) => Ok(Some(profiles::load_full(&state.db, pid)?)),
        None => Ok(None),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPreview {
    pub changes: Vec<EnvChange>,
    pub shells: Vec<&'static str>,
    pub default_shell: String,
    /// Özel komutun çözülmüş tam yolu (varsa) veya hata kodu.
    pub custom_program: Option<String>,
    pub custom_args: Vec<String>,
    pub custom_error: Option<String>,
}

#[tauri::command]
pub fn project_launch_preview(state: State<'_, AppState>, id: i64) -> AppResult<LaunchPreview> {
    let p = store::load_full(&state.db, id)?;
    let profile = profile_parts(&state, &p)?;
    let provider = state.provider.as_ref();
    let base = effective::fresh_environment(provider)?;
    let env = launch::launch_env(
        provider,
        &p,
        profile
            .as_ref()
            .map(|f| (f.vars.as_slice(), f.paths.as_slice())),
    )?;
    let profile_vars = profile.as_ref().map(|f| f.vars.clone()).unwrap_or_default();
    let (custom_program, custom_args, custom_error) = match &p.launch.custom {
        Some(c) => match launch::resolve_target(&Target::Custom(c), &env) {
            Ok((prog, args)) => (Some(prog.to_string_lossy().into_owned()), args, None),
            Err(e) => (None, c.args.clone(), Some(e.code().to_string())),
        },
        None => (None, Vec::new(), None),
    };
    Ok(LaunchPreview {
        changes: launch::diff(&base, &env, &profile_vars, &p),
        shells: launch::available_shells(&env),
        default_shell: p
            .launch
            .shell
            .clone()
            .unwrap_or_else(|| state.settings.read().workspace.default_shell.clone()),
        custom_program,
        custom_args,
        custom_error,
    })
}

/// Kullanıcının açık isteğiyle bir kabuk veya tanımlı özel programı proje ortamıyla başlatır.
#[tauri::command]
pub fn project_launch(
    state: State<'_, AppState>,
    id: i64,
    target: String,
    shell: Option<String>,
) -> AppResult<u32> {
    let p = store::load_full(&state.db, id)?;
    let profile = profile_parts(&state, &p)?;
    let env = launch::launch_env(
        state.provider.as_ref(),
        &p,
        profile
            .as_ref()
            .map(|f| (f.vars.as_slice(), f.paths.as_slice())),
    )?;
    let default_shell = p
        .launch
        .shell
        .clone()
        .unwrap_or_else(|| state.settings.read().workspace.default_shell.clone());
    let shell_name = shell.unwrap_or(default_shell);
    let t = match target.as_str() {
        "shell" => Target::Shell(&shell_name),
        "custom" => Target::Custom(
            p.launch
                .custom
                .as_ref()
                .ok_or_else(|| AppError::validation("invalid_program", ""))?,
        ),
        _ => return Err(AppError::validation("invalid_target", target)),
    };
    let (program, args) = launch::resolve_target(&t, &env)?;
    let res = launch::spawn(&program, &args, Path::new(&p.directory), &env);
    let program_name = program
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    activity::record_quiet(
        &state.db,
        "project",
        "launch",
        Some(&p.name),
        if res.is_ok() {
            Status::Ok
        } else {
            Status::Failed
        },
        Some(json!({ "program": program_name, "args": args.len() })),
    );
    res
}
