//! Player settings, kept in `%APPDATA%\<identifier>\settings.json` (see [`crate::paths`]).
use std::fs;
use std::path::{Path, PathBuf};

use crate::paths;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Write the event log to files (off unless the player turns it on).
    pub save_logs: bool,
    /// Ask GitHub for a newer release when the app starts.
    pub check_updates: bool,
    /// The first-run notice has been shown.
    pub intro_seen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            save_logs: false,
            check_updates: true,
            intro_seen: false,
        }
    }
}

fn file() -> Option<PathBuf> {
    Some(paths::config_dir()?.join("settings.json"))
}

/// Where versions before 0.1.0-alpha.2 kept the settings.
fn old_file() -> Option<PathBuf> {
    Some(
        PathBuf::from(std::env::var_os("APPDATA")?)
            .join(paths::OLD_NAME)
            .join("settings.json"),
    )
}

/// First start after the update: copy the settings an older version saved. The old file stays.
fn carry_over(old: &Path, new: &Path) {
    if new.exists() || !old.exists() {
        return;
    }
    if let Some(dir) = new.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::copy(old, new);
}

/// Saved settings, or the defaults when there are none (or they cannot be read).
pub fn load() -> Settings {
    if let (Some(old), Some(new)) = (old_file(), file()) {
        carry_over(&old, &new);
    }
    file()
        .and_then(|f| fs::read(f).ok())
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

pub fn save(settings: &Settings) -> Result<(), String> {
    let path = file().ok_or("APPDATA is not set")?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?;
    // Write a temporary file first so a crash never leaves half a settings file.
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let s: Settings = serde_json::from_str(r#"{"saveLogs":true}"#).unwrap();
        assert_eq!(
            s,
            Settings {
                save_logs: true,
                check_updates: true,
                intro_seen: false
            }
        );
        assert!(!Settings::default().save_logs);
    }

    /// A fresh folder under the workspace's `target`, so tests never write outside the project.
    fn scratch(name: &str) -> PathBuf {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/test-tmp")
            .join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn old_settings_are_copied_once_and_kept() {
        let dir = scratch("carry_over_copies");
        let (old, new) = (dir.join("old/settings.json"), dir.join("new/settings.json"));
        fs::create_dir_all(old.parent().unwrap()).unwrap();
        fs::write(&old, r#"{"saveLogs":true}"#).unwrap();

        carry_over(&old, &new);
        assert_eq!(fs::read_to_string(&new).unwrap(), r#"{"saveLogs":true}"#);
        assert!(old.exists());

        // A newer file is never overwritten by the old one.
        fs::write(&new, r#"{"saveLogs":false}"#).unwrap();
        carry_over(&old, &new);
        assert_eq!(fs::read_to_string(&new).unwrap(), r#"{"saveLogs":false}"#);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn nothing_to_carry_over_creates_nothing() {
        let dir = scratch("carry_over_none");
        let new = dir.join("new/settings.json");
        carry_over(&dir.join("missing.json"), &new);
        assert!(!new.exists());
        let _ = fs::remove_dir_all(&dir);
    }
}
