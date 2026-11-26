use crate::activity::{self, Status};
use crate::discovery::{self, Options, Report};
use crate::error::AppResult;
use crate::state::AppState;
use serde_json::json;
use std::time::Duration;
use tauri::{Manager, State};

/// Araçları keşfeder (yalnızca istendiğinde). Hiçbir şey indirilmez/kurulmaz, PATH değişmez.
#[tauri::command]
pub async fn discovery_run(app: tauri::AppHandle) -> AppResult<Report> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let (probe, timeout) = {
            let s = state.settings.read();
            (
                s.performance.probe_tool_versions,
                s.performance.probe_timeout_ms,
            )
        };
        let r = discovery::discover(
            state.provider.as_ref(),
            &Options {
                probe,
                timeout: Duration::from_millis(timeout as u64),
            },
        )?;
        let found = r.tools.iter().filter(|t| t.found).count();
        activity::record_quiet(
            &state.db,
            "discovery",
            "run",
            None,
            Status::Ok,
            Some(json!({ "found": found, "probed": r.probed })),
        );
        *state.last_discovery.lock() = Some(r.clone());
        Ok(r)
    })
    .await
    .map_err(|e| crate::error::AppError::Internal(e.to_string()))?
}

#[tauri::command]
pub fn discovery_last(state: State<'_, AppState>) -> Option<Report> {
    state.last_discovery.lock().clone()
}
