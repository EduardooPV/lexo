use serde::Serialize;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

use crate::cache::{cache_lookup, cache_store};
use crate::history::history_push;
use crate::language::guess_target;
use crate::languages::{base, normalize_target};
use crate::quota::handle_usage;
use crate::settings::load_settings;
use crate::state::AppState;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TranslationResult {
    pub(crate) text: String,
    pub(crate) detected_source: String,
    pub(crate) target: String,
    pub(crate) cached: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Usage {
    pub(crate) character_count: u64,
    pub(crate) character_limit: u64,
}

const LONG_TEXT_CHARS: usize = 5_000;
const MAX_REQUEST_BYTES: usize = 120_000;
const DEEPL_BODY_LIMIT: usize = 128 * 1024;
const USAGE_INTERVAL: Duration = Duration::from_secs(10 * 60);

const DEEPL_SIGNUP_URL: &str = "https://www.deepl.com/pro-api";

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
        413 => Err("too_large: This translation exceeds the DeepL request size.".into()),
        403 => Err("deepl_auth: DeepL rejected the key. Check it in Settings.".to_string()),
        456 => {
            Err("limit: DeepL monthly character quota reached. It resets next month.".to_string())
        }
        code if !(200..300).contains(&code) => Err(format!("http_error: DeepL http {code}")),
        _ => Ok(()),
    }
}

fn request_body(text: &str, target: &str, formality: &str) -> serde_json::Value {
    let mut body = serde_json::json!({ "text": [text], "target_lang": target });
    match formality {
        "more" => body["formality"] = "prefer_more".into(),
        "less" => body["formality"] = "prefer_less".into(),
        _ => {}
    }
    body
}

fn check_length(text: &str, confirmed: bool) -> Result<(), String> {
    if text.len() > MAX_REQUEST_BYTES {
        return Err(format!(
            "too_large: Maximum input is {MAX_REQUEST_BYTES} bytes."
        ));
    }
    let chars = text.chars().count();
    if chars > LONG_TEXT_CHARS && !confirmed {
        return Err(format!("too_long: {chars}"));
    }
    Ok(())
}

fn initial_target<'a>(text: &str, primary: &'a str, secondary: &'a str) -> &'a str {
    let pair = (base(primary), base(secondary));
    if matches!(pair, ("PT", "EN") | ("EN", "PT")) && guess_target(text) == pair.0 {
        primary
    } else {
        secondary
    }
}

fn next_target<'a>(
    primary: &'a str,
    secondary: &'a str,
    target: &str,
    detected: &str,
) -> Option<&'a str> {
    if base(detected) != base(target) {
        return None;
    }
    Some(if target == primary {
        secondary
    } else {
        primary
    })
}

async fn deepl_translate(
    client: &reqwest::Client,
    text: &str,
    target: &str,
    key: &str,
    formality: &str,
) -> Result<(String, String), String> {
    let body = serde_json::to_vec(&request_body(text, target, formality))
        .map_err(|error| format!("encode_error: {error}"))?;
    if body.len() > DEEPL_BODY_LIMIT {
        return Err("too_large: This translation exceeds the DeepL request size.".into());
    }

    let response = client
        .post(deepl_endpoint(key, "translate"))
        .timeout(Duration::from_secs(10))
        .header("Authorization", format!("DeepL-Auth-Key {key}"))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body)
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

async fn translate_target(
    app: &AppHandle,
    client: &reqwest::Client,
    text: &str,
    target: String,
    key: &str,
    formality: &str,
) -> Result<TranslationResult, String> {
    if let Some(hit) = cache_lookup(app, text, &target, formality) {
        return Ok(TranslationResult {
            text: hit.text,
            detected_source: hit.detected,
            target,
            cached: true,
        });
    }

    let (translated, detected_source) =
        deepl_translate(client, text, &target, key, formality).await?;
    cache_store(app, text, &target, formality, &translated, &detected_source);
    Ok(TranslationResult {
        text: translated,
        detected_source,
        target,
        cached: false,
    })
}

pub(crate) async fn translate_text(
    app: &AppHandle,
    text: String,
    target: Option<String>,
    confirmed: bool,
) -> Result<TranslationResult, String> {
    let settings = load_settings(app);
    let key = settings.deepl_key.trim();
    if key.is_empty() {
        return Err("no_key: Add your DeepL API key in Settings to translate.".to_string());
    }
    if text.trim().is_empty() {
        return Err("empty: Nothing to translate.".to_string());
    }
    check_length(&text, confirmed)?;

    let primary = &settings.primary_lang;
    let secondary = &settings.secondary_lang;
    let formality = &settings.formality;
    let auto = target.is_none();
    let target = match target {
        Some(value) => normalize_target(&value)?,
        None => initial_target(&text, primary, secondary).into(),
    };
    let state = app.state::<AppState>();
    let client = &state.http;
    let mut result = translate_target(app, client, &text, target, key, formality).await?;

    let initial_cached = result.cached;
    if auto {
        if let Some(retry) =
            next_target(primary, secondary, &result.target, &result.detected_source)
        {
            result = translate_target(app, client, &text, retry.into(), key, formality).await?;
        }
    }

    if !result.cached {
        history_push(
            app,
            &text,
            &result.text,
            &result.detected_source,
            &result.target,
        );
    }
    result.cached &= initial_cached;
    if !result.cached {
        spawn_usage_check(app, key.to_owned());
    }

    Ok(result)
}

