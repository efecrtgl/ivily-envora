use crate::activity::{self, Status};
use crate::analysis::{self, Config, Report, RuleInfo};
use crate::error::AppResult;
use crate::security::overrides;
use crate::state::AppState;
use serde_json::json;
use tauri::{Manager, State};

fn config(state: &AppState) -> Config {
    let s = state.settings.read();
    Config {
        disabled: s.analysis.disabled_rules.clone(),
        severity: s.analysis.severity.clone(),
    }
}

#[tauri::command]
pub fn analysis_rules(state: State<'_, AppState>) -> Vec<RuleInfo> {
    analysis::rule_infos(&config(&state))
}

/// Analizi çalıştırır (yalnızca istendiğinde). Sonuç bellekte saklanır; pano son sonucu gösterir.
#[tauri::command]
pub async fn analysis_run(app: tauri::AppHandle) -> AppResult<Report> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let o = overrides::all(&state.db)?;
        let r = analysis::run(&state.db, state.provider.as_ref(), &o, &config(&state))?;
        activity::record_quiet(
            &state.db,
            "analysis",
            "run",
            None,
            Status::Ok,
            Some(json!({ "counts": r.counts, "rules": r.rules_run })),
        );
        *state.last_analysis.lock() = Some(r.clone());
        Ok(r)
    })
    .await
    .map_err(|e| crate::error::AppError::Internal(e.to_string()))?
}

#[tauri::command]
pub fn analysis_last(state: State<'_, AppState>) -> Option<Report> {
    state.last_analysis.lock().clone()
}
