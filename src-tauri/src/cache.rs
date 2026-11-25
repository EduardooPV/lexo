use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tauri::{AppHandle, Manager};

use crate::persist::{self, Kind};
use crate::state::AppState;

const CACHE_LIMIT: usize = 400;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct CacheEntry {
    pub(crate) key: String,
    pub(crate) text: String,
    pub(crate) detected: String,
}

#[derive(Default)]
pub(crate) struct Cache {
    map: HashMap<String, CacheEntry>,
    order: VecDeque<String>,
}

impl From<Vec<CacheEntry>> for Cache {
    fn from(entries: Vec<CacheEntry>) -> Self {
        let mut cache = Self::default();
        for entry in entries {
            cache.insert(entry);
        }
        cache
    }
}

impl Cache {
    fn touch(&mut self, key: &str) {
        if let Some(index) = self.order.iter().position(|item| item == key) {
            self.order.remove(index);
        }
        self.order.push_back(key.to_owned());
    }

    fn lookup(&mut self, key: &str) -> Option<CacheEntry> {
        let hit = self.map.get(key)?.clone();
        self.touch(key);
        Some(hit)
    }

    fn insert(&mut self, entry: CacheEntry) {
        self.touch(&entry.key);
        self.map.insert(entry.key.clone(), entry);
        if self.order.len() > CACHE_LIMIT {
            if let Some(oldest) = self.order.pop_front() {
                self.map.remove(&oldest);
            }
        }
    }

    pub(crate) fn snapshot(&self) -> Vec<CacheEntry> {
        self.order
            .iter()
            .filter_map(|key| self.map.get(key))
            .cloned()
            .collect()
    }
}

fn cache_key(text: &str, target: &str) -> String {
    let normalized = text.replace("\r\n", "\n");
    let mut key = format!("v2\u{1}{target}\u{1}");
    let mut in_space = false;
    for ch in normalized.trim().chars() {
        let space = matches!(ch, ' ' | '\t');
        if !space || !in_space {
            key.push(if space { ' ' } else { ch });
        }
        in_space = space;
    }
    key
}

pub(crate) fn cache_lookup(app: &AppHandle, text: &str, target: &str) -> Option<CacheEntry> {
    let state = app.state::<AppState>();
    let mut cache = state.cache.lock().ok()?;
    let hit = cache.lookup(&cache_key(text, target))?;
    persist::schedule(app, Kind::Cache);
    Some(hit)
}

pub(crate) fn cache_store(
    app: &AppHandle,
    text: &str,
    target: &str,
    translated: &str,
    detected: &str,
) {
    let state = app.state::<AppState>();
    let Ok(mut cache) = state.cache.lock() else {
        return;
    };
    cache.insert(CacheEntry {
        key: cache_key(text, target),
        text: translated.to_string(),
        detected: detected.to_string(),
    });
    persist::schedule(app, Kind::Cache);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(index: usize) -> CacheEntry {
        CacheEntry {
            key: index.to_string(),
            text: format!("Translation {index}"),
            detected: "PT".into(),
        }
    }

    #[test]
    fn normalises_horizontal_whitespace_without_joining_paragraphs() {
        assert_eq!(cache_key("  a \t b\r\n", "EN"), cache_key("a b", "EN"));
        assert_eq!(cache_key("a\r\n\r\nb", "EN"), cache_key("a\n\nb", "EN"));
        assert_ne!(cache_key("a\n\nb", "EN"), cache_key("a b", "EN"));
        assert_ne!(cache_key("hello", "EN"), cache_key("hello", "PT"));
        assert_ne!(cache_key("hello", "EN"), "EN\u{1}hello");
    }

    #[test]
    fn cache_hits_protect_entries_from_eviction() {
        let mut cache = Cache::from((0..CACHE_LIMIT).map(entry).collect::<Vec<_>>());
        assert!(cache.lookup("0").is_some());
        cache.insert(entry(CACHE_LIMIT));
        assert_eq!(cache.snapshot().len(), CACHE_LIMIT);
        assert!(cache.lookup("1").is_none());
        assert!(cache.lookup("0").is_some());
    }

    #[test]
    fn snapshots_preserve_the_lru_order_when_reloaded() {
        let mut cache = Cache::from(vec![entry(1), entry(2), entry(3)]);
        cache.lookup("1").unwrap();
        let restored = Cache::from(cache.snapshot());
        let keys: Vec<_> = restored
            .snapshot()
            .into_iter()
            .map(|item| item.key)
            .collect();
        assert_eq!(keys, ["2", "3", "1"]);
    }
}
