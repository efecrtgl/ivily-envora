use crate::activity::{self, Entry, Query};
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn activity_list(state: State<'_, AppState>, query: Query) -> AppResult<Vec<Entry>> {
    activity::list(&state.db, &query)
}
