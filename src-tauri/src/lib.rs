use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, PhysicalPosition, WindowEvent,
};
use tauri_plugin_autostart::{ManagerExt, MacosLauncher};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

const DEFAULT_HOTKEY: &str = "Alt+R";
const DEFAULT_SELECTION_HOTKEY: &str = "Alt+T";

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Appearance {
    #[serde(default = "default_opacity")]
    opacity: f64,
    #[serde(default = "default_bg")]
    bg: String,
    #[serde(default = "default_accent")]
    accent: String,
    #[serde(default = "default_text")]
    text: String,
    #[serde(default = "default_font")]
    font: String,
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
struct Settings {
    #[serde(default = "default_hotkey")]
    hotkey: String,
    #[serde(default = "default_swap_hotkey")]
    swap_hotkey: String,
    // Global shortcut that translates the current selection from any app.
    #[serde(default = "default_selection_hotkey")]
    selection_hotkey: String,
    #[serde(default = "default_true")]
    auto_translate_clipboard: bool,
    // DeepL API key — the translation engine. Required for the app to translate.
    #[serde(default)]
    deepl_key: String,
    #[serde(default)]
    appearance: Appearance,
    #[serde(default)]
    popup_x: Option<i32>,
    #[serde(default)]
    popup_y: Option<i32>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            hotkey: default_hotkey(),
            swap_hotkey: default_swap_hotkey(),
            selection_hotkey: default_selection_hotkey(),
            auto_translate_clipboard: true,
            deepl_key: String::new(),
            appearance: Appearance::default(),
            popup_x: None,
            popup_y: None,
        }
    }
}

fn default_hotkey() -> String {
    DEFAULT_HOTKEY.to_string()
}
fn default_swap_hotkey() -> String {
    "Alt+E".to_string()
}
fn default_selection_hotkey() -> String {
    DEFAULT_SELECTION_HOTKEY.to_string()
}
fn default_true() -> bool {
    true
}
fn default_opacity() -> f64 {
    1.0
}
fn default_bg() -> String {
    "#F2EDE1".to_string()
}
fn default_accent() -> String {
    "#B98B3E".to_string()
}
fn default_text() -> String {
    "#1B2A4A".to_string()
}
fn default_font() -> String {
    "'Segoe UI', system-ui, sans-serif".to_string()
}

fn settings_path(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .expect("no app config dir");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("settings.json")
}

