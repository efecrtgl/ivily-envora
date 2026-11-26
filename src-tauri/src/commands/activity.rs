use crate::activity::{self, Entry, Query};
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn activity_list(state: State<'_, AppState>, query: Query) -> AppResult<Vec<Entry>> {
    activity::list(&state.db, &query)
}

/// Etkinlik geçmişini temizler (onay ön uçta alınır).
#[tauri::command]
pub fn activity_clear(state: State<'_, AppState>) -> AppResult<usize> {
    let n = activity::clear(&state.db)?;
    activity::record_quiet(
        &state.db,
        "settings",
        "activity_cleared",
        None,
        activity::Status::Ok,
        Some(serde_json::json!({ "count": n })),
    );
    Ok(n)
}
