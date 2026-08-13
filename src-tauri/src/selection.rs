use std::time::{Duration, Instant};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::deepl::translate_text;
use crate::history::unique_id;
use crate::windows::{show_mini, MiniPayload};

/// How long to wait for a simulated copy to actually reach the clipboard. A
/// plain edit box answers in a few milliseconds, but Chromium- and Qt-based
/// apps (Calibre's viewer among them) route the copy through another process
/// first and can take the better part of a second on a busy machine.
const COPY_TIMEOUT: Duration = Duration::from_millis(800);
/// The second attempt only has to cover apps that ignored the first one.
const FALLBACK_TIMEOUT: Duration = Duration::from_millis(400);
const CLIPBOARD_POLL: Duration = Duration::from_millis(25);
/// How long to give the user to let go of the hotkey's own modifiers.
const MODIFIER_TIMEOUT: Duration = Duration::from_millis(500);

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

/// Whether any modifier is physically held right now. Only Windows can answer;
/// everywhere else this stays `false` and the wait below is a no-op.
#[cfg(target_os = "windows")]
fn modifiers_down() -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
    };
    // Bit 15 is "down right now"; bit 0 is "was pressed since the last call",
    // which would still be set for a key the user has already released.
    unsafe {
        [VK_MENU, VK_CONTROL, VK_SHIFT, VK_LWIN, VK_RWIN]
            .iter()
            .any(|key| GetAsyncKeyState(*key as i32) as u16 & 0x8000 != 0)
    }
}

#[cfg(not(target_os = "windows"))]
fn modifiers_down() -> bool {
    false
}

/// The hotkey that got us here is a modifier combo (Alt+T by default) and the
/// copy is simulated milliseconds later, while those modifiers are usually
/// still held down. The app being copied from then sees Ctrl+Alt+C instead of
/// Ctrl+C, and anything that matches its shortcuts exactly — Chromium- and
/// Qt-based apps, so Calibre's viewer too — does nothing at all. Wait for the
/// keys to come up on their own, then force them up so a plain Ctrl+C lands.
fn release_modifiers(enigo: &mut enigo::Enigo) {
    use enigo::{Direction, Key, Keyboard};

    let deadline = Instant::now() + MODIFIER_TIMEOUT;
    while modifiers_down() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(15));
    }

    // Only forced when they really are still down: a key-up for a key the app
    // never saw pressed is how you make it open its menu bar instead.
    if !modifiers_down() {
        return;
    }
    for key in [Key::Alt, Key::Shift, Key::Control, Key::Meta] {
        let _ = enigo.key(key, Direction::Release);
    }
    std::thread::sleep(Duration::from_millis(40));
}

/// Calls `read` until it answers or the timeout passes.
fn poll_for<F>(timeout: Duration, mut read: F) -> Option<String>
where
    F: FnMut() -> Option<String>,
{
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(value) = read() {
            return Some(value);
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(CLIPBOARD_POLL);
    }
}

/// Waits for the copy to land: anything on the clipboard that is neither the
/// marker nor empty. A fixed sleep used to stand in for this, and a read that
/// failed — which is what happens while the other app still owns the clipboard
/// — counted as "nothing was selected", so a perfectly good selection came back
/// empty.
fn await_copy(app: &AppHandle, sentinel: &str, timeout: Duration) -> Option<String> {
    poll_for(timeout, || {
        let text = app.clipboard().read_text().ok()?;
        (text != sentinel && !text.is_empty()).then_some(text)
    })
}

/// Puts the marker on the clipboard and makes sure it stuck — the write can be
/// swallowed while another process holds the clipboard, and everything after
/// this reads "still the marker" as "nothing was copied".
fn write_sentinel(app: &AppHandle, sentinel: &str) {
    for _ in 0..4 {
        if app.clipboard().write_text(sentinel).is_ok()
            && app
                .clipboard()
                .read_text()
                .map(|text| text == sentinel)
                .unwrap_or(false)
        {
            return;
        }
        std::thread::sleep(CLIPBOARD_POLL);
    }
}

fn capture_selection(app: &AppHandle, enigo: &mut enigo::Enigo) -> Captured {
    use enigo::{Key, Mouse};

    // Read before the waiting starts: the bubble belongs where the cursor was
    // when the hotkey fired, not where it drifted to afterwards.
    let cursor = enigo.location().ok();
    let original = app.clipboard().read_text().ok();
    let sentinel = format!("__lexo_marker_{}__", unique_id());
    write_sentinel(app, &sentinel);

    release_modifiers(enigo);

    #[cfg(target_os = "windows")]
    let (primary, fallback) = (Key::C, Some(Key::Insert));
    #[cfg(not(target_os = "windows"))]
    let (primary, fallback) = (Key::Unicode('c'), None);

    press_combo(enigo, primary);
    let mut selected = await_copy(app, &sentinel, COPY_TIMEOUT);

    // Ctrl+Insert is the older copy binding, and some apps still answer only
    // to that one.
    if selected.is_none() {
        if let Some(key) = fallback {
            press_combo(enigo, key);
            selected = await_copy(app, &sentinel, FALLBACK_TIMEOUT);
        }
    }

    Captured {
        cursor,
        original,
        text: selected.unwrap_or_default(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    mod poll_for {
        use super::*;

        #[test]
        fn keeps_reading_until_the_copy_lands() {
            let attempts = Cell::new(0);
            let found = poll_for(Duration::from_secs(2), || {
                attempts.set(attempts.get() + 1);
                (attempts.get() >= 3).then(|| "o texto copiado".to_string())
            });

            assert_eq!(found.as_deref(), Some("o texto copiado"));
            assert_eq!(attempts.get(), 3);
        }

        #[test]
        fn takes_the_first_answer_without_waiting() {
            let started = Instant::now();
            let found = poll_for(Duration::from_secs(2), || Some("pronto".to_string()));

            assert_eq!(found.as_deref(), Some("pronto"));
            assert!(started.elapsed() < CLIPBOARD_POLL);
        }

        #[test]
        fn retries_before_giving_up_on_the_deadline() {
            let attempts = Cell::new(0);
            let started = Instant::now();
            let found = poll_for(Duration::from_millis(120), || {
                attempts.set(attempts.get() + 1);
                None
            });

            assert!(found.is_none());
            assert!(attempts.get() > 1, "a single read is not a wait");
            assert!(started.elapsed() >= Duration::from_millis(120));
        }
    }
}
