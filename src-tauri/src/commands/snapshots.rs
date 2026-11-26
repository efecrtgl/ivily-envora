use crate::activity::{self, Status};
use crate::env::plan::ChangePlan;
use crate::env::Scope;
use crate::error::AppResult;
use crate::security::overrides;
use crate::snapshots::manage::{self, CreateRequest, CreateResult, Detail};
use crate::snapshots::store::{self, SnapshotMeta};
use crate::state::AppState;
use serde_json::json;
use tauri::State;

#[tauri::command]
pub fn snapshots_list(state: State<'_, AppState>) -> AppResult<Vec<SnapshotMeta>> {
    store::list(&state.db)
}

#[tauri::command]
pub fn snapshot_get(state: State<'_, AppState>, id: i64) -> AppResult<Detail> {
    manage::detail(&state.db, id)
}

#[tauri::command]
pub fn snapshot_create(
    state: State<'_, AppState>,
    request: CreateRequest,
) -> AppResult<CreateResult> {
    let o = overrides::all(&state.db)?;
    let r = manage::create(&state.db, state.provider.as_ref(), &o, &request)?;
    activity::record_quiet(
        &state.db,
        "snapshot",
        "create",
        Some(request.name.trim()),
        Status::Ok,
        Some(json!({ "items": r.items, "excludedSensitive": r.excluded_sensitive })),
    );
    Ok(r)
}

#[tauri::command]
pub fn snapshot_update(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    description: Option<String>,
) -> AppResult<()> {
    manage::update_meta(&state.db, id, &name, description.as_deref())
}

#[tauri::command]
pub fn snapshot_delete(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let name = store::get_meta(&state.db, id)
        .map(|m| m.name)
        .unwrap_or_default();
    store::delete(&state.db, id)?;
    activity::record_quiet(
        &state.db,
        "snapshot",
        "delete",
        Some(&name),
        Status::Ok,
        None,
    );
    Ok(())
}

#[tauri::command]
pub fn snapshot_reveal(
    state: State<'_, AppState>,
    id: i64,
    scope: Scope,
    name: String,
) -> AppResult<String> {
    let v = manage::reveal(&state.db, id, scope, &name)?;
    activity::record_quiet(
        &state.db,
        "security",
        "reveal",
        Some(&name),
        Status::Ok,
        Some(json!({ "snapshot": id })),
    );
    Ok(v)
}

/// Geri yükleme planları (kalıcı kapsam başına). Uygulama, önizleme ve onaydan sonra
/// `env_plan_apply` ile `snapshotRestore` bağlamında yapılır.
#[tauri::command]
pub fn snapshot_restore_plans(
    state: State<'_, AppState>,
    id: i64,
    names: Option<Vec<String>>,
) -> AppResult<Vec<ChangePlan>> {
    manage::restore_plans(&state.db, id, names.as_deref())
}
