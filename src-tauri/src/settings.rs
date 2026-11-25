use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::{AppHandle, Manager};

use crate::paths::{read_json, write_json};
use crate::persist::{self, Kind};
use crate::secrets::{store_verified, KeyringStore, SecretStore};
use crate::state::AppState;

const DEFAULT_HOTKEY: &str = "Alt+R";
const DEFAULT_SELECTION_HOTKEY: &str = "Alt+T";
const DEFAULT_REPLACE_HOTKEY: &str = "Alt+Shift+T";
const DEFAULT_OCR_HOTKEY: &str = "Alt+S";
const DEFAULT_SWAP_HOTKEY: &str = "Alt+E";

const SETTINGS_VERSION: u32 = 2;
const KEYRING: &str = "keyring";
const FILE: &str = "file";
const FALLBACK_WARNING: &str =
    "Saved, but the system keychain is unavailable — the key is stored in the settings file.";

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Appearance {
    #[serde(default = "default_opacity")]
    pub(crate) opacity: f64,
    #[serde(default = "default_bg")]
    pub(crate) bg: String,
    #[serde(default = "default_accent")]
    pub(crate) accent: String,
    #[serde(default = "default_text")]
    pub(crate) text: String,
    #[serde(default = "default_font")]
    pub(crate) font: String,
    #[serde(default = "default_font_weight")]
    pub(crate) font_weight: u32,
    #[serde(default = "default_zoom")]
    pub(crate) zoom: f64,
}

