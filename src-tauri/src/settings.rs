use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::paths::{read_json, settings_path};

const DEFAULT_HOTKEY: &str = "Alt+R";
const DEFAULT_SELECTION_HOTKEY: &str = "Alt+T";
const DEFAULT_REPLACE_HOTKEY: &str = "Alt+Shift+T";
const DEFAULT_OCR_HOTKEY: &str = "Alt+S";
const DEFAULT_SWAP_HOTKEY: &str = "Alt+E";

const SETTINGS_VERSION: u32 = 1;

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
}

impl Default for Appearance {
    fn default() -> Self {
        Appearance {
            opacity: default_opacity(),
            bg: default_bg(),
            accent: default_accent(),
            text: default_text(),
            font: default_font(),
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

pub(crate) fn load_settings(app: &AppHandle) -> Settings {
    read_json(&settings_path(app))
}

pub(crate) fn persist_settings(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(settings_path(app), json).map_err(|e| e.to_string())
}

pub(crate) fn migrate_settings(app: &AppHandle) {
    if !settings_path(app).exists() {
        return;
    }
    let mut settings = load_settings(app);
    if settings.settings_version >= SETTINGS_VERSION {
        return;
    }
    settings.auto_translate_clipboard = false;
    settings.settings_version = SETTINGS_VERSION;
    let _ = persist_settings(app, &settings);
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    load_settings(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let current = load_settings(&app);
    let merged = Settings {
        popup_x: settings.popup_x.or(current.popup_x),
        popup_y: settings.popup_y.or(current.popup_y),
        shortcuts_paused: current.shortcuts_paused,
        settings_version: SETTINGS_VERSION,
        ..settings
    };
    persist_settings(&app, &merged)
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
