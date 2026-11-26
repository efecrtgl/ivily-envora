use crate::activity::{self, Status as ActStatus};
use crate::env::plan::ChangePlan;
use crate::env::Scope;
use crate::error::AppResult;
use crate::profiles::activate::{self, ActivationRecord};
use crate::profiles::model::{Profile, ProfileInput, ProfileSummary, Revision, Status};
use crate::profiles::store;
use crate::profiles::validate::{self, Finding};
use crate::security::overrides;
use crate::state::AppState;
use serde_json::json;
use tauri::State;

fn log(state: &AppState, action: &str, target: &str, details: Option<serde_json::Value>) {
    activity::record_quiet(
        &state.db,
        "profile",
        action,
        Some(target),
        ActStatus::Ok,
        details,
    );
}

#[tauri::command]
pub fn profiles_list(
    state: State<'_, AppState>,
    include_archived: bool,
) -> AppResult<Vec<ProfileSummary>> {
    store::list(&state.db, include_archived)
}

#[tauri::command]
pub fn profile_get(state: State<'_, AppState>, id: i64) -> AppResult<Profile> {
    Ok(store::masked(store::load_full(&state.db, id)?))
}

#[tauri::command]
pub fn profile_create(state: State<'_, AppState>, input: ProfileInput) -> AppResult<i64> {
    let o = overrides::all(&state.db)?;
    let id = store::create(&state.db, &input, &o)?;
    log(
        &state,
        "create",
        input.name.trim(),
        Some(json!({ "vars": input.vars.len(), "paths": input.paths.len() })),
    );
    Ok(id)
}

#[tauri::command]
pub fn profile_update(
    state: State<'_, AppState>,
    id: i64,
    input: ProfileInput,
    note: Option<String>,
) -> AppResult<i64> {
    let o = overrides::all(&state.db)?;
    let version = store::update(&state.db, id, &input, note.as_deref(), &o)?;
    log(
        &state,
        "update",
        input.name.trim(),
        Some(json!({ "version": version, "vars": input.vars.len() })),
    );
    Ok(version)
}

#[tauri::command]
pub fn profile_duplicate(state: State<'_, AppState>, id: i64, name: String) -> AppResult<i64> {
    let new_id = store::duplicate(&state.db, id, &name)?;
    log(
        &state,
        "duplicate",
        name.trim(),
        Some(json!({ "from": id })),
    );
    Ok(new_id)
}

#[tauri::command]
pub fn profile_rename(state: State<'_, AppState>, id: i64, name: String) -> AppResult<()> {
    store::rename(&state.db, id, &name)?;
    log(&state, "rename", name.trim(), None);
    Ok(())
}

#[tauri::command]
pub fn profile_set_status(state: State<'_, AppState>, id: i64, status: Status) -> AppResult<()> {
    store::set_status(&state.db, id, status)?;
    log(
        &state,
        if status == Status::Archived {
            "archive"
        } else {
            "unarchive"
        },
        &id.to_string(),
        None,
    );
    Ok(())
}

#[tauri::command]
pub fn profile_delete(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let name = store::load_full(&state.db, id)
        .map(|p| p.name)
        .unwrap_or_default();
    store::delete(&state.db, id)?;
    log(&state, "delete", &name, None);
    Ok(())
}

#[tauri::command]
pub fn profile_revisions(state: State<'_, AppState>, id: i64) -> AppResult<Vec<Revision>> {
    store::revisions(&state.db, id)
}

#[tauri::command]
pub fn profile_revision_get(
    state: State<'_, AppState>,
    id: i64,
    revision_id: i64,
) -> AppResult<Profile> {
    store::revision_masked(&state.db, id, revision_id)
}

#[tauri::command]
pub fn profile_restore_revision(
    state: State<'_, AppState>,
    id: i64,
    revision_id: i64,
) -> AppResult<i64> {
    let v = store::restore_revision(&state.db, id, revision_id)?;
    log(
        &state,
        "restore_revision",
        &id.to_string(),
        Some(json!({ "revision": revision_id, "version": v })),
    );
    Ok(v)
}

#[tauri::command]
pub fn profile_reveal_var(state: State<'_, AppState>, id: i64, name: String) -> AppResult<String> {
    let v = store::reveal_var(&state.db, id, &name)?;
    activity::record_quiet(
        &state.db,
        "security",
        "reveal",
        Some(&name),
        ActStatus::Ok,
        Some(json!({ "profile": id })),
    );
    Ok(v)
}

#[tauri::command]
pub fn profile_validate(state: State<'_, AppState>, id: i64) -> AppResult<Vec<Finding>> {
    let p = store::load_full(&state.db, id)?;
    validate::check(state.provider.as_ref(), &p)
}

/// Kalıcı uygulama planları (her kapsam için bir tane). Planlar gizli değer içermez.
#[tauri::command]
pub fn profile_activation_plans(state: State<'_, AppState>, id: i64) -> AppResult<Vec<ChangePlan>> {
    let p = store::load_full(&state.db, id)?;
    activate::build_plans(state.provider.as_ref(), &p)
}

#[tauri::command]
pub fn profile_deactivation_plan(
    state: State<'_, AppState>,
    id: i64,
    scope: Scope,
) -> AppResult<ChangePlan> {
    activate::deactivation_plan(&state.db, id, scope)
}

#[tauri::command]
pub fn profile_forget_activation(
    state: State<'_, AppState>,
    id: i64,
    scope: Scope,
) -> AppResult<()> {
    activate::forget_activation(&state.db, id, scope)?;
    log(
        &state,
        "forget_activation",
        &id.to_string(),
        Some(json!({ "scope": scope })),
    );
    Ok(())
}

#[tauri::command]
pub fn profile_activations(
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<Vec<ActivationRecord>> {
    activate::history(&state.db, id)
}
