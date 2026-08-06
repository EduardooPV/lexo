mod ocr;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow, WindowEvent, Wry,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

const DEFAULT_HOTKEY: &str = "Alt+R";
const DEFAULT_SELECTION_HOTKEY: &str = "Alt+T";
const DEFAULT_REPLACE_HOTKEY: &str = "Alt+Shift+T";
const DEFAULT_OCR_HOTKEY: &str = "Alt+S";
const DEFAULT_SWAP_HOTKEY: &str = "Alt+E";

const HISTORY_LIMIT: usize = 200;
const CACHE_LIMIT: usize = 400;

const SETTINGS_VERSION: u32 = 1;

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
    #[serde(default = "default_selection_hotkey")]
    selection_hotkey: String,
    #[serde(default = "default_replace_hotkey")]
    replace_hotkey: String,
    #[serde(default = "default_ocr_hotkey")]
    ocr_hotkey: String,
    #[serde(default)]
    auto_translate_clipboard: bool,
    #[serde(default)]
    shortcuts_paused: bool,
    #[serde(default)]
    deepl_key: String,
    #[serde(default)]
    appearance: Appearance,
    #[serde(default)]
    popup_x: Option<i32>,
    #[serde(default)]
    popup_y: Option<i32>,
    #[serde(default)]
    settings_version: u32,
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

