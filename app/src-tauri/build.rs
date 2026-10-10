fn main() {
    // Only the commands listed here can be called from the UI; the capability
    // file then grants them to the main window (ADR-011).
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["start", "stop", "status"])),
    )
    .expect("failed to run tauri-build");
}
