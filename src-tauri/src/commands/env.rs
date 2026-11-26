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

// ---------- Değişiklik planları ----------

use crate::env::apply::{self, ApplyOptions, ApplyOutcome};
use crate::env::plan::{self, Action, ChangePlan, Preview};
use tauri::Manager;

#[tauri::command]
pub fn env_plan_preview(state: State<'_, AppState>, plan: ChangePlan) -> AppResult<Preview> {
    let o = overrides::all(&state.db)?;
    plan::preview(state.provider.as_ref(), &plan, &o)
}

fn plan_label(plan: &ChangePlan) -> String {
    let names: Vec<&str> = plan.ops.iter().map(|o| o.target()).take(3).collect();
    let more = plan.ops.len().saturating_sub(3);
    let mut s = format!("{} · {}", plan.scope.as_str(), names.join(", "));
    if more > 0 {
        s.push_str(&format!(" +{more}"));
    }
    s
}

/// Onaylanmış planı uygular: otomatik yedek → uygulama → `WM_SETTINGCHANGE`.
/// Sistem kapsamında ve yönetici değilsek yalnızca bu işlem için UAC istemi açılır.
#[tauri::command]
pub async fn env_plan_apply(app: tauri::AppHandle, plan: ChangePlan) -> AppResult<ApplyOutcome> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let o = overrides::all(&state.db)?;
        let preview = plan::preview(state.provider.as_ref(), &plan, &o)?;
        let count = |a: Action| preview.items.iter().filter(|i| i.action == a).count();
        let settings = state.settings.read().clone();
        let label = plan_label(&plan);
        let opts = ApplyOptions {
            backup: settings.backup.auto_snapshot_before_changes,
            max_auto_snapshots: settings.backup.max_auto_snapshots,
            label: &label,
        };
        let data_dir = state.data_dir.clone();
        let elevate =
            move |ops: &[plan::Resolved]| crate::env::elevate::run_elevated(&data_dir, ops);
        let target = if plan.ops.len() == 1 {
            Some(plan.ops[0].target().to_string())
        } else {
            None
        };
        let res = apply::apply(
            &state.db,
            state.provider.as_ref(),
            &plan,
            &o,
            &opts,
            &elevate,
        );
        let details = json!({
            "scope": plan.scope,
            "created": count(Action::Create),
            "updated": count(Action::Update),
            "deleted": count(Action::Delete),
            "elevated": res.as_ref().map(|r| r.elevated).unwrap_or(false),
            "snapshotId": res.as_ref().ok().and_then(|r| r.snapshot_id),
            "error": res.as_ref().err().map(|e| e.code()),
        });
        let status = match &res {
            Ok(_) => Status::Ok,
            Err(crate::error::AppError::Cancelled) => Status::Warning,
            Err(_) => Status::Failed,
        };
        activity::record_quiet(
            &state.db,
            "variable",
            "apply",
            target.as_deref(),
            status,
            Some(details),
        );
        res
    })
    .await
    .map_err(|e| crate::error::AppError::Internal(e.to_string()))?
}