impl Default for Appearance {
    fn default() -> Self {
        Appearance {
            opacity: default_opacity(),
            bg: default_bg(),
            accent: default_accent(),
            text: default_text(),
            font: default_font(),
            font_weight: default_font_weight(),
            zoom: default_zoom(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Settings {
    #[serde(default = "default_hotkey")]
    pub(crate) hotkey: String,
    #[serde(default = "default_swap_hotkey")]
    pub(crate) swap_hotkey: String,
    #[serde(default = "default_selection_hotkey")]
    pub(crate) selection_hotkey: String,
    #[serde(default = "default_replace_hotkey")]
    pub(crate) replace_hotkey: String,
    #[serde(default = "default_ocr_hotkey")]
    pub(crate) ocr_hotkey: String,
    #[serde(default)]
    pub(crate) auto_translate_clipboard: bool,
    #[serde(default)]
    pub(crate) shortcuts_paused: bool,
    #[serde(default)]
    pub(crate) deepl_key: String,
    #[serde(default = "default_key_storage")]
    pub(crate) key_storage: String,
    #[serde(default)]
    pub(crate) quota_warned: bool,
    #[serde(default = "default_true")]
    pub(crate) keep_history: bool,
    #[serde(default)]
    pub(crate) appearance: Appearance,
    #[serde(default)]
    pub(crate) popup_x: Option<i32>,
    #[serde(default)]
    pub(crate) popup_y: Option<i32>,
    #[serde(default)]
    pub(crate) settings_version: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            hotkey: default_hotkey(),
            swap_hotkey: default_swap_hotkey(),
            selection_hotkey: default_selection_hotkey(),
            replace_hotkey: default_replace_hotkey(),
            ocr_hotkey: default_ocr_hotkey(),
            auto_translate_clipboard: false,
            shortcuts_paused: false,
            deepl_key: String::new(),
            key_storage: default_key_storage(),
            quota_warned: false,
            keep_history: true,
            appearance: Appearance::default(),
            popup_x: None,
            popup_y: None,
            settings_version: SETTINGS_VERSION,
        }
    }
}

fn default_hotkey() -> String {
    DEFAULT_HOTKEY.to_string()
}
fn default_swap_hotkey() -> String {
    DEFAULT_SWAP_HOTKEY.to_string()
}
fn default_selection_hotkey() -> String {
    DEFAULT_SELECTION_HOTKEY.to_string()
}
fn default_replace_hotkey() -> String {
    DEFAULT_REPLACE_HOTKEY.to_string()
}
fn default_ocr_hotkey() -> String {
    DEFAULT_OCR_HOTKEY.to_string()
}
fn default_opacity() -> f64 {
    1.0
}
fn default_bg() -> String {
    "#282a36".to_string()
}
fn default_accent() -> String {
    "#bd93f9".to_string()
}
fn default_text() -> String {
    "#f8f8f2".to_string()
}
fn default_font() -> String {
    "'Segoe UI', system-ui, sans-serif".to_string()
}

fn default_font_weight() -> u32 {
    400
}
fn default_zoom() -> f64 {
    1.0
}

fn default_key_storage() -> String {
    FILE.into()
}

fn default_true() -> bool {
    true
}

pub(crate) fn load_settings(app: &AppHandle) -> Settings {
    app.state::<AppState>()
        .settings
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

fn settings_for_disk(settings: &Settings) -> Settings {
    let mut settings = settings.clone();
    if settings.key_storage == KEYRING {
        settings.deepl_key.clear();
    }
    settings
}

fn write_settings(path: &Path, settings: &Settings) -> Result<(), String> {
    write_json(path, &settings_for_disk(settings)).map_err(|error| error.to_string())
}

pub(crate) fn write_settings_snapshot(state: &AppState) -> Result<(), String> {
    // Serialize all writes to settings.json. Take the snapshot after acquiring
    // this lock so a debounced write cannot overwrite a synchronous save.
    let _io = state.settings_io.lock().unwrap_or_else(|e| e.into_inner());
    let settings = state
        .settings
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    write_settings(&state.dir.join("settings.json"), &settings)
}

fn commit_settings(
    state: &AppState,
    settings: &mut Settings,
    updated: Settings,
) -> Result<(), String> {
    write_settings(&state.dir.join("settings.json"), &updated)?;
    *settings = updated;
    Ok(())
}

pub(crate) fn persist_settings(
    app: &AppHandle,
    update: impl FnOnce(&mut Settings),
) -> Result<Settings, String> {
    let state = app.state::<AppState>();
    let _io = state.settings_io.lock().unwrap_or_else(|e| e.into_inner());
    let mut current = state.settings.write().unwrap_or_else(|e| e.into_inner());
    let mut updated = current.clone();
    update(&mut updated);
    commit_settings(&state, &mut current, updated)?;
    Ok(current.clone())
}

pub(crate) fn update_popup_position(app: &AppHandle, x: i32, y: i32) {
    let state = app.state::<AppState>();
    let mut settings = state.settings.write().unwrap_or_else(|e| e.into_inner());
    settings.popup_x = Some(x);
    settings.popup_y = Some(y);
    persist::schedule(app, Kind::Settings);
}

pub(crate) fn load_from_disk(path: &Path, store: &dyn SecretStore) -> Settings {
    let mut settings: Settings = read_json(path);
    // Load a keyring-backed file once. An inaccessible keyring must never
    // cause an empty in-memory key to be persisted as a file-backed key.
    if settings.key_storage == KEYRING {
        settings.deepl_key = store
            .get()
            .unwrap_or_else(|error| {
                eprintln!("{error}");
                None
            })
            .unwrap_or_default();
    }
    if migrate_settings(&mut settings, store) {
        let _ = write_settings(path, &settings);
    }
    settings
}

fn migrate_settings(settings: &mut Settings, store: &dyn SecretStore) -> bool {
    let version = settings.settings_version;
    if version < 1 {
        settings.auto_translate_clipboard = false;
        settings.settings_version = 1;
    }
    if settings.settings_version == 1 {
        if settings.deepl_key.is_empty() {
            settings.settings_version = 2;
        } else if let Err(error) = store_verified(store, &settings.deepl_key) {
            eprintln!("{error}");
        } else {
            settings.key_storage = KEYRING.into();
            settings.settings_version = 2;
        }
    }
    settings.settings_version != version
}

fn merge_settings(current: &Settings, settings: Settings) -> Settings {
    Settings {
        popup_x: current.popup_x,
        popup_y: current.popup_y,
        shortcuts_paused: current.shortcuts_paused,
        key_storage: current.key_storage.clone(),
        quota_warned: current.quota_warned,
        settings_version: current.settings_version,
        ..settings
    }
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    load_settings(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<Option<String>, String> {
    let state = app.state::<AppState>();
    let _io = state.settings_io.lock().unwrap_or_else(|e| e.into_inner());
    let mut current = state.settings.write().unwrap_or_else(|e| e.into_inner());
    let mut updated = merge_settings(&current, settings);
    updated.deepl_key = updated.deepl_key.trim().into();
    if updated.deepl_key == current.deepl_key {
        commit_settings(&state, &mut current, updated)?;
        return Ok(None);
    }

    let store = KeyringStore;
    if updated.deepl_key.is_empty() {
        updated.key_storage = FILE.into();
        updated.settings_version = updated.settings_version.max(SETTINGS_VERSION);
        // Commit first. A failed disk write must not remove the only usable key.
        commit_settings(&state, &mut current, updated)?;
        return Ok(store
            .delete()
            .err()
            .map(|error| format!("Saved, but {error}.")));
    }

    let warning = match store_verified(&store, &updated.deepl_key) {
        Ok(()) => {
            updated.key_storage = KEYRING.into();
            None
        }
        Err(error) => {
            eprintln!("{error}");
            updated.key_storage = FILE.into();
            Some(FALLBACK_WARNING.into())
        }
    };
    updated.settings_version = updated.settings_version.max(SETTINGS_VERSION);

    if let Err(error) = commit_settings(&state, &mut current, updated) {
        // A replacement can change the keyring before the atomic file write
        // fails. Restore the key referenced by the still-current settings.
        if current.key_storage == KEYRING && !current.deepl_key.is_empty() {
            if let Err(rollback) = store_verified(&store, &current.deepl_key) {
                eprintln!("Could not restore the previous keychain entry: {rollback}");
                return Err(format!(
                    "{error}. Could not restore the previous keychain entry."
                ));
            }
        }
        return Err(error);
    }

    Ok(warning)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod settings {
        use super::*;

        #[test]
        fn defaults_match_the_documented_shortcuts() {
            let settings = Settings::default();
            assert_eq!(settings.hotkey, "Alt+R");
            assert_eq!(settings.selection_hotkey, "Alt+T");
            assert_eq!(settings.replace_hotkey, "Alt+Shift+T");
            assert_eq!(settings.ocr_hotkey, "Alt+S");
            assert_eq!(settings.swap_hotkey, "Alt+E");
        }

        #[test]
        fn appearance_defaults_cover_font_weight_and_zoom() {
            let settings: Settings =
                serde_json::from_str(r##"{"appearance":{"bg":"#000000"}}"##).unwrap();
            assert_eq!(settings.appearance.font_weight, 400);
            assert_eq!(settings.appearance.zoom, 1.0);
        }

        #[test]
        fn auto_translate_clipboard_is_opt_in() {
            assert!(!Settings::default().auto_translate_clipboard);
        }

        #[test]
        fn a_fresh_default_is_already_at_the_current_schema_version() {
            assert_eq!(Settings::default().settings_version, SETTINGS_VERSION);
        }

        #[test]
        fn an_empty_json_object_loads_with_every_default_filled_in() {
            let settings: Settings = serde_json::from_str("{}").unwrap();
            assert_eq!(settings.hotkey, "Alt+R");
            assert_eq!(settings.appearance.bg, "#282a36");
            assert!(settings.deepl_key.is_empty());
            assert!(settings.keep_history);
            assert_eq!(settings.key_storage, "file");
            assert!(!settings.quota_warned);
        }

        #[test]
        fn a_pre_migration_file_reports_schema_version_zero() {
            let stored = r#"{"hotkey":"Alt+R","autoTranslateClipboard":true}"#;
            let settings: Settings = serde_json::from_str(stored).unwrap();
            assert_eq!(settings.settings_version, 0);
            assert!(settings.auto_translate_clipboard);
        }

        #[test]
        fn unknown_fields_from_a_newer_version_do_not_break_loading() {
            let stored = r#"{"hotkey":"Alt+K","somethingFromTheFuture":42}"#;
            let settings: Settings = serde_json::from_str(stored).unwrap();
            assert_eq!(settings.hotkey, "Alt+K");
        }

        #[test]
        fn round_trips_through_json_unchanged() {
            let original = Settings::default();
            let restored: Settings =
                serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap();
            assert_eq!(restored.hotkey, original.hotkey);
            assert_eq!(restored.settings_version, original.settings_version);
            assert_eq!(restored.appearance.accent, original.appearance.accent);
        }

        #[test]
        fn serialises_as_camel_case_for_the_webview() {
            let json = serde_json::to_string(&Settings::default()).unwrap();
            assert!(json.contains("autoTranslateClipboard"));
            assert!(json.contains("selectionHotkey"));
            assert!(!json.contains("auto_translate_clipboard"));
        }
    }
}

#[cfg(test)]
mod migration_tests {
    use super::*;
    use crate::secrets::MemoryStore;

    fn legacy_settings(key: &str) -> Settings {
        Settings {
            deepl_key: key.into(),
            settings_version: 1,
            ..Settings::default()
        }
    }

    #[test]
    fn migration_uses_the_keyring_only_after_verification() {
        let store = MemoryStore::default();
        let mut settings = legacy_settings("legacy-key");
        assert!(migrate_settings(&mut settings, &store));
        assert_eq!(settings.key_storage, KEYRING);
        assert_eq!(settings.settings_version, 2);
        assert_eq!(settings.deepl_key, "legacy-key");
        assert_eq!(store.get().unwrap().as_deref(), Some("legacy-key"));
        assert!(settings_for_disk(&settings).deepl_key.is_empty());
        assert!(!migrate_settings(&mut settings, &store));
    }

    #[test]
    fn migration_leaves_v1_intact_if_the_keyring_fails_verification() {
        for store in [
            MemoryStore {
                fail_set: true,
                ..MemoryStore::default()
            },
            MemoryStore {
                wrong_read: true,
                ..MemoryStore::default()
            },
        ] {
            let mut settings = legacy_settings("legacy-key");
            assert!(!migrate_settings(&mut settings, &store));
            assert_eq!(settings.settings_version, 1);
            assert_eq!(settings.key_storage, FILE);
            assert_eq!(settings_for_disk(&settings).deepl_key, "legacy-key");
        }
    }

    #[test]
    fn migration_keeps_empty_keys_out_of_the_keyring() {
        let store = MemoryStore::default();
        let mut settings: Settings =
            serde_json::from_str(r#"{"autoTranslateClipboard":true}"#).unwrap();
        assert!(migrate_settings(&mut settings, &store));
        assert!(!settings.auto_translate_clipboard);
        assert_eq!(settings.settings_version, 2);
        assert_eq!(settings.key_storage, FILE);
        assert_eq!(store.get().unwrap(), None);
    }

    #[test]
    fn opening_a_legacy_file_migrates_it_and_reloads_the_key_from_the_keyring() {
        let dir =
            std::env::temp_dir().join(format!("lexo-migrate-{}", crate::history::unique_id()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("settings.json");
        let store = MemoryStore::default();

        write_settings(&path, &legacy_settings("legacy-key")).unwrap();
        let settings = load_from_disk(&path, &store);
        assert_eq!(settings.deepl_key, "legacy-key");
        let persisted: Settings = read_json(&path);
        assert_eq!(persisted.key_storage, KEYRING);
        assert!(persisted.deepl_key.is_empty());
        assert_eq!(load_from_disk(&path, &store).deepl_key, "legacy-key");

        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn merge_keeps_internal_fields_and_applies_frontend_preferences() {
        let current = Settings {
            popup_x: Some(5),
            popup_y: Some(10),
            shortcuts_paused: true,
            key_storage: KEYRING.into(),
            quota_warned: true,
            ..Settings::default()
        };
        let updated = Settings {
            popup_x: Some(500),
            keep_history: false,
            ..Settings::default()
        };

        let merged = merge_settings(&current, updated);
        assert_eq!(merged.popup_x, Some(5));
        assert_eq!(merged.popup_y, Some(10));
        assert!(merged.shortcuts_paused);
        assert_eq!(merged.key_storage, KEYRING);
        assert!(merged.quota_warned);
        assert!(!merged.keep_history);
    }
}
