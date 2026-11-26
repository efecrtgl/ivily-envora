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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyResult {
    pub sensitive: bool,
    /// Panonun temizleneceği süre (saniye); 0 = temizlenmez.
    pub clear_after: u32,
}

/// Değeri arayüze göndermeden doğrudan panoya kopyalar.
#[tauri::command]
pub fn env_copy(state: State<'_, AppState>, scope: Scope, name: String) -> AppResult<CopyResult> {
    let var = state
        .provider
        .get(scope, &name)?
        .ok_or_else(|| crate::error::AppError::NotFound(name.clone()))?;
    let o = overrides::all(&state.db)?;
    let sensitive = inspect::view_of(&var, &o).sensitive;
    let seq = crate::platform::clipboard::copy_text(&var.value, sensitive)?;
    let clear_after = if sensitive {
        state.settings.read().security.clipboard_clear_seconds
    } else {
        0
    };
    crate::platform::clipboard::schedule_clear(seq, clear_after);
    if sensitive {
        activity::record_quiet(
            &state.db,
            "security",
            "copy_sensitive",
            Some(&name),
            Status::Ok,
            Some(json!({ "scope": scope })),
        );
    }
    Ok(CopyResult {
        sensitive,
        clear_after,
    })
}
