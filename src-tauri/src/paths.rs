use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub(crate) fn config_dir(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_config_dir().expect("no app config dir");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub(crate) fn settings_path(app: &AppHandle) -> PathBuf {
    config_dir(app).join("settings.json")
}
pub(crate) fn history_path(app: &AppHandle) -> PathBuf {
    config_dir(app).join("history.json")
}
pub(crate) fn cache_path(app: &AppHandle) -> PathBuf {
    config_dir(app).join("cache.json")
}

pub(crate) fn read_json<T: serde::de::DeserializeOwned + Default>(path: &PathBuf) -> T {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub(crate) fn write_json<T: Serialize>(path: &PathBuf, value: &T) {
    if let Ok(json) = serde_json::to_string(value) {
        let _ = std::fs::write(path, json);
    }
}
