use crate::activity::{self, Status};
use crate::env::inspect::{self, Detail, Overview, VarView};
use crate::env::Scope;
use crate::error::AppResult;
use crate::security::overrides::{self, OverrideEntry};
use crate::state::AppState;
use serde_json::json;
use tauri::State;

#[tauri::command]
pub fn env_list(state: State<'_, AppState>, scopes: Option<Vec<Scope>>) -> AppResult<Vec<VarView>> {
    let scopes = scopes.unwrap_or_else(|| Scope::ALL.to_vec());
    let o = overrides::all(&state.db)?;
    inspect::list_views(state.provider.as_ref(), &scopes, &o)
}

#[tauri::command]
pub fn env_overview(state: State<'_, AppState>) -> AppResult<Overview> {
    let o = overrides::all(&state.db)?;
    inspect::overview(state.provider.as_ref(), &o)
}

#[tauri::command]
pub fn env_detail(state: State<'_, AppState>, name: String) -> AppResult<Detail> {
    let o = overrides::all(&state.db)?;
    inspect::detail(state.provider.as_ref(), &name, &o)
}

/// Hassas bir değeri kullanıcının açık isteğiyle döndürür ve bunu (değer olmadan) kaydeder.
#[tauri::command]
pub fn env_reveal(state: State<'_, AppState>, scope: Scope, name: String) -> AppResult<String> {
    let value = inspect::reveal(state.provider.as_ref(), scope, &name)?;
    activity::record_quiet(
        &state.db,
        "security",
        "reveal",
        Some(&name),
        Status::Ok,
        Some(json!({ "scope": scope })),
    );
    Ok(value)
}

#[tauri::command]
pub fn sensitivity_set(
    state: State<'_, AppState>,
    name: String,
    sensitive: Option<bool>,
) -> AppResult<()> {
    crate::env::validate::validate_name(&name)?;
    overrides::set(&state.db, &name, sensitive)?;
    let action = match sensitive {
        Some(true) => "mark_sensitive",
        Some(false) => "mark_not_sensitive",
        None => "clear_override",
    };
    activity::record_quiet(&state.db, "security", action, Some(&name), Status::Ok, None);
    Ok(())
}

#[tauri::command]
pub fn sensitivity_list(state: State<'_, AppState>) -> AppResult<Vec<OverrideEntry>> {
    overrides::list(&state.db)
}
