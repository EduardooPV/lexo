use std::sync::Mutex;
use tauri::{AppHandle, Manager};

use crate::cache::CacheEntry;
use crate::history::HistoryEntry;
use crate::paths::{cache_path, history_path, read_json};

#[derive(Default)]
pub(crate) struct AppState {
    pub(crate) history: Mutex<Vec<HistoryEntry>>,
    pub(crate) cache: Mutex<Vec<CacheEntry>>,
    pub(crate) pending_update: Mutex<Option<tauri_plugin_updater::Update>>,
}

pub(crate) fn load_state(app: &AppHandle) {
    let state = app.state::<AppState>();

    let history = state.history.lock();
    if let Ok(mut history) = history {
        *history = read_json(&history_path(app));
    }

    let cache = state.cache.lock();
    if let Ok(mut cache) = cache {
        *cache = read_json(&cache_path(app));
    }
}
