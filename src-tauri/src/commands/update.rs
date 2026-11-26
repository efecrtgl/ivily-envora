use crate::activity::{self, Status};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::update::{self, UpdateInfo};
use serde::Serialize;
use serde_json::json;
use tauri::ipc::Channel;
use tauri::{Manager, State};
use tauri_plugin_updater::UpdaterExt;

/// Denetimde bulunan ve kullanıcı onayını bekleyen güncelleme.
#[derive(Default)]
pub struct PendingUpdate(pub parking_lot::Mutex<Option<tauri_plugin_updater::Update>>);

fn map_err(e: tauri_plugin_updater::Error) -> AppError {
    AppError::Platform(format!("güncelleme: {e}"))
}

/// Güncelleme olup olmadığını denetler; hiçbir şey indirmez veya kurmaz.
#[tauri::command]
pub async fn update_check(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> AppResult<Option<UpdateInfo>> {
    let channel = state.settings.read().updates.channel.clone();
    let endpoint = update::endpoint_for(&channel).await?;
    if !update::is_official_asset(&endpoint) {
        return Err(AppError::validation("invalid_update_source", ""));
    }
    let url = endpoint
        .parse()
        .map_err(|_| AppError::validation("invalid_update_source", ""))?;
    let updater = app
        .updater_builder()
        .endpoints(vec![url])
        .map_err(map_err)?
        .build()
        .map_err(map_err)?;
    let result = updater.check().await;
    let found = match result {
        Ok(u) => u,
        Err(e) => {
            activity::record_quiet(
                &state.db,
                "update",
                "check",
                None,
                Status::Failed,
                Some(json!({ "channel": channel })),
            );
            return Err(map_err(e));
        }
    };
    let info = found.as_ref().map(|u| UpdateInfo {
        version: u.version.clone(),
        current_version: u.current_version.clone(),
        notes: u.body.clone(),
        date: u.date.map(|d| d.to_string()),
        channel: channel.clone(),
    });
    activity::record_quiet(
        &state.db,
        "update",
        "check",
        info.as_ref().map(|i| i.version.as_str()),
        Status::Ok,
        Some(json!({ "channel": channel, "available": info.is_some() })),
    );
    *app.state::<PendingUpdate>().0.lock() = found;
    Ok(info)
}

#[derive(Clone, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum Progress {
    #[serde(rename_all = "camelCase")]
    Downloading {
        received: u64,
        total: Option<u64>,
    },
    Installing,
}

/// Kullanıcı onayından sonra: indirir, imzayı doğrular, kurar ve uygulamayı yeniden başlatır.
/// İmza doğrulanamazsa kurulum yapılmaz.
#[tauri::command]
pub async fn update_install(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    on_progress: Channel<Progress>,
) -> AppResult<()> {
    let update = app
        .state::<PendingUpdate>()
        .0
        .lock()
        .take()
        .ok_or_else(|| AppError::validation("no_pending_update", ""))?;
    let version = update.version.clone();
    let mut received = 0u64;
    let progress = on_progress.clone();
    let result = update
        .download_and_install(
            move |chunk, total| {
                received += chunk as u64;
                let _ = progress.send(Progress::Downloading { received, total });
            },
            move || {
                let _ = on_progress.send(Progress::Installing);
            },
        )
        .await;
    match result {
        Ok(()) => {
            activity::record_quiet(
                &state.db,
                "update",
                "install",
                Some(&version),
                Status::Ok,
                None,
            );
            app.restart();
        }
        Err(e) => {
            activity::record_quiet(
                &state.db,
                "update",
                "install",
                Some(&version),
                Status::Failed,
                None,
            );
            Err(map_err(e))
        }
    }
}
