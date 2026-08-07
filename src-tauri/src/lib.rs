mod autostart;
mod cache;
mod capture;
mod deepl;
mod history;
mod language;
mod ocr;
mod paths;
mod selection;
mod settings;
mod shortcuts;
mod state;
mod tray;
mod updater;
mod windows;

use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};

use crate::autostart::seed_or_refresh_autostart;
use crate::capture::start_region_capture;
use crate::selection::{replace_selection, translate_selection};
use crate::settings::{load_settings, migrate_settings, persist_settings, Settings};
use crate::shortcuts::{register_global_shortcuts, set_shortcuts_paused};
use crate::state::{load_state, AppState};
use crate::tray::{build_tray_menu, open_view};
use crate::updater::check_for_update_on_startup;
use crate::windows::{show_popup, toggle_popup};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let settings = load_settings(app);
                    if settings.shortcuts_paused {
                        return;
                    }
                    let matches = |combo: &str| {
                        combo
                            .trim()
                            .parse::<Shortcut>()
                            .map(|parsed| &parsed == shortcut)
                            .unwrap_or(false)
                    };

                    if matches(&settings.selection_hotkey) {
                        translate_selection(app);
                    } else if matches(&settings.replace_hotkey) {
                        replace_selection(app);
                    } else if matches(&settings.ocr_hotkey) {
                        start_region_capture(app.clone());
                    } else {
                        toggle_popup(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            settings::get_settings,
            settings::save_settings,
            shortcuts::update_shortcuts,
            shortcuts::get_shortcuts_paused,
            shortcuts::set_shortcuts_paused,
            deepl::translate,
            deepl::get_usage,
            deepl::verify_deepl_key,
            deepl::open_deepl_signup,
            history::get_history,
            history::clear_history,
            history::delete_history_entry,
            history::toggle_history_pin,
            windows::set_clipboard,
            windows::hide_popup,
            windows::hide_mini,
            windows::resize_window,
            windows::resize_mini,
            autostart::set_autostart,
            autostart::get_autostart,
            capture::ocr_available,
            capture::ocr_region,
            capture::start_region_capture,
            capture::cancel_region_capture,
            updater::check_for_update,
            updater::pending_update,
            updater::app_version,
            updater::install_update
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle().clone();

            load_state(&handle);
            migrate_settings(&handle);

            let settings = load_settings(&handle);
            if let Err(error) = register_global_shortcuts(&handle, &settings) {
                eprintln!("Shortcut registration issue: {error}");
                let fallback = Settings::default();
                let _ = register_global_shortcuts(&handle, &fallback);
            }

            seed_or_refresh_autostart(&handle);
            check_for_update_on_startup(&handle);

            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Lexo — PT ⇄ EN")
                .menu(&build_tray_menu(&handle, settings.shortcuts_paused)?)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_popup(app),
                    "history" => open_view(app, "history"),
                    "settings" => open_view(app, "settings"),
                    "pause" => {
                        let paused = !load_settings(app).shortcuts_paused;
                        let _ = set_shortcuts_paused(app.clone(), paused);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_popup(tray.app_handle());
                    }
                })
                .build(app)?;

            show_popup(&handle);

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                let _ = window.hide();
                api.prevent_close();
            }
            WindowEvent::Moved(position) if window.label() == "popup" => {
                let app = window.app_handle();
                let mut settings = load_settings(app);
                settings.popup_x = Some(position.x);
                settings.popup_y = Some(position.y);
                let _ = persist_settings(app, &settings);
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
