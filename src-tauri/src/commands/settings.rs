use crate::db::DbStatus;
use crate::error::{AppError, AppResult};
use crate::settings::{self, Settings};
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPayload {
    pub settings: Settings,
    /// Okuma sırasında varsayılana döndürülen alanlar.
    pub fixed: Vec<String>,
    pub degraded: Option<String>,
    pub restore_note: Option<String>,
}

#[tauri::command]
pub fn settings_get(state: State<'_, AppState>) -> SettingsPayload {
    SettingsPayload {
        settings: state.settings.read().clone(),
        fixed: state.settings_fixed.clone(),
        degraded: state.degraded.clone(),
        restore_note: state.restore_note.clone(),
    }
}

#[tauri::command]
pub fn settings_update(state: State<'_, AppState>, settings: Settings) -> AppResult<Settings> {
    settings::save(&state.db, &settings)?;
    *state.settings.write() = settings.clone();
    Ok(settings)
}

#[tauri::command]
pub fn settings_reset(state: State<'_, AppState>) -> AppResult<Settings> {
    let s = Settings::default();
    settings::save(&state.db, &s)?;
    *state.settings.write() = s.clone();
    crate::activity::record_quiet(
        &state.db,
        "settings",
        "reset",
        None,
        crate::activity::Status::Ok,
        None,
    );
    Ok(s)
}

#[tauri::command]
pub fn db_status(state: State<'_, AppState>) -> AppResult<DbStatus> {
    state.db.status()
}

/// Veritabanının tutarlı bir kopyasını kullanıcının seçtiği yola yazar.
#[tauri::command]
pub fn db_backup(state: State<'_, AppState>, dest: String) -> AppResult<()> {
    let path = std::path::PathBuf::from(&dest);
    if !path.is_absolute() {
        return Err(AppError::validation("invalid_path", "mutlak yol gerekli"));
    }
    crate::commands::io::check_extension(&path, &["db"])?;
    let res = state.db.backup_to(&path);
    let status = if res.is_ok() {
        crate::activity::Status::Ok
    } else {
        crate::activity::Status::Failed
    };
    crate::activity::record_quiet(&state.db, "backup", "database_backup", None, status, None);
    res
}

#[tauri::command]
pub fn db_backup_inspect(path: String) -> AppResult<crate::db::restore::BackupInfo> {
    crate::db::restore::inspect(std::path::Path::new(&path))
}

/// Yedeği doğrular ve bir sonraki açılışta geri yüklenmek üzere hazırlar.
#[tauri::command]
pub fn db_restore_schedule(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<crate::db::restore::BackupInfo> {
    let info = crate::db::restore::schedule(&state.data_dir, std::path::Path::new(&path))?;
    crate::activity::record_quiet(
        &state.db,
        "backup",
        "database_restore_scheduled",
        None,
        crate::activity::Status::Ok,
        None,
    );
    Ok(info)
}

#[tauri::command]
pub fn db_restore_cancel(state: State<'_, AppState>) -> AppResult<()> {
    crate::db::restore::cancel(&state.data_dir)
}

#[tauri::command]
pub fn db_restore_pending(state: State<'_, AppState>) -> bool {
    crate::db::restore::is_pending(&state.data_dir)
}
