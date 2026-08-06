use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::settings::load_settings;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MiniPayload {
    pub(crate) text: String,
    pub(crate) origin: String,
    pub(crate) error: Option<String>,
}

fn clamp_to_screen(win: &WebviewWindow, x: i32, y: i32, width: i32, height: i32) -> (i32, i32) {
    const MARGIN: i32 = 12;

    let mut area: Option<(i32, i32, i32, i32)> = None;

    if let Ok(monitors) = win.available_monitors() {
        for monitor in &monitors {
            let position = monitor.position();
            let size = monitor.size();
            if x >= position.x
                && x < position.x + size.width as i32
                && y >= position.y
                && y < position.y + size.height as i32
            {
                let usable = monitor.work_area();
                area = Some((
                    usable.position.x,
                    usable.position.y,
                    usable.size.width as i32,
                    usable.size.height as i32,
                ));
                break;
            }
        }
    }

    if area.is_none() {
        if let Ok(Some(monitor)) = win.primary_monitor() {
            let usable = monitor.work_area();
            area = Some((
                usable.position.x,
                usable.position.y,
                usable.size.width as i32,
                usable.size.height as i32,
            ));
        }
    }

    match area {
        Some((ax, ay, aw, ah)) => (
            x.min(ax + aw - width - MARGIN).max(ax + MARGIN),
            y.min(ay + ah - height - MARGIN).max(ay + MARGIN),
        ),
        None => (x, y),
    }
}

pub(crate) fn show_mini(app: &AppHandle, anchor: Option<(i32, i32)>, payload: MiniPayload) {
    let Some(win) = app.get_webview_window("mini") else {
        return;
    };
    if let Some((x, y)) = anchor {
        let size = win.outer_size().unwrap_or(PhysicalSize::new(360, 180));
        let (x, y) = clamp_to_screen(&win, x, y, size.width as i32, size.height as i32);
        let _ = win.set_position(PhysicalPosition::new(x, y));
    }
    let _ = win.show();
    let _ = win.set_focus();
    let _ = app.emit_to("mini", "mini-translate", payload);
}

fn place_top_right(win: &WebviewWindow) {
    if let Ok(Some(monitor)) = win.current_monitor() {
        let scale = monitor.scale_factor();
        let position = monitor.position();
        let size = monitor.size();
        let window = win.outer_size().unwrap_or(PhysicalSize::new(520, 250));
        let margin = (80.0 * scale) as i32;
        let x = position.x + size.width as i32 - window.width as i32 - margin;
        let y = position.y + margin;
        let _ = win.set_position(PhysicalPosition::new(x, y));
    }
}

pub(crate) fn show_popup(app: &AppHandle) {
    let Some(win) = app.get_webview_window("popup") else {
        return;
    };
    let settings = load_settings(app);

    match (settings.popup_x, settings.popup_y) {
        (Some(x), Some(y)) => {
            let _ = win.set_position(PhysicalPosition::new(x, y));
        }
        _ => place_top_right(&win),
    }

    let _ = win.show();
    let _ = win.unminimize();
    let _ = win.set_focus();

    let clipboard = if settings.auto_translate_clipboard {
        app.clipboard().read_text().unwrap_or_default()
    } else {
        String::new()
    };
    let _ = win.emit("popup-shown", serde_json::json!({ "clipboard": clipboard }));
}

pub(crate) fn toggle_popup(app: &AppHandle) {
    let Some(win) = app.get_webview_window("popup") else {
        return;
    };
    if win.is_visible().unwrap_or(false) {
        let _ = win.hide();
    } else {
        show_popup(app);
    }
}

#[tauri::command]
pub fn hide_popup(app: AppHandle) {
    if let Some(win) = app.get_webview_window("popup") {
        let _ = win.hide();
    }
}

#[tauri::command]
pub fn hide_mini(app: AppHandle) {
    if let Some(win) = app.get_webview_window("mini") {
        let _ = win.hide();
    }
}

#[tauri::command]
pub fn resize_window(app: AppHandle, height: f64) {
    if let Some(win) = app.get_webview_window("popup") {
        let _ = win.set_size(tauri::LogicalSize::new(520.0_f64, height.max(120.0)));
    }
}

#[tauri::command]
pub fn resize_mini(app: AppHandle, height: f64) {
    let Some(win) = app.get_webview_window("mini") else {
        return;
    };
    let height = height.max(80.0);
    let _ = win.set_size(tauri::LogicalSize::new(360.0_f64, height));

    let scale = win.scale_factor().unwrap_or(1.0);
    let width = (360.0 * scale).round() as i32;
    let height = (height * scale).round() as i32;

    if let Ok(position) = win.outer_position() {
        let (x, y) = clamp_to_screen(&win, position.x, position.y, width, height);
        if (x, y) != (position.x, position.y) {
            let _ = win.set_position(PhysicalPosition::new(x, y));
        }
    }
}

#[tauri::command]
pub fn set_clipboard(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}
