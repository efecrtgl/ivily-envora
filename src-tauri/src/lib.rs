//! Ivily Envora arka ucu.
//!
//! Modül düzeni için `docs/MIMARI.md` belgesine bakın.

pub mod commands;
pub mod error;
pub mod security;

pub const APP_NAME: &str = "Ivily Envora";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            use tauri::Manager;
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![commands::app::app_info])
        .run(tauri::generate_context!())
        .expect("Ivily Envora başlatılamadı");
}