fn load_settings(app: &AppHandle) -> Settings {
    let path = settings_path(app);
    match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

#[tauri::command]
fn get_settings(app: AppHandle) -> Settings {
    load_settings(&app)
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let path = settings_path(&app);
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_clipboard(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_popup(app: AppHandle) {
    if let Some(win) = app.get_webview_window("popup") {
        let _ = win.hide();
    }
}

// Resize the popup so its height hugs the content (no empty space below).
#[tauri::command]
fn resize_window(app: AppHandle, height: f64) {
    if let Some(win) = app.get_webview_window("popup") {
        let _ = win.set_size(tauri::LogicalSize::new(520.0_f64, height.max(120.0)));
    }
}

// Resize the mini bubble to hug its content.
#[tauri::command]
fn resize_mini(app: AppHandle, height: f64) {
    if let Some(win) = app.get_webview_window("mini") {
        let _ = win.set_size(tauri::LogicalSize::new(360.0_f64, height.max(80.0)));
    }
}

#[tauri::command]
fn hide_mini(app: AppHandle) {
    if let Some(win) = app.get_webview_window("mini") {
        let _ = win.hide();
    }
}

fn save_popup_position(app: &AppHandle, position: &PhysicalPosition<i32>) {
    let mut config = load_settings(app);
    config.popup_x = Some(position.x);
    config.popup_y = Some(position.y);
    let _ = save_settings(app.clone(), config);
}

#[tauri::command]
fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|e| e.to_string())
    } else {
        manager.disable().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn get_autostart(app: AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

// Enable "start with Windows" by default, but only the very first time the app
// runs (tracked by a marker file). If the user later disables it, the marker
// stays, so we never re-enable it behind their back.
fn seed_autostart_default(app: &AppHandle) {
    if let Ok(dir) = app.path().app_config_dir() {
        let marker = dir.join(".autostart_seeded");
        if !marker.exists() {
            let _ = app.autolaunch().enable();
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(&marker, "1");
        }
    }
}

// Translate `text` with DeepL (best PT<->EN quality, fast). `key` is the user's
// DeepL Auth Key; free keys end with ":fx" and use the api-free host.
async fn translate_deepl(
    client: &reqwest::Client,
    text: &str,
    source: &str,
    target: &str,
    key: &str,
) -> Result<String, String> {
    // DeepL wants uppercase codes; English/Portuguese targets must be regional.
    let source_lang = source.to_uppercase();
    let target_lang = if target.to_lowercase().starts_with("en") {
        "EN-US".to_string()
    } else if target.to_lowercase().starts_with("pt") {
        "PT-BR".to_string()
    } else {
        target.to_uppercase()
    };

    let endpoint = if key.ends_with(":fx") {
        "https://api-free.deepl.com/v2/translate"
    } else {
        "https://api.deepl.com/v2/translate"
    };

    let body = serde_json::json!({
        "text": [text],
        "source_lang": source_lang,
        "target_lang": target_lang,
    });

    let resp = client
        .post(endpoint)
        .timeout(std::time::Duration::from_secs(8))
        .header("Authorization", format!("DeepL-Auth-Key {key}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("network_error: {e}"))?;

    // 403 = bad/rejected key; 456 = monthly quota exhausted.
    if resp.status().as_u16() == 403 {
        return Err(
            "deepl_auth: DeepL rejected the key. Check the DeepL key in Settings.".to_string(),
        );
    }
    if resp.status().as_u16() == 456 {
        return Err(
            "limit: DeepL monthly character quota reached. It resets next month.".to_string(),
        );
    }
    if !resp.status().is_success() {
        return Err(format!("http_error: DeepL http {}", resp.status()));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("decode_error: {e}"))?;

    let translated = data["translations"][0]["text"]
        .as_str()
        .unwrap_or_default();

    if translated.is_empty() {
        return Err("api_error: empty response from DeepL".to_string());
    }

    Ok(translated.to_string())
}

// Translate `text` via DeepL. `langpair` is e.g. "pt|en" or "en|pt".
// A DeepL API key (set in Settings) is required.
#[tauri::command]
async fn translate(app: AppHandle, text: String, langpair: String) -> Result<String, String> {
    let cfg = load_settings(&app);
    let deepl_key = cfg.deepl_key.trim();
    if deepl_key.is_empty() {
        return Err("no_key: Add your DeepL API key in Settings to translate.".to_string());
    }

    let mut parts = langpair.splitn(2, '|');
    let source = parts.next().unwrap_or("en");
    let target = parts.next().unwrap_or("pt");

    let client = reqwest::Client::new();
    translate_deepl(&client, &text, source, target, deepl_key).await
}

// Place the popup near the top-right of the screen, ~80px from the top and
// right edges (accounting for display scaling and multi-monitor offsets).
fn place_top_right(win: &tauri::WebviewWindow) {
    if let Ok(Some(monitor)) = win.current_monitor() {
        let scale = monitor.scale_factor();
        let m_pos = monitor.position();
        let m_size = monitor.size();
        let win_size = win
            .outer_size()
            .unwrap_or(tauri::PhysicalSize::new(520, 250));
        let margin = (80.0 * scale) as i32;
        let x = m_pos.x + m_size.width as i32 - win_size.width as i32 - margin;
        let y = m_pos.y + margin;
        let _ = win.set_position(PhysicalPosition::new(x, y));
    }
}

fn show_popup(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("popup") {
        let cfg = load_settings(app);
        // Restore a saved position if the user moved the window before; otherwise
        // default to the top-right corner.
        if let (Some(x), Some(y)) = (cfg.popup_x, cfg.popup_y) {
            let _ = win.set_position(PhysicalPosition::new(x, y));
        } else {
            place_top_right(&win);
        }
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();

        let clipboard = app.clipboard().read_text().unwrap_or_default();
        let _ = win.emit("popup-shown", serde_json::json!({ "clipboard": clipboard }));
    }
}

fn toggle_popup(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("popup") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
        } else {
            show_popup(app);
        }
    }
}

// Translate whatever is currently selected in ANY app:
// simulate Ctrl+C, read the clipboard, show the popup at the cursor, and
// hand the captured text to the frontend to translate. The user's previous
// clipboard contents are restored afterwards.
fn translate_selection(app: &AppHandle) {
    use enigo::{Direction, Enigo, Key, Keyboard, Mouse, Settings as EnigoSettings};

    let app = app.clone();
    std::thread::spawn(move || {
        let mut enigo = match Enigo::new(&EnigoSettings::default()) {
            Ok(e) => e,
            Err(_) => return,
        };

        // Remember the cursor position before anything moves.
        let cursor = enigo.location().ok();

        // Preserve the user's current clipboard and detect whether the copy
        // action produced new selection data.
        let original = app.clipboard().read_text().ok();
        let sentinel = format!("__tradutor_marker_{}__", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        let _ = app.clipboard().write_text(&sentinel);

        // Simulate copy on the focused external app.
        #[cfg(target_os = "windows")]
        let primary_copy_key = Key::C;
        #[cfg(target_os = "windows")]
        let fallback_copy_key = Some(Key::Insert);
        #[cfg(not(target_os = "windows"))]
        let primary_copy_key = Key::Unicode('c');
        #[cfg(not(target_os = "windows"))]
        let fallback_copy_key = None;

        let mut copy_with = |key| {
            let _ = enigo.key(Key::Control, Direction::Press);
            std::thread::sleep(std::time::Duration::from_millis(25));
            let _ = enigo.key(key, Direction::Click);
            std::thread::sleep(std::time::Duration::from_millis(25));
            let _ = enigo.key(Key::Control, Direction::Release);
        };

        copy_with(primary_copy_key);
        std::thread::sleep(std::time::Duration::from_millis(200));
        let mut selected = app.clipboard().read_text().unwrap_or_default();

        if selected == sentinel || selected.is_empty() {
            if let Some(key) = fallback_copy_key {
                copy_with(key);
                std::thread::sleep(std::time::Duration::from_millis(200));
                selected = app.clipboard().read_text().unwrap_or_default();
            }
        }

        let text_for_mini = if selected != sentinel && !selected.is_empty() {
            selected
        } else {
            String::new()
        };

        // Show the compact "mini" bubble at the cursor and hand it the text.
        if let Some(win) = app.get_webview_window("mini") {
            if let Some((x, y)) = cursor {
                // Place the bubble slightly below and to the right of the cursor,
                // similar to a translation tooltip.
                let _ = win.set_position(tauri::PhysicalPosition::new(x + 16, y + 26));
            }
            let _ = win.show();
            let _ = win.set_focus();
            let _ = app.emit_to("mini", "mini-translate", text_for_mini);
        }

        // Restore the original clipboard.
        if let Some(orig) = original {
            std::thread::sleep(std::time::Duration::from_millis(60));
            let _ = app.clipboard().write_text(orig);
        }
    });
}

// (Re)register the global shortcuts from the current settings. Called at
// startup and again whenever the user saves new hotkeys, so changes take effect
// immediately — no app restart. The in-app swap hotkey (Alt+E) is handled in the
// webview, not as a global shortcut, so it is not registered here.
fn register_global_shortcuts(app: &AppHandle, cfg: &Settings) -> Result<(), String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();

    let open: Shortcut = cfg
        .hotkey
        .parse()
        .map_err(|_| format!("'{}' is not a valid shortcut", cfg.hotkey))?;
    gs.register(open)
        .map_err(|e| format!("Could not register '{}': {e}", cfg.hotkey))?;

    let selection: Shortcut = cfg
        .selection_hotkey
        .parse()
        .map_err(|_| format!("'{}' is not a valid shortcut", cfg.selection_hotkey))?;
    if selection != open {
        gs.register(selection)
            .map_err(|e| format!("Could not register '{}': {e}", cfg.selection_hotkey))?;
    }
    Ok(())
}

// Apply the currently-saved hotkeys. The frontend calls this right after saving
// settings so shortcut changes are live.
#[tauri::command]
fn update_shortcuts(app: AppHandle) -> Result<(), String> {
    let cfg = load_settings(&app);
    register_global_shortcuts(&app, &cfg)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
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
                    let cfg = load_settings(app);
                    if let Ok(sel) = cfg.selection_hotkey.parse::<Shortcut>() {
                        if shortcut == &sel {
                            translate_selection(app);
                            return;
                        }
                    }
                    // Default action (open shortcut, or any other registered one).
                    toggle_popup(app);
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            translate,
            set_clipboard,
            hide_popup,
            resize_window,
            resize_mini,
            hide_mini,
            set_autostart,
            get_autostart,
            update_shortcuts
        ])
        .setup(|app| {
            // On macOS, run as a background/menu-bar agent (no dock icon).
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle().clone();

            // Register the configured global shortcuts. If the saved combos are
            // invalid or unavailable, fall back to the defaults so the app stays
            // openable.
            let cfg = load_settings(&handle);
            if let Err(e) = register_global_shortcuts(&handle, &cfg) {
                eprintln!("Shortcut registration issue: {e}");
                if let Ok(def) = DEFAULT_HOTKEY.parse::<Shortcut>() {
                    let _ = app.global_shortcut().register(def);
                }
                if let Ok(def_sel) = DEFAULT_SELECTION_HOTKEY.parse::<Shortcut>() {
                    let _ = app.global_shortcut().register(def_sel);
                }
            }

            // Enable "start with Windows" by default on first run.
            seed_autostart_default(&handle);

            // System tray.
            let open_i = MenuItem::with_id(app, "open", "Open Lexo", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &settings_i, &quit_i])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Lexo — PT ⇄ EN")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_popup(app),
                    "settings" => {
                        show_popup(app);
                        if let Some(win) = app.get_webview_window("popup") {
                            let _ = win.emit("open-settings", ());
                        }
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

            // Open on the screen at launch (top-right) instead of staying hidden
            // in the tray.
            show_popup(&handle);

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    let _ = window.hide();
                    api.prevent_close();
                }
                WindowEvent::Moved(position) => {
                    if window.label() == "popup" {
                        save_popup_position(&window.app_handle(), &position);
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
