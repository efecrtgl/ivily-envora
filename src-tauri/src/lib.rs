//! Ivily Envora arka ucu.
//!
//! Modül düzeni için `docs/MIMARI.md` belgesine bakın.

pub mod activity;
pub mod commands;
pub mod db;
pub mod env;
pub mod error;
pub mod pathmgr;
pub mod platform;
pub mod profiles;
pub mod security;
pub mod settings;
pub mod snapshots;
pub mod sources;
pub mod state;

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
        .manage(state::AppState::init())
        .invoke_handler(tauri::generate_handler![
            commands::app::app_info,
            commands::settings::settings_get,
            commands::settings::settings_update,
            commands::settings::settings_reset,
            commands::settings::db_status,
            commands::settings::db_backup,
            commands::env::env_list,
            commands::env::env_overview,
            commands::env::env_detail,
            commands::env::env_reveal,
            commands::env::env_copy,
            commands::env::env_plan_preview,
            commands::env::env_plan_apply,
            commands::profiles::profiles_list,
            commands::profiles::profile_get,
            commands::profiles::profile_create,
            commands::profiles::profile_update,
            commands::profiles::profile_duplicate,
            commands::profiles::profile_rename,
            commands::profiles::profile_set_status,
            commands::profiles::profile_delete,
            commands::profiles::profile_revisions,
            commands::profiles::profile_revision_get,
            commands::profiles::profile_restore_revision,
            commands::profiles::profile_reveal_var,
            commands::profiles::profile_validate,
            commands::profiles::profile_activation_plans,
            commands::profiles::profile_deactivation_plan,
            commands::profiles::profile_forget_activation,
            commands::profiles::profile_activations,
            commands::env::sensitivity_set,
            commands::env::sensitivity_list,
        ])
        .run(tauri::generate_context!())
        .expect("Ivily Envora başlatılamadı");
}
