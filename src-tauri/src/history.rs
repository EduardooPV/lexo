use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

use crate::paths::{history_path, write_json};
use crate::state::AppState;

const HISTORY_LIMIT: usize = 200;

pub(crate) fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub(crate) fn unique_id() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HistoryEntry {
    pub(crate) id: String,
    pub(crate) source: String,
    pub(crate) translated: String,
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) at: u64,
    #[serde(default)]
    pub(crate) pinned: bool,
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

pub(crate) fn history_push(app: &AppHandle, source: &str, translated: &str, from: &str, to: &str) {
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
pub fn get_history(app: AppHandle) -> Vec<HistoryEntry> {
    let state = app.state::<AppState>();
    let locked = state.history.lock();
    locked.map(|history| history.clone()).unwrap_or_default()
}

#[tauri::command]
pub fn clear_history(app: AppHandle) -> Vec<HistoryEntry> {
    with_history(&app, |history| history.retain(|entry| entry.pinned))
}

#[tauri::command]
pub fn delete_history_entry(app: AppHandle, id: String) -> Vec<HistoryEntry> {
    with_history(&app, |history| history.retain(|entry| entry.id != id))
}

#[tauri::command]
pub fn toggle_history_pin(app: AppHandle, id: String) -> Vec<HistoryEntry> {
    with_history(&app, |history| {
        if let Some(entry) = history.iter_mut().find(|entry| entry.id == id) {
            entry.pinned = !entry.pinned;
        }
    })
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
}
