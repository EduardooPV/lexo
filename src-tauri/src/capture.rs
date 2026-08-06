use std::time::Duration;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};

use crate::selection::cursor_position;

use crate::windows::{show_mini, MiniPayload};

#[tauri::command]
pub fn ocr_available() -> bool {
    crate::ocr::available()
}

#[tauri::command]
pub fn cancel_region_capture(app: AppHandle) {
    if let Some(win) = app.get_webview_window("overlay") {
        let _ = win.hide();
    }
}

#[tauri::command]
pub fn start_region_capture(app: AppHandle) {
    if !crate::ocr::available() {
        return;
    }
    let Some(win) = app.get_webview_window("overlay") else {
        return;
    };

    let monitors = win.available_monitors().unwrap_or_default();
    let target = cursor_position()
        .and_then(|(x, y)| {
            monitors
                .iter()
                .map(|monitor| {
                    (
                        monitor.position().x,
                        monitor.position().y,
                        monitor.size().width,
                        monitor.size().height,
                    )
                })
                .find(|&(mx, my, mw, mh)| {
                    x >= mx && x < mx + mw as i32 && y >= my && y < my + mh as i32
                })
        })
        .or_else(|| {
            win.primary_monitor().ok().flatten().map(|monitor| {
                (
                    monitor.position().x,
                    monitor.position().y,
                    monitor.size().width,
                    monitor.size().height,
                )
            })
        });

    if let Some((x, y, width, height)) = target {
        let _ = win.set_position(PhysicalPosition::new(x, y));
        let _ = win.set_size(PhysicalSize::new(width, height));
    }

    let _ = win.show();
    let _ = win.set_always_on_top(true);
    let _ = win.set_focus();
}

#[tauri::command]
pub fn ocr_region(app: AppHandle, x: f64, y: f64, width: f64, height: f64) {
    let Some(overlay) = app.get_webview_window("overlay") else {
        return;
    };
    let scale = overlay.scale_factor().unwrap_or(1.0);
    let origin = overlay
        .outer_position()
        .unwrap_or(PhysicalPosition::new(0, 0));
    let _ = overlay.hide();

    let px = origin.x + (x * scale).round() as i32;
    let py = origin.y + (y * scale).round() as i32;
    let pw = (width * scale).round() as i32;
    let ph = (height * scale).round() as i32;

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(140));

        let anchor = Some((px, py + ph + 12));
        match crate::ocr::recognize(px, py, pw, ph) {
            Ok(text) => show_mini(
                &app,
                anchor,
                MiniPayload {
                    text,
                    origin: "ocr".to_string(),
                    error: None,
                },
            ),
            Err(error) => show_mini(
                &app,
                anchor,
                MiniPayload {
                    text: String::new(),
                    origin: "ocr".to_string(),
                    error: Some(error),
                },
            ),
        }
    });
}
