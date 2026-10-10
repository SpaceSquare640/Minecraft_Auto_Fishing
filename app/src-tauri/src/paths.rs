//! Where the app keeps its own data: folders named after the bundle identifier, apart from the
//! install folder (`%LOCALAPPDATA%\Minecraft Auto Fishing`). They are the folders the Windows
//! uninstaller's "Delete the application data" option removes.
use std::path::PathBuf;

/// Must match `identifier` in tauri.conf.json (checked by a test).
pub const IDENTIFIER: &str = "io.github.spacesquare640.minecraftautofishing";

/// Folder name used before 0.1.0-alpha.2. Only the settings file is carried over from it.
pub const OLD_NAME: &str = "Minecraft Auto Fishing";

/// Settings: `%APPDATA%\<identifier>`.
pub fn config_dir() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("APPDATA")?).join(IDENTIFIER))
}

/// Local data such as log files: `%LOCALAPPDATA%\<identifier>`.
pub fn local_dir() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join(IDENTIFIER))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_matches_tauri_conf() {
        let conf: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(conf["identifier"], IDENTIFIER);
    }
}