fn config_dir(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_config_dir().expect("no app config dir");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn settings_path(app: &AppHandle) -> PathBuf {
    config_dir(app).join("settings.json")
}
fn history_path(app: &AppHandle) -> PathBuf {
    config_dir(app).join("history.json")
}
fn cache_path(app: &AppHandle) -> PathBuf {
    config_dir(app).join("cache.json")
}

fn read_json<T: serde::de::DeserializeOwned + Default>(path: &PathBuf) -> T {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn write_json<T: Serialize>(path: &PathBuf, value: &T) {
    if let Ok(json) = serde_json::to_string(value) {
        let _ = std::fs::write(path, json);
    }
}

fn load_settings(app: &AppHandle) -> Settings {
    read_json(&settings_path(app))
}

fn persist_settings(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(settings_path(app), json).map_err(|e| e.to_string())
}

fn migrate_settings(app: &AppHandle) {
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

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn unique_id() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct HistoryEntry {
    id: String,
    source: String,
    translated: String,
    from: String,
    to: String,
    at: u64,
    #[serde(default)]
    pinned: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct CacheEntry {
    key: String,
    text: String,
    detected: String,
}

#[derive(Default)]
struct AppState {
    history: Mutex<Vec<HistoryEntry>>,
    cache: Mutex<Vec<CacheEntry>>,
    pending_update: Mutex<Option<tauri_plugin_updater::Update>>,
}

fn load_state(app: &AppHandle) {
    let state = app.state::<AppState>();

    let history = state.history.lock();
    if let Ok(mut history) = history {
        *history = read_json(&history_path(app));
    }

    let cache = state.cache.lock();
    if let Ok(mut cache) = cache {
        *cache = read_json(&cache_path(app));
    }
}

fn cache_key(text: &str, target: &str) -> String {
    format!("{target}\u{1}{text}")
}

fn cache_lookup(app: &AppHandle, text: &str, target: &str) -> Option<CacheEntry> {
    let key = cache_key(text, target);
    let state = app.state::<AppState>();
    let cache = state.cache.lock().ok()?;
    cache.iter().find(|entry| entry.key == key).cloned()
}

fn cache_store(app: &AppHandle, text: &str, target: &str, translated: &str, detected: &str) {
    let key = cache_key(text, target);
    let state = app.state::<AppState>();
    let Ok(mut cache) = state.cache.lock() else {
        return;
    };
    cache.retain(|entry| entry.key != key);
    cache.push(CacheEntry {
        key,
        text: translated.to_string(),
        detected: detected.to_string(),
    });
    while cache.len() > CACHE_LIMIT {
        cache.remove(0);
    }
    let snapshot = cache.clone();
    drop(cache);
    write_json(&cache_path(app), &snapshot);
}

fn with_history<F>(app: &AppHandle, edit: F) -> Vec<HistoryEntry>
where
    F: FnOnce(&mut Vec<HistoryEntry>),
{
    let state = app.state::<AppState>();
    let locked = state.history.lock();
    let Ok(mut history) = locked else {
        return Vec::new();
    };

    edit(&mut history);

    let snapshot = history.clone();
    drop(history);
    write_json(&history_path(app), &snapshot);
    snapshot
}

fn history_push(app: &AppHandle, source: &str, translated: &str, from: &str, to: &str) {
    with_history(app, |history| {
        push_entry(history, source, translated, from, to);
    });
}

fn push_entry(
    history: &mut Vec<HistoryEntry>,
    source: &str,
    translated: &str,
    from: &str,
    to: &str,
) {
    match history
        .iter_mut()
        .find(|entry| entry.source == source && entry.to == to)
    {
        Some(existing) => {
            existing.translated = translated.to_string();
            existing.from = from.to_string();
            existing.at = now_secs();
        }
        None => history.insert(
            0,
            HistoryEntry {
                id: unique_id(),
                source: source.to_string(),
                translated: translated.to_string(),
                from: from.to_string(),
                to: to.to_string(),
                at: now_secs(),
                pinned: false,
            },
        ),
    }

    history.sort_by_key(|entry| std::cmp::Reverse(entry.at));
    let mut kept = 0usize;
    history.retain(|entry| {
        if entry.pinned {
            return true;
        }
        kept += 1;
        kept <= HISTORY_LIMIT
    });
}

#[tauri::command]
fn get_history(app: AppHandle) -> Vec<HistoryEntry> {
    let state = app.state::<AppState>();
    let locked = state.history.lock();
    locked.map(|history| history.clone()).unwrap_or_default()
}

#[tauri::command]
fn clear_history(app: AppHandle) -> Vec<HistoryEntry> {
    with_history(&app, |history| history.retain(|entry| entry.pinned))
}

#[tauri::command]
fn delete_history_entry(app: AppHandle, id: String) -> Vec<HistoryEntry> {
    with_history(&app, |history| history.retain(|entry| entry.id != id))
}

#[tauri::command]
fn toggle_history_pin(app: AppHandle, id: String) -> Vec<HistoryEntry> {
    with_history(&app, |history| {
        if let Some(entry) = history.iter_mut().find(|entry| entry.id == id) {
            entry.pinned = !entry.pinned;
        }
    })
}

const PT_WORDS: &[&str] = &[
    " o ",
    " a ",
    " os ",
    " as ",
    " um ",
    " uma ",
    " de ",
    " do ",
    " da ",
    " dos ",
    " das ",
    " em ",
    " no ",
    " na ",
    " que ",
    " e ",
    " ou ",
    " mas ",
    " com ",
    " sem ",
    " por ",
    " para ",
    " se ",
    " eu ",
    " voce ",
    " ele ",
    " ela ",
    " nos ",
    " eles ",
    " meu ",
    " minha ",
    " seu ",
    " sua ",
    " isso ",
    " isto ",
    " aqui ",
    " ali ",
    " nao ",
    " sim ",
    " muito ",
    " mais ",
    " menos ",
    " tudo ",
    " nada ",
    " bom ",
    " boa ",
    " dia ",
    " noite ",
    " obrigado ",
    " obrigada ",
    " ola ",
    " porque ",
    " quando ",
    " onde ",
    " como ",
    " quem ",
    " qual ",
    " ser ",
    " estar ",
    " ter ",
    " fazer ",
    " vou ",
    " vai ",
    " esta ",
    " sao ",
    " foi ",
    " tem ",
    " quero ",
    " preciso ",
    " gosto ",
    " casa ",
    " agua ",
    " hoje ",
    " amanha ",
];

const EN_WORDS: &[&str] = &[
    " the ",
    " an ",
    " of ",
    " to ",
    " in ",
    " on ",
    " at ",
    " is ",
    " are ",
    " was ",
    " were ",
    " be ",
    " been ",
    " and ",
    " or ",
    " but ",
    " with ",
    " without ",
    " for ",
    " if ",
    " i ",
    " you ",
    " he ",
    " she ",
    " we ",
    " they ",
    " it ",
    " my ",
    " your ",
    " his ",
    " her ",
    " this ",
    " that ",
    " these ",
    " those ",
    " here ",
    " there ",
    " no ",
    " yes ",
    " not ",
    " very ",
    " more ",
    " less ",
    " all ",
    " nothing ",
    " good ",
    " day ",
    " night ",
    " thanks ",
    " thank ",
    " hello ",
    " hi ",
    " because ",
    " when ",
    " where ",
    " how ",
    " who ",
    " which ",
    " do ",
    " does ",
    " did ",
    " have ",
    " has ",
    " had ",
    " will ",
    " would ",
    " can ",
    " want ",
    " need ",
    " like ",
    " house ",
    " water ",
    " today ",
    " tomorrow ",
    " go ",
    " going ",
    " me ",
    " please ",
];

fn guess_target(text: &str) -> &'static str {
    let lowered = text.to_lowercase();

    if lowered.chars().any(|c| "ãõáàâêôçéíóúü".contains(c)) {
        return "EN";
    }

    let letters: String = lowered
        .chars()
        .map(|c| {
            if c.is_ascii_alphabetic() || c == '\'' {
                c
            } else {
                ' '
            }
        })
        .collect();
    let padded = format!(
        " {} ",
        letters.split_whitespace().collect::<Vec<_>>().join(" ")
    );

    let mut pt = PT_WORDS.iter().filter(|w| padded.contains(**w)).count() as i32;
    let mut en = EN_WORDS.iter().filter(|w| padded.contains(**w)).count() as i32;

    for pattern in [
        "ção", "ções", "lh", "nh", "mente ", "ando ", "endo ", "inho ", "inha ",
    ] {
        pt += lowered.matches(pattern).count() as i32;
    }
    for pattern in ["th", "wh", "ght", "ing ", "tion ", "ly "] {
        en += lowered.matches(pattern).count() as i32;
    }

    if pt != en {
        return if pt > en { "EN" } else { "PT" };
    }
    if lowered.contains(['w', 'y', 'k']) {
        "PT"
    } else {
        "EN"
    }
}

fn normalize_target(target: &str) -> String {
    if target.to_uppercase().starts_with("PT") {
        "PT".to_string()
    } else {
        "EN".to_string()
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TranslationResult {
    text: String,
    detected_source: String,
    target: String,
    cached: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Usage {
    character_count: u64,
    character_limit: u64,
}

fn deepl_endpoint(key: &str, path: &str) -> String {
    let host = if key.ends_with(":fx") {
        "https://api-free.deepl.com"
    } else {
        "https://api.deepl.com"
    };
    format!("{host}/v2/{path}")
}

fn check_status(code: u16) -> Result<(), String> {
    match code {
        403 => Err("deepl_auth: DeepL rejected the key. Check it in Settings.".to_string()),
        456 => {
            Err("limit: DeepL monthly character quota reached. It resets next month.".to_string())
        }
        code if !(200..300).contains(&code) => Err(format!("http_error: DeepL http {code}")),
        _ => Ok(()),
    }
}

async fn deepl_translate(
    client: &reqwest::Client,
    text: &str,
    target: &str,
    key: &str,
) -> Result<(String, String), String> {
    let target_lang = if target == "PT" { "PT-BR" } else { "EN-US" };
    let body = serde_json::json!({ "text": [text], "target_lang": target_lang });

    let response = client
        .post(deepl_endpoint(key, "translate"))
        .timeout(Duration::from_secs(10))
        .header("Authorization", format!("DeepL-Auth-Key {key}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("network_error: {e}"))?;

    check_status(response.status().as_u16())?;

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("decode_error: {e}"))?;

    let translated = data["translations"][0]["text"].as_str().unwrap_or_default();
    let detected = data["translations"][0]["detected_source_language"]
        .as_str()
        .unwrap_or_default();

    if translated.is_empty() {
        return Err("api_error: empty response from DeepL".to_string());
    }
    Ok((translated.to_string(), detected.to_uppercase()))
}

async fn translate_text(
    app: &AppHandle,
    text: String,
    target: Option<String>,
) -> Result<TranslationResult, String> {
    let key = load_settings(app).deepl_key.trim().to_string();
    if key.is_empty() {
        return Err("no_key: Add your DeepL API key in Settings to translate.".to_string());
    }

    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("empty: Nothing to translate.".to_string());
    }

    let auto = target.is_none();
    let mut target = target
        .map(|value| normalize_target(&value))
        .unwrap_or_else(|| guess_target(&text).to_string());

    if let Some(hit) = cache_lookup(app, &text, &target) {
        return Ok(TranslationResult {
            text: hit.text,
            detected_source: hit.detected,
            target,
            cached: true,
        });
    }

    let client = reqwest::Client::new();
    let (mut translated, mut detected) = deepl_translate(&client, &text, &target, &key).await?;

    if auto && detected.starts_with(&target) {
        target = if target == "EN" { "PT" } else { "EN" }.to_string();
        if let Some(hit) = cache_lookup(app, &text, &target) {
            return Ok(TranslationResult {
                text: hit.text,
                detected_source: hit.detected,
                target,
                cached: true,
            });
        }
        let retry = deepl_translate(&client, &text, &target, &key).await?;
        translated = retry.0;
        detected = retry.1;
    }

    cache_store(app, &text, &target, &translated, &detected);
    history_push(app, &text, &translated, &detected, &target);

    Ok(TranslationResult {
        text: translated,
        detected_source: detected,
        target,
        cached: false,
    })
}

#[tauri::command]
async fn translate(
    app: AppHandle,
    text: String,
    target: Option<String>,
) -> Result<TranslationResult, String> {
    translate_text(&app, text, target).await
}

#[tauri::command]
async fn get_usage(app: AppHandle) -> Result<Usage, String> {
    let key = load_settings(&app).deepl_key.trim().to_string();
    if key.is_empty() {
        return Err("no_key: Add your DeepL API key in Settings.".to_string());
    }

    let response = reqwest::Client::new()
        .get(deepl_endpoint(&key, "usage"))
        .timeout(Duration::from_secs(8))
        .header("Authorization", format!("DeepL-Auth-Key {key}"))
        .send()
        .await
        .map_err(|e| format!("network_error: {e}"))?;

    check_status(response.status().as_u16())?;

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("decode_error: {e}"))?;

    Ok(Usage {
        character_count: data["character_count"].as_u64().unwrap_or(0),
        character_limit: data["character_limit"].as_u64().unwrap_or(0),
    })
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MiniPayload {
    text: String,
    origin: String,
    error: Option<String>,
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

fn show_mini(app: &AppHandle, anchor: Option<(i32, i32)>, payload: MiniPayload) {
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

fn show_popup(app: &AppHandle) {
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

fn toggle_popup(app: &AppHandle) {
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
fn hide_popup(app: AppHandle) {
    if let Some(win) = app.get_webview_window("popup") {
        let _ = win.hide();
    }
}

#[tauri::command]
fn hide_mini(app: AppHandle) {
    if let Some(win) = app.get_webview_window("mini") {
        let _ = win.hide();
    }
}

#[tauri::command]
fn resize_window(app: AppHandle, height: f64) {
    if let Some(win) = app.get_webview_window("popup") {
        let _ = win.set_size(tauri::LogicalSize::new(520.0_f64, height.max(120.0)));
    }
}

#[tauri::command]
fn resize_mini(app: AppHandle, height: f64) {
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
fn set_clipboard(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}

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

fn cursor_position() -> Option<(i32, i32)> {
    use enigo::Mouse;
    new_enigo()?.location().ok()
}

fn translate_selection(app: &AppHandle) {
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

fn replace_selection(app: &AppHandle) {
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

#[tauri::command]
fn ocr_available() -> bool {
    ocr::available()
}

#[tauri::command]
fn cancel_region_capture(app: AppHandle) {
    if let Some(win) = app.get_webview_window("overlay") {
        let _ = win.hide();
    }
}

#[tauri::command]
fn start_region_capture(app: AppHandle) {
    if !ocr::available() {
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
fn ocr_region(app: AppHandle, x: f64, y: f64, width: f64, height: f64) {
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
        match ocr::recognize(px, py, pw, ph) {
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

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct UpdateInfo {
    version: String,
    notes: Option<String>,
}

async fn check_remote_update(
    app: &AppHandle,
) -> Result<Option<tauri_plugin_updater::Update>, String> {
    use tauri_plugin_updater::UpdaterExt;
    app.updater()
        .map_err(|e| format!("updater_error: {e}"))?
        .check()
        .await
        .map_err(|e| format!("updater_error: {e}"))
}

async fn refresh_pending_update(app: &AppHandle) -> Result<Option<UpdateInfo>, String> {
    let update = check_remote_update(app).await?;
    let info = update.as_ref().map(|update| UpdateInfo {
        version: update.version.clone(),
        notes: update.body.clone(),
    });
    let state = app.state::<AppState>();
    if let Ok(mut pending) = state.pending_update.lock() {
        *pending = update;
    }
    Ok(info)
}

#[tauri::command]
async fn check_for_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    refresh_pending_update(&app).await
}

#[tauri::command]
fn pending_update(app: AppHandle) -> Option<UpdateInfo> {
    let state = app.state::<AppState>();
    let pending = state.pending_update.lock().ok()?;
    pending.as_ref().map(|update| UpdateInfo {
        version: update.version.clone(),
        notes: update.body.clone(),
    })
}

#[tauri::command]
fn app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
async fn install_update(app: AppHandle) -> Result<(), String> {
    let taken = {
        let state = app.state::<AppState>();
        state
            .pending_update
            .lock()
            .ok()
            .and_then(|mut pending| pending.take())
    };
    let Some(update) = taken else {
        return Err("updater_none: already on the latest version.".to_string());
    };
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| format!("updater_error: {e}"))?;
    app.restart();
}

fn check_for_update_on_startup(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Ok(Some(info)) = refresh_pending_update(&app).await {
            let _ = app.emit_to("popup", "update-available", info);
        }
    });
}

#[tauri::command]
fn get_settings(app: AppHandle) -> Settings {
    load_settings(&app)
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
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

fn seed_or_refresh_autostart(app: &AppHandle) {
    let marker = config_dir(app).join(".autostart_seeded");
    if !marker.exists() {
        let _ = app.autolaunch().enable();
        let _ = std::fs::write(&marker, "1");
        return;
    }

    let manager = app.autolaunch();
    if manager.is_enabled().unwrap_or(false) {
        let _ = manager.enable();
    }
}

fn register_global_shortcuts(app: &AppHandle, settings: &Settings) -> Result<(), String> {
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
    if ocr::available() {
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
fn update_shortcuts(app: AppHandle) -> Result<(), String> {
    let settings = load_settings(&app);
    register_global_shortcuts(&app, &settings)
}

#[tauri::command]
fn get_shortcuts_paused(app: AppHandle) -> bool {
    load_settings(&app).shortcuts_paused
}

#[tauri::command]
fn set_shortcuts_paused(app: AppHandle, paused: bool) -> Result<(), String> {
    let mut settings = load_settings(&app);
    settings.shortcuts_paused = paused;
    persist_settings(&app, &settings)?;
    refresh_tray(&app, paused);
    register_global_shortcuts(&app, &settings)
}

fn build_tray_menu(app: &AppHandle, paused: bool) -> tauri::Result<Menu<Wry>> {
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

fn refresh_tray(app: &AppHandle, paused: bool) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(menu) = build_tray_menu(app, paused) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn open_view(app: &AppHandle, view: &str) {
    show_popup(app);
    if let Some(win) = app.get_webview_window("popup") {
        let _ = win.emit("open-view", view.to_string());
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            get_settings,
            save_settings,
            update_shortcuts,
            get_shortcuts_paused,
            set_shortcuts_paused,
            translate,
            get_usage,
            get_history,
            clear_history,
            delete_history_entry,
            toggle_history_pin,
            set_clipboard,
            hide_popup,
            hide_mini,
            resize_window,
            resize_mini,
            set_autostart,
            get_autostart,
            ocr_available,
            ocr_region,
            start_region_capture,
            cancel_region_capture,
            check_for_update,
            pending_update,
            app_version,
            install_update
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

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, source: &str, to: &str, at: u64, pinned: bool) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            source: source.to_string(),
            translated: format!("{source} translated"),
            from: if to == "EN" { "PT" } else { "EN" }.to_string(),
            to: to.to_string(),
            at,
            pinned,
        }
    }

    mod guess_target {
        use super::*;

        #[test]
        fn portuguese_diacritics_are_decisive_on_their_own() {
            assert_eq!(guess_target("ação"), "EN");
            assert_eq!(guess_target("você"), "EN");
            assert_eq!(guess_target("não"), "EN");
        }

        #[test]
        fn reads_plain_portuguese_without_accents() {
            assert_eq!(guess_target("preciso revisar isso antes do deploy"), "EN");
            assert_eq!(guess_target("bom dia para voce"), "EN");
        }

        #[test]
        fn reads_plain_english() {
            assert_eq!(
                guess_target("I need to review this before the deploy"),
                "PT"
            );
            assert_eq!(guess_target("the build failed on the runner"), "PT");
        }

        #[test]
        fn never_panics_on_degenerate_input() {
            for text in ["", " ", "123", "!@#$", "\n\t"] {
                let _ = guess_target(text);
            }
        }
    }

    mod normalize_target {
        use super::*;

        #[test]
        fn accepts_either_case_and_regional_variants() {
            assert_eq!(normalize_target("PT"), "PT");
            assert_eq!(normalize_target("pt"), "PT");
            assert_eq!(normalize_target("pt-BR"), "PT");
            assert_eq!(normalize_target("EN"), "EN");
            assert_eq!(normalize_target("en-US"), "EN");
        }

        #[test]
        fn anything_unrecognised_falls_back_to_english() {
            assert_eq!(normalize_target("klingon"), "EN");
            assert_eq!(normalize_target(""), "EN");
        }
    }

    mod deepl_endpoint {
        use super::*;

        #[test]
        fn free_tier_keys_route_to_the_free_host() {
            assert_eq!(
                deepl_endpoint("abc-123:fx", "translate"),
                "https://api-free.deepl.com/v2/translate"
            );
        }

        #[test]
        fn paid_keys_route_to_the_paid_host() {
            assert_eq!(
                deepl_endpoint("abc-123", "translate"),
                "https://api.deepl.com/v2/translate"
            );
        }

        #[test]
        fn the_same_split_applies_to_every_path() {
            assert_eq!(
                deepl_endpoint("abc:fx", "usage"),
                "https://api-free.deepl.com/v2/usage"
            );
        }
    }

    mod check_status {
        use super::*;

        #[test]
        fn success_codes_pass() {
            assert!(check_status(200).is_ok());
            assert!(check_status(299).is_ok());
        }

        #[test]
        fn auth_and_quota_get_prefixes_the_ui_keys_off() {
            assert!(check_status(403).unwrap_err().starts_with("deepl_auth:"));
            assert!(check_status(456).unwrap_err().starts_with("limit:"));
        }

        #[test]
        fn other_failures_fall_back_to_a_generic_http_error() {
            assert!(check_status(500).unwrap_err().starts_with("http_error:"));
            assert!(check_status(404).unwrap_err().starts_with("http_error:"));
        }
    }

    mod cache_key {
        use super::*;

        #[test]
        fn the_target_is_part_of_the_key() {
            assert_ne!(cache_key("hello", "EN"), cache_key("hello", "PT"));
        }

        #[test]
        fn the_separator_is_a_control_char_so_keys_cannot_collide() {
            let key = cache_key("hello", "EN");
            assert!(key.contains('\u{1}'));
            assert!(key.starts_with("EN"));
            assert!(key.ends_with("hello"));
        }
    }

    mod push_entry {
        use super::*;

        #[test]
        fn adds_a_new_entry_at_the_front() {
            let mut history = Vec::new();
            push_entry(&mut history, "hello", "ola", "EN", "PT");

            assert_eq!(history.len(), 1);
            assert_eq!(history[0].source, "hello");
            assert_eq!(history[0].translated, "ola");
            assert_eq!(history[0].to, "PT");
            assert!(!history[0].pinned);
        }

        #[test]
        fn retranslating_the_same_text_updates_instead_of_duplicating() {
            let mut history = Vec::new();
            push_entry(&mut history, "hello", "ola", "EN", "PT");
            push_entry(&mut history, "hello", "olá", "EN", "PT");

            assert_eq!(history.len(), 1);
            assert_eq!(history[0].translated, "olá");
        }

        #[test]
        fn the_same_text_in_the_other_direction_is_a_separate_entry() {
            let mut history = Vec::new();
            push_entry(&mut history, "hello", "ola", "EN", "PT");
            push_entry(&mut history, "hello", "hello", "EN", "EN");

            assert_eq!(history.len(), 2);
        }

        #[test]
        fn unpinned_entries_are_evicted_past_the_limit() {
            let mut history: Vec<HistoryEntry> = (0..HISTORY_LIMIT)
                .map(|i| {
                    entry(
                        &i.to_string(),
                        &format!("source {i}"),
                        "EN",
                        i as u64,
                        false,
                    )
                })
                .collect();

            push_entry(&mut history, "the newest one", "translated", "PT", "EN");

            assert_eq!(history.len(), HISTORY_LIMIT);
            assert!(history.iter().any(|e| e.source == "the newest one"));
            assert!(!history.iter().any(|e| e.source == "source 0"));
        }

        #[test]
        fn pinned_entries_survive_eviction_even_when_oldest() {
            let mut history = vec![entry("pinned", "keep me", "EN", 0, true)];
            history.extend((0..HISTORY_LIMIT).map(|i| {
                entry(
                    &i.to_string(),
                    &format!("source {i}"),
                    "EN",
                    (i + 1) as u64,
                    false,
                )
            }));

            push_entry(&mut history, "the newest one", "translated", "PT", "EN");

            assert!(
                history.iter().any(|e| e.source == "keep me"),
                "a pinned entry must never be evicted"
            );
            assert_eq!(history.iter().filter(|e| !e.pinned).count(), HISTORY_LIMIT);
        }

        #[test]
        fn entries_end_up_newest_first() {
            let mut history = vec![
                entry("a", "oldest", "EN", 10, false),
                entry("b", "middle", "EN", 20, false),
            ];
            push_entry(&mut history, "newest", "translated", "PT", "EN");

            let timestamps: Vec<u64> = history.iter().map(|e| e.at).collect();
            let mut sorted = timestamps.clone();
            sorted.sort_by(|a, b| b.cmp(a));
            assert_eq!(timestamps, sorted);
        }
    }

    mod unique_id {
        use super::*;

        #[test]
        fn is_a_string_so_the_webview_cannot_lose_precision() {
            let id = unique_id();
            assert!(id.parse::<u128>().unwrap() > (1u128 << 53));
        }

        #[test]
        fn successive_ids_differ() {
            assert_ne!(unique_id(), unique_id());
        }
    }

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
