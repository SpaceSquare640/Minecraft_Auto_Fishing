//! Player settings, kept in `%APPDATA%\Minecraft Auto Fishing\settings.json`.
use std::fs;
use std::path::PathBuf;

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
    Some(
        PathBuf::from(std::env::var_os("APPDATA")?)
            .join("Minecraft Auto Fishing")
            .join("settings.json"),
    )
}

/// Saved settings, or the defaults when there are none (or they cannot be read).
pub fn load() -> Settings {
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
}
