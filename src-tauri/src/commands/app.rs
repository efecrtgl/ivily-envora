use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub platform: &'static str,
    pub arch: &'static str,
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: crate::APP_NAME,
        version: crate::APP_VERSION,
        platform: std::env::consts::OS,
        arch: std::env::consts::ARCH,
    }
}

/// Uygulamayı yeniden başlatır (ör. veritabanı geri yüklemesinden sonra).
#[tauri::command]
pub fn app_restart(app: tauri::AppHandle) {
    app.restart();
}

/// Hata bildirimlerine eklenebilecek, değişken adı/değeri içermeyen tanılama raporu.
#[tauri::command]
pub fn diagnostics_report(
    state: tauri::State<'_, crate::state::AppState>,
) -> crate::error::AppResult<String> {
    let extra = crate::diagnostics::Extra {
        webview: tauri::webview_version().unwrap_or_else(|_| "bilinmiyor".into()),
        analysis: state
            .last_analysis
            .lock()
            .as_ref()
            .map(|r| r.counts.clone()),
    };
    let settings = state.settings.read().clone();
    crate::diagnostics::report(&state.db, state.provider.as_ref(), &settings, &extra)
}
