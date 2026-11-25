use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::{
    deepl::Usage,
    settings::{load_settings, persist_settings},
    state::AppState,
};

const QUOTA_WARN_PERCENT: f64 = 80.0;

#[derive(Debug, PartialEq, Eq)]
enum Decision {
    Notify,
    Reset,
    Nothing,
}

fn percent(usage: &Usage) -> f64 {
    if usage.character_limit == 0 {
        return 0.0;
    }
    100.0 * usage.character_count as f64 / usage.character_limit as f64
}

fn decide(percent: f64, quota_warned: bool) -> Decision {
    match (percent >= QUOTA_WARN_PERCENT, quota_warned) {
        (true, false) => Decision::Notify,
        (false, true) => Decision::Reset,
        _ => Decision::Nothing,
    }
}

pub(crate) fn handle_usage(app: &AppHandle, key: &str, usage: &Usage, started: Instant) {
    let state = app.state::<AppState>();
    let Ok(mut last) = state.last_usage_applied.lock() else {
        return;
    };
    if last.is_some_and(|at| at > started) {
        return;
    }
    let current = load_settings(app);
    if current.deepl_key.trim() != key {
        return;
    }

    *last = Some(started);
    let _ = app.emit_to("popup", "usage-updated", usage);
    if usage.character_limit == 0
        || decide(percent(usage), current.quota_warned) == Decision::Nothing
    {
        return;
    }

    let mut action = Decision::Nothing;
    let saved = persist_settings(app, |settings| {
        if settings.deepl_key.trim() != key {
            return false;
        }
        action = decide(percent(usage), settings.quota_warned);
        settings.quota_warned = action == Decision::Notify;
        action != Decision::Nothing
    });
    if saved.is_ok() && action == Decision::Notify {
        if let Err(error) = app
            .notification()
            .builder()
            .title("Lexo")
            .body(format!(
                "You've used {:.0}% of your monthly DeepL quota ({} of {} characters).",
                QUOTA_WARN_PERCENT, usage.character_count, usage.character_limit
            ))
            .show()
        {
            eprintln!("Could not show the DeepL quota notification: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warns_once_then_rearms_below_the_threshold() {
        assert_eq!(decide(79.0, false), Decision::Nothing);
        assert_eq!(decide(80.0, false), Decision::Notify);
        assert_eq!(decide(85.0, true), Decision::Nothing);
        assert_eq!(decide(40.0, true), Decision::Reset);
    }

    #[test]
    fn calculates_usage_from_the_reported_quota() {
        assert_eq!(
            percent(&Usage {
                character_count: 8,
                character_limit: 10
            }),
            80.0
        );
        assert_eq!(
            percent(&Usage {
                character_count: 8,
                character_limit: 0
            }),
            0.0
        );
    }
}
