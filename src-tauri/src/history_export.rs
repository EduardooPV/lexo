use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::{
    history::{get_history, HistoryEntry},
    paths::atomic_write,
};

enum Format {
    Json,
    Csv,
}

impl Format {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            _ => Err("export_error: Choose JSON or CSV.".into()),
        }
    }

    fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Csv => "csv",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::Csv => "CSV",
        }
    }

    fn encode(&self, entries: &[HistoryEntry]) -> Result<Vec<u8>, String> {
        match self {
            Self::Json => serde_json::to_vec_pretty(entries).map_err(|error| error.to_string()),
            Self::Csv => Ok(csv(entries).into_bytes()),
        }
    }
}

pub(crate) async fn export_history(
    app: AppHandle,
    format: String,
) -> Result<Option<String>, String> {
    let format = Format::parse(&format)?;
    tauri::async_runtime::spawn_blocking(move || {
        let popup = app
            .get_webview_window("popup")
            .ok_or("The Lexo popup is unavailable.")?;
        let ext = format.extension();
        let selected = app
            .dialog()
            .file()
            .set_parent(&popup)
            .set_file_name(format!("lexo-history.{ext}"))
            .add_filter(format.name(), &[ext])
            .blocking_save_file();
        let Some(selected) = selected else {
            return Ok(None);
        };
        let path = selected.into_path().map_err(|error| error.to_string())?;
        let mut entries = get_history(app.clone());
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.at));
        atomic_write(&path, &format.encode(&entries)?).map_err(|error| error.to_string())?;
        Ok::<_, String>(Some(path.to_string_lossy().into_owned()))
    })
    .await
    .map_err(|error| format!("export_error: {error}"))?
    .map_err(|error| format!("export_error: {error}"))
}

fn csv(entries: &[HistoryEntry]) -> String {
    let mut output = String::from("\u{feff}at,at_iso,from,to,pinned,source,translated\r\n");
    for entry in entries {
        let epoch = entry.at.to_string();
        let date = iso_utc(entry.at);
        let pinned = entry.pinned.to_string();
        let row = [
            epoch.as_str(),
            &date,
            &entry.from,
            &entry.to,
            &pinned,
            &entry.source,
            &entry.translated,
        ];
        for (index, field) in row.into_iter().enumerate() {
            if index != 0 {
                output.push(',');
            }
            output.push_str(&csv_field(field));
        }
        output.push_str("\r\n");
    }
    output
}

fn csv_field(value: &str) -> String {
    let prefix = if value.trim_start().starts_with(['=', '+', '-', '@']) {
        "'"
    } else {
        ""
    };
    format!("\"{prefix}{}\"", value.replace('"', "\"\""))
}

fn iso_utc(at: u64) -> String {
    // Gregorian civil date from days since the Unix epoch. Start the year in
    // March so leap days fall at the end of each adjusted year.
    let days = (at / 86_400) as i64 + 719_468;
    let era = days / 146_097;
    let day_of_era = days % 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    let hour = at % 86_400 / 3600;
    let minute = at % 3600 / 60;
    let second = at % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_csv_with_bom_quoted_fields_and_formula_protection() {
        let entry = HistoryEntry {
            id: "test".into(),
            at: 0,
            from: "PT".into(),
            to: "EN-US".into(),
            pinned: true,
            source: "Olá, \"mundo\"!\nOutra linha".into(),
            translated: "  +hello".into(),
        };
        let output = csv(&[entry]);
        assert!(output.as_bytes().starts_with(&[0xef, 0xbb, 0xbf]));
        assert!(output.contains("at,at_iso,from,to,pinned,source,translated\r\n"));
        assert!(output.contains("\"Olá, \"\"mundo\"\"!\nOutra linha\",\"'  +hello\"\r\n"));
        for prefix in ['=', '+', '-', '@'] {
            assert_eq!(
                csv_field(&format!("{prefix}formula")),
                format!("\"'{prefix}formula\"")
            );
        }
    }

    #[test]
    fn converts_epoch_leap_day_and_year_boundary_to_utc() {
        assert_eq!(iso_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(iso_utc(946_684_799), "1999-12-31T23:59:59Z");
        assert_eq!(iso_utc(946_684_800), "2000-01-01T00:00:00Z");
    }
}
