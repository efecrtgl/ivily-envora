//! Ivily Envora arka ucu.
//!
//! Modül düzeni için `docs/MIMARI.md` belgesine bakın.

pub mod activity;
pub mod analysis;
pub mod commands;
pub mod compare;
pub mod db;
pub mod discovery;
pub mod env;
pub mod envset;
pub mod error;
pub mod io;
pub mod pathmgr;
pub mod platform;
pub mod profiles;
pub mod projects;
pub mod security;
pub mod settings;
pub mod snapshots;
pub mod sources;
pub mod state;
pub mod templates;

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
            commands::activity::activity_list,
            commands::analysis::analysis_rules,
            commands::analysis::analysis_run,
            commands::analysis::analysis_last,
            commands::discovery::discovery_run,
            commands::discovery::discovery_last,
            commands::templates::templates_list,
            commands::templates::template_get,
            commands::templates::template_create,
            commands::templates::template_update,
            commands::templates::template_duplicate,
            commands::templates::template_delete,
            commands::settings::settings_get,
            commands::settings::settings_update,
            commands::settings::settings_reset,
            commands::settings::db_status,
            commands::settings::db_backup,
            commands::snapshots::snapshots_list,
            commands::snapshots::snapshot_get,
            commands::snapshots::snapshot_create,
            commands::snapshots::snapshot_update,
            commands::snapshots::snapshot_delete,
            commands::snapshots::snapshot_reveal,
            commands::snapshots::snapshot_restore_plans,
            commands::settings::db_backup_inspect,
            commands::settings::db_restore_schedule,
            commands::settings::db_restore_cancel,
            commands::settings::db_restore_pending,
            commands::app::app_restart,
            commands::env::env_list,
            commands::env::env_overview,
            commands::env::env_detail,
            commands::env::env_reveal,
            commands::env::env_copy,
            commands::env::env_plan_preview,
            commands::env::env_plan_apply,
            commands::path::path_analyze,
            commands::compare::compare_sources,
            commands::io::import_preview,
            commands::io::import_discard,
            commands::io::import_variables_plan,
            commands::io::import_path_plan,
            commands::io::import_create,
            commands::io::export_write,
            commands::path::path_conflicts,
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
            commands::projects::projects_list,
            commands::projects::project_get,
            commands::projects::project_inspect_dir,
            commands::projects::project_create,
            commands::projects::project_update,
            commands::projects::project_delete,
            commands::projects::project_reveal_var,
            commands::projects::project_launch_preview,
            commands::projects::project_launch,
            commands::env::sensitivity_set,
            commands::env::sensitivity_list,
        ])
        .run(tauri::generate_context!())
        .expect("Ivily Envora başlatılamadı");
}
