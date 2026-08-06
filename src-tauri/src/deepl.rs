use serde::Serialize;
use std::time::Duration;
use tauri::AppHandle;

use crate::cache::{cache_lookup, cache_store};
use crate::history::history_push;
use crate::language::{guess_target, normalize_target};
use crate::settings::load_settings;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TranslationResult {
    pub(crate) text: String,
    pub(crate) detected_source: String,
    pub(crate) target: String,
    pub(crate) cached: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Usage {
    pub(crate) character_count: u64,
    pub(crate) character_limit: u64,
}

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

pub(crate) async fn translate_text(
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
pub async fn translate(
    app: AppHandle,
    text: String,
    target: Option<String>,
) -> Result<TranslationResult, String> {
    translate_text(&app, text, target).await
}

async fn fetch_usage(key: &str) -> Result<Usage, String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("no_key: Add your DeepL API key in Settings.".to_string());
    }

    let response = reqwest::Client::new()
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
    fetch_usage(&load_settings(&app).deepl_key).await
}

#[tauri::command]
pub async fn verify_deepl_key(key: String) -> Result<Usage, String> {
    fetch_usage(&key).await
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
