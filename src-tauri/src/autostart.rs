use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::paths::config_dir;

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|e| e.to_string())
    } else {
        manager.disable().map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn get_autostart(app: AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

pub(crate) fn seed_or_refresh_autostart(app: &AppHandle) {
    let marker = config_dir(app).join(".autostart_seeded");
    if !marker.exists() {
        let _ = app.autolaunch().enable();
        let _ = std::fs::write(&marker, "1");
        return;
    }

    let manager = app.autolaunch();
    if manager.is_enabled().unwrap_or(false) {
        let _ = manager.enable();
    }
}
