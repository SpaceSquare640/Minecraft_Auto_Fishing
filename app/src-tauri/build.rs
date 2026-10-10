const COMMANDS: &[&str] = &[
    "app_info",
    "start",
    "stop",
    "status",
    "log_since",
    "get_settings",
    "set_settings",
    "open_link",
    "open_log_folder",
    "install_pack",
];

fn main() {
    // Only the commands listed here can be called from the UI; the capability
    // file then grants them to the main window (ADR-011).
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("failed to run tauri-build");
}
