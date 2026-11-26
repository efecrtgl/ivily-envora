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
