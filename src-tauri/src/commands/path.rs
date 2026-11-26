use crate::env::Scope;
use crate::error::AppResult;
use crate::pathmgr::analyze::{self, Conflict, PathReport};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn path_analyze(state: State<'_, AppState>, scope: Scope) -> AppResult<PathReport> {
    analyze::analyze(state.provider.as_ref(), scope)
}

/// Dizinleri listeler (dosya çalıştırmaz); kullanıcı istediğinde çağrılır.
#[tauri::command]
pub async fn path_conflicts(app: tauri::AppHandle) -> AppResult<Vec<Conflict>> {
    use tauri::Manager;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        analyze::conflicts(state.provider.as_ref())
    })
    .await
    .map_err(|e| crate::error::AppError::Internal(e.to_string()))?
}
