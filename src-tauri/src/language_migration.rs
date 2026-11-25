use std::{fs, io, path::Path};

use crate::{history::HistoryEntry, paths::write_json};

fn migrate_history_targets(history: &mut [HistoryEntry]) -> bool {
    let mut changed = false;
    for entry in history {
        let target = match entry.to.as_str() {
            "PT" => "PT-BR",
            "EN" => "EN-US",
            _ => continue,
        };
        entry.to = target.into();
        changed = true;
    }
    changed
}

pub(crate) fn migrate_history_and_cache(dir: &Path) -> io::Result<()> {
    let history_path = dir.join("history.json");
    match fs::read(&history_path) {
        Ok(raw) => {
            let mut history: Vec<HistoryEntry> = serde_json::from_slice(&raw)?;
            if migrate_history_targets(&mut history) {
                write_json(&history_path, &history)?;
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    match fs::remove_file(dir.join("cache.json")) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrades_legacy_targets_and_discards_the_old_cache_idempotently() {
        let dir = std::env::temp_dir().join(format!("lexo-lang-{}", crate::history::unique_id()));
        fs::create_dir(&dir).unwrap();
        let history_path = dir.join("history.json");
        let cache_path = dir.join("cache.json");
        let mut entries: Vec<HistoryEntry> = ["PT", "EN", "DE"]
            .into_iter()
            .map(|to| HistoryEntry {
                id: to.into(),
                at: 20,
                from: "EN".into(),
                to: to.into(),
                pinned: true,
                source: "source".into(),
                translated: "translation".into(),
            })
            .collect();
        write_json(&history_path, &entries).unwrap();
        fs::write(&cache_path, "[]").unwrap();

        migrate_history_and_cache(&dir).unwrap();
        assert!(!cache_path.exists());
        let raw = fs::read(&history_path).unwrap();
        entries = serde_json::from_slice(&raw).unwrap();
        assert_eq!(
            entries.iter().map(|e| e.to.as_str()).collect::<Vec<_>>(),
            ["PT-BR", "EN-US", "DE"]
        );
        assert!(entries.iter().all(|e| e.from == "EN" && e.pinned));

        migrate_history_and_cache(&dir).unwrap();
        assert_eq!(raw, fs::read(&history_path).unwrap());
        fs::remove_file(history_path).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
