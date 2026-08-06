use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::paths::{cache_path, write_json};
use crate::state::AppState;

const CACHE_LIMIT: usize = 400;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct CacheEntry {
    pub(crate) key: String,
    pub(crate) text: String,
    pub(crate) detected: String,
}

fn cache_key(text: &str, target: &str) -> String {
    format!("{target}\u{1}{text}")
}

pub(crate) fn cache_lookup(app: &AppHandle, text: &str, target: &str) -> Option<CacheEntry> {
    let key = cache_key(text, target);
    let state = app.state::<AppState>();
    let cache = state.cache.lock().ok()?;
    cache.iter().find(|entry| entry.key == key).cloned()
}

pub(crate) fn cache_store(
    app: &AppHandle,
    text: &str,
    target: &str,
    translated: &str,
    detected: &str,
) {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
