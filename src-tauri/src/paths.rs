use serde::Serialize;
use std::{
    ffi::OsString,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use tauri::{AppHandle, Manager};

pub(crate) fn config_dir(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_config_dir().unwrap_or_else(|error| {
        eprintln!("Could not locate the Lexo configuration directory: {error}");
        std::env::temp_dir().join("lexo")
    });
    if let Err(error) = fs::create_dir_all(&dir) {
        eprintln!("Could not create the Lexo configuration directory: {error}");
    }
    dir
}

pub(crate) fn read_json<T: serde::de::DeserializeOwned + Default>(path: &Path) -> T {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn open_temp(path: &Path) -> io::Result<(PathBuf, File)> {
    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing file name"))?;
    let mut temp_name = OsString::from(name);
    temp_name.push(".tmp");
    let temp = path.with_file_name(temp_name);

    match File::create_new(&temp) {
        Ok(file) => Ok((temp, file)),
        // A prior interruption can leave a temp file behind. Never overwrite it.
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => loop {
            let mut unique = OsString::from(name);
            let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
            unique.push(format!(".{}.{id}.tmp", std::process::id()));
            let temp = path.with_file_name(unique);
            match File::create_new(&temp) {
                Ok(file) => break Ok((temp, file)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => break Err(error),
            }
        },
        Err(error) => Err(error),
    }
}

pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let (temp, mut file) = open_temp(path)?;
    let result = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    let result = result.and_then(|()| fs::rename(&temp, path));
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

pub(crate) fn write_json<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let result = serde_json::to_vec_pretty(value)
        .map_err(io::Error::from)
        .and_then(|json| atomic_write(path, &json));
    if let Err(error) = &result {
        eprintln!("Could not write {}: {error}", path.display());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_creates_and_replaces_without_leaving_a_temp_file() {
        let dir = std::env::temp_dir().join(format!("lexo-atomic-{}", crate::history::unique_id()));
        fs::create_dir(&dir).unwrap();
        let path = dir.join("settings.json");

        atomic_write(&path, b"old").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"old");
        atomic_write(&path, b"replacement").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"replacement");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);

        let stale = dir.join("settings.json.tmp");
        fs::write(&stale, b"partial").unwrap();
        atomic_write(&path, b"complete").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"complete");
        assert_eq!(fs::read(&stale).unwrap(), b"partial");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);

        assert!(atomic_write(&dir.join("missing/settings.json"), b"none").is_err());
        fs::remove_file(path).unwrap();
        fs::remove_file(stale).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
