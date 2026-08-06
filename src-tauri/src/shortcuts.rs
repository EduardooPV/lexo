use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::settings::{load_settings, persist_settings, Settings};
use crate::tray::refresh_tray;

pub(crate) fn register_global_shortcuts(
    app: &AppHandle,
    settings: &Settings,
) -> Result<(), String> {
    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister_all();

    if settings.shortcuts_paused {
        return Ok(());
    }

    let mut wanted = vec![
        ("Open", settings.hotkey.as_str()),
        ("Translate selection", settings.selection_hotkey.as_str()),
        ("Translate and replace", settings.replace_hotkey.as_str()),
    ];
    if crate::ocr::available() {
        wanted.push(("Screen OCR", settings.ocr_hotkey.as_str()));
    }

    let mut registered: Vec<Shortcut> = Vec::new();
    for (label, combo) in wanted {
        let combo = combo.trim();
        if combo.is_empty() {
            continue;
        }
        let parsed: Shortcut = combo
            .parse()
            .map_err(|_| format!("{label}: '{combo}' is not a valid shortcut"))?;
        if registered.contains(&parsed) {
            continue;
        }
        shortcuts
            .register(parsed)
            .map_err(|e| format!("{label}: could not register '{combo}' ({e})"))?;
        registered.push(parsed);
    }
    Ok(())
}

#[tauri::command]
pub fn update_shortcuts(app: AppHandle) -> Result<(), String> {
    let settings = load_settings(&app);
    register_global_shortcuts(&app, &settings)
}

#[tauri::command]
pub fn get_shortcuts_paused(app: AppHandle) -> bool {
    load_settings(&app).shortcuts_paused
}

#[tauri::command]
pub fn set_shortcuts_paused(app: AppHandle, paused: bool) -> Result<(), String> {
    let mut settings = load_settings(&app);
    settings.shortcuts_paused = paused;
    persist_settings(&app, &settings)?;
    refresh_tray(&app, paused);
    register_global_shortcuts(&app, &settings)
}
