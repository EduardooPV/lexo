use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::deepl::translate_text;
use crate::history::unique_id;
use crate::windows::{show_mini, MiniPayload};

struct Captured {
    cursor: Option<(i32, i32)>,
    original: Option<String>,
    text: String,
}

fn press_combo(enigo: &mut enigo::Enigo, key: enigo::Key) {
    use enigo::{Direction, Key, Keyboard};
    let _ = enigo.key(Key::Control, Direction::Press);
    std::thread::sleep(Duration::from_millis(25));
    let _ = enigo.key(key, Direction::Click);
    std::thread::sleep(Duration::from_millis(25));
    let _ = enigo.key(Key::Control, Direction::Release);
}

fn capture_selection(app: &AppHandle, enigo: &mut enigo::Enigo) -> Captured {
    use enigo::{Key, Mouse};

    let cursor = enigo.location().ok();
    let original = app.clipboard().read_text().ok();
    let sentinel = format!("__lexo_marker_{}__", unique_id());
    let _ = app.clipboard().write_text(&sentinel);

    #[cfg(target_os = "windows")]
    let (primary, fallback) = (Key::C, Some(Key::Insert));
    #[cfg(not(target_os = "windows"))]
    let (primary, fallback) = (Key::Unicode('c'), None);

    press_combo(enigo, primary);
    std::thread::sleep(Duration::from_millis(200));
    let mut selected = app.clipboard().read_text().unwrap_or_default();

    if selected == sentinel || selected.is_empty() {
        if let Some(key) = fallback {
            press_combo(enigo, key);
            std::thread::sleep(Duration::from_millis(200));
            selected = app.clipboard().read_text().unwrap_or_default();
        }
    }

    Captured {
        cursor,
        original,
        text: if selected == sentinel {
            String::new()
        } else {
            selected
        },
    }
}

fn restore_clipboard(app: &AppHandle, original: &Option<String>) {
    if let Some(text) = original {
        std::thread::sleep(Duration::from_millis(60));
        let _ = app.clipboard().write_text(text.clone());
    }
}

fn new_enigo() -> Option<enigo::Enigo> {
    enigo::Enigo::new(&enigo::Settings::default()).ok()
}

pub(crate) fn cursor_position() -> Option<(i32, i32)> {
    use enigo::Mouse;
    new_enigo()?.location().ok()
}

pub(crate) fn translate_selection(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let Some(mut enigo) = new_enigo() else {
            return;
        };
        let captured = capture_selection(&app, &mut enigo);
        restore_clipboard(&app, &captured.original);

        show_mini(
            &app,
            captured.cursor.map(|(x, y)| (x + 16, y + 26)),
            MiniPayload {
                text: captured.text,
                origin: "selection".to_string(),
                error: None,
            },
        );
    });
}

pub(crate) fn replace_selection(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let Some(mut enigo) = new_enigo() else {
            return;
        };
        let captured = capture_selection(&app, &mut enigo);
        let anchor = captured.cursor.map(|(x, y)| (x + 16, y + 26));

        if captured.text.trim().is_empty() {
            restore_clipboard(&app, &captured.original);
            show_mini(
                &app,
                anchor,
                MiniPayload {
                    text: String::new(),
                    origin: "selection".to_string(),
                    error: None,
                },
            );
            return;
        }

        match tauri::async_runtime::block_on(translate_text(&app, captured.text.clone(), None)) {
            Ok(result) => {
                let _ = app.clipboard().write_text(result.text);
                std::thread::sleep(Duration::from_millis(80));
                #[cfg(target_os = "windows")]
                press_combo(&mut enigo, enigo::Key::V);
                #[cfg(not(target_os = "windows"))]
                press_combo(&mut enigo, enigo::Key::Unicode('v'));
                std::thread::sleep(Duration::from_millis(220));
                restore_clipboard(&app, &captured.original);
            }
            Err(error) => {
                restore_clipboard(&app, &captured.original);
                show_mini(
                    &app,
                    anchor,
                    MiniPayload {
                        text: captured.text,
                        origin: "selection".to_string(),
                        error: Some(error),
                    },
                );
            }
        }
    });
}
