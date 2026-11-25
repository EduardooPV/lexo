use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::settings::{load_settings, Settings};
use crate::windows::show_popup;

pub(crate) fn build_tray_menu(app: &AppHandle, paused: bool) -> tauri::Result<Menu<Wry>> {
    let open = MenuItem::with_id(app, "open", "Open Lexo", true, None::<&str>)?;
    let history = MenuItem::with_id(app, "history", "History", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let pause = MenuItem::with_id(
        app,
        "pause",
        if paused {
            "Resume shortcuts"
        } else {
            "Pause shortcuts"
        },
        true,
        None::<&str>,
    )?;
    let first = PredefinedMenuItem::separator(app)?;
    let second = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    Menu::with_items(
        app,
        &[&open, &history, &settings, &first, &pause, &second, &quit],
    )
}

pub(crate) fn tray_tooltip(settings: &Settings) -> String {
    format!(
        "Lexo — {} ⇄ {}",
        settings.primary_lang, settings.secondary_lang
    )
}

pub(crate) fn refresh_tray_tooltip(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(tray_tooltip(&load_settings(app))));
    }
}

pub(crate) fn refresh_tray(app: &AppHandle, paused: bool) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(menu) = build_tray_menu(app, paused) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

pub(crate) fn open_view(app: &AppHandle, view: &str) {
    show_popup(app);
    if let Some(win) = app.get_webview_window("popup") {
        let _ = win.emit("open-view", view.to_string());
    }
}