#[tauri::command]
pub async fn translate(
    app: AppHandle,
    text: String,
    target: Option<String>,
    confirmed: Option<bool>,
) -> Result<TranslationResult, String> {
    translate_text(&app, text, target, confirmed.unwrap_or(false)).await
}

async fn fetch_usage(client: &reqwest::Client, key: &str) -> Result<Usage, String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("no_key: Add your DeepL API key in Settings.".to_string());
    }

    let response = client
        .get(deepl_endpoint(key, "usage"))
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

#[tauri::command]
pub async fn get_usage(app: AppHandle) -> Result<Usage, String> {
    let settings = load_settings(&app);
    fetch_current_usage(&app, &settings.deepl_key).await
}

#[tauri::command]
pub async fn verify_deepl_key(app: AppHandle, key: String) -> Result<Usage, String> {
    fetch_usage(&app.state::<AppState>().http, &key).await
}

fn should_warm_up(last: Option<Instant>, now: Instant) -> bool {
    last.is_none_or(|at| now.saturating_duration_since(at) > Duration::from_secs(60))
}

pub(crate) fn warm_up(app: &AppHandle) {
    let key = load_settings(app).deepl_key;
    if key.trim().is_empty() {
        return;
    }

    let state = app.state::<AppState>();
    let Ok(mut last) = state.last_warmup.lock() else {
        return;
    };
    let now = Instant::now();
    if !should_warm_up(*last, now) {
        return;
    }
    *last = Some(now);

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = fetch_current_usage(&app, &key).await;
    });
}

async fn fetch_current_usage(app: &AppHandle, key: &str) -> Result<Usage, String> {
    let started = Instant::now();
    let usage = fetch_usage(&app.state::<AppState>().http, key).await?;
    handle_usage(app, key.trim(), &usage, started);
    Ok(usage)
}

fn spawn_usage_check(app: &AppHandle, key: String) {
    let state = app.state::<AppState>();
    let Ok(mut last) = state.last_usage_check.lock() else {
        return;
    };
    let now = Instant::now();
    if last.is_some_and(|at| now.saturating_duration_since(at) < USAGE_INTERVAL) {
        return;
    }
    *last = Some(now);

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = fetch_current_usage(&app, &key).await;
    });
}

#[tauri::command]
pub fn open_deepl_signup(app: AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(DEEPL_SIGNUP_URL, None::<&str>)
        .map_err(|e| format!("open_error: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warmup_is_throttled_for_sixty_seconds() {
        let now = Instant::now();
        assert!(should_warm_up(None, now));
        assert!(!should_warm_up(Some(now), now + Duration::from_secs(10)));
        assert!(!should_warm_up(Some(now), now + Duration::from_secs(60)));
        assert!(should_warm_up(Some(now), now + Duration::from_secs(61)));
    }

    #[test]
    fn retries_the_other_language_only_when_the_detected_base_matches() {
        assert_eq!(next_target("PT-BR", "EN-US", "EN-US", "EN"), Some("PT-BR"));
        assert_eq!(next_target("ES", "DE", "DE", "DE"), Some("ES"));
        assert_eq!(next_target("ES", "DE", "DE", "ES"), None);
        assert_eq!(next_target("DE", "ES", "DE", "IT"), None);
        assert_eq!(initial_target("action", "PT-PT", "EN-GB"), "EN-GB");
        assert_eq!(
            initial_target("the build failed", "PT-PT", "EN-GB"),
            "PT-PT"
        );
        assert_eq!(initial_target("the build failed", "DE", "ES"), "ES");
    }

    #[test]
    fn sends_only_the_selected_target_and_supported_formality_parameters() {
        let auto = request_body("hello", "DE", "default");
        assert_eq!(
            auto,
            serde_json::json!({"text": ["hello"], "target_lang": "DE"})
        );
        assert_eq!(
            request_body("hello", "DE", "more")["formality"],
            "prefer_more"
        );
        assert_eq!(
            request_body("hello", "DE", "less")["formality"],
            "prefer_less"
        );
    }

    #[test]
    fn applies_the_character_warning_and_enforces_the_byte_limit() {
        assert!(check_length(&"é".repeat(5_000), false).is_ok());
        let long = "é".repeat(5_001);
        assert_eq!(check_length(&long, false).unwrap_err(), "too_long: 5001");
        assert!(check_length(&long, true).is_ok());
        assert!(check_length(&"a".repeat(MAX_REQUEST_BYTES), true).is_ok());
        let oversized = "🌍".repeat(MAX_REQUEST_BYTES / 4 + 1);
        assert!(check_length(&oversized, true)
            .unwrap_err()
            .starts_with("too_large:"));
        assert!(check_length(&oversized, false)
            .unwrap_err()
            .starts_with("too_large:"));
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
}
