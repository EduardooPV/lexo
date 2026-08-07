use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateInfo {
    pub(crate) version: String,
    pub(crate) notes: Option<String>,
}

async fn check_remote_update(
    app: &AppHandle,
) -> Result<Option<tauri_plugin_updater::Update>, String> {
    use tauri_plugin_updater::UpdaterExt;
    app.updater()
        .map_err(|e| format!("updater_error: {e}"))?
        .check()
        .await
        .map_err(|e| format!("updater_error: {e}"))
}

async fn refresh_pending_update(app: &AppHandle) -> Result<Option<UpdateInfo>, String> {
    let update = check_remote_update(app).await?;
    let info = update.as_ref().map(|update| UpdateInfo {
        version: update.version.clone(),
        notes: update.body.clone(),
    });
    let state = app.state::<AppState>();
    if let Ok(mut pending) = state.pending_update.lock() {
        *pending = update;
    }
    Ok(info)
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    refresh_pending_update(&app).await
}

#[tauri::command]
pub fn pending_update(app: AppHandle) -> Option<UpdateInfo> {
    let state = app.state::<AppState>();
    let pending = state.pending_update.lock().ok()?;
    pending.as_ref().map(|update| UpdateInfo {
        version: update.version.clone(),
        notes: update.body.clone(),
    })
}

#[tauri::command]
pub fn app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let taken = {
        let state = app.state::<AppState>();
        state
            .pending_update
            .lock()
            .ok()
            .and_then(|mut pending| pending.take())
    };
    let Some(update) = taken else {
        return Err("updater_none: already on the latest version.".to_string());
    };
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| format!("updater_error: {e}"))?;
    app.restart();
}

pub(crate) fn check_for_update_on_startup(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Ok(Some(info)) = refresh_pending_update(&app).await {
            let _ = app.emit_to("popup", "update-available", info);
        }
    });
}
