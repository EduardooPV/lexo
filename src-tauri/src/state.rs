use std::{
    path::PathBuf,
    sync::{Mutex, RwLock},
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};

use crate::cache::{Cache, CacheEntry};
use crate::history::HistoryEntry;
use crate::paths::{config_dir, read_json};
use crate::persist::Writer;
use crate::secrets::KeyringStore;
use crate::settings::{load_from_disk, Settings};

pub(crate) struct AppState {
    pub(crate) history: Mutex<Vec<HistoryEntry>>,
    pub(crate) cache: Mutex<Cache>,
    pub(crate) pending_update: Mutex<Option<tauri_plugin_updater::Update>>,
    pub(crate) settings: RwLock<Settings>,
    pub(crate) settings_io: Mutex<()>,
    pub(crate) http: reqwest::Client,
    pub(crate) last_warmup: Mutex<Option<Instant>>,
    pub(crate) dir: PathBuf,
    pub(crate) writer: Writer,
}

pub(crate) fn load_state(app: &AppHandle) {
    let dir = config_dir(app);
    let state = AppState {
        history: Mutex::new(read_json(&dir.join("history.json"))),
        cache: Mutex::new(Cache::from(read_json::<Vec<CacheEntry>>(
            &dir.join("cache.json"),
        ))),
        pending_update: Mutex::default(),
        settings: RwLock::new(load_from_disk(&dir.join("settings.json"), &KeyringStore)),
        settings_io: Mutex::default(),
        http: reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(30))
            .build()
            .unwrap_or_default(),
        last_warmup: Mutex::default(),
        dir,
        writer: Writer::start(app.clone()),
    };
    app.manage(state);
}
