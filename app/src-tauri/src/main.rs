// No console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod detect;
mod driver;
mod eventlog;
mod packs;
mod settings;

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use driver::{Driver, SharedLog, Status};
use eventlog::{Entry, EventLog};
use maf_platform_win::{hotkey, shell};
use serde::Serialize;
use settings::Settings;
use tauri::Manager;

const TICK: Duration = Duration::from_millis(50);

/// Fixed link targets: the UI can only pick one by name, never pass a URL.
const LINKS: [(&str, &str); 3] = [
    (
        "github",
        "https://github.com/SpaceSquare640/Minecraft_Auto_Fishing",
    ),
    ("discord", "https://discord.gg/aaUQVJeCgC"),
    (
        "website",
        "https://spacesquare640.github.io/Minecraft_Auto_Fishing/",
    ),
];

struct AppState {
    driver: Mutex<Driver>,
    log: SharedLog,
    settings: Mutex<Settings>,
}

/// A panic while holding a lock must not brick the app.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Serialize)]
struct AppInfo {
    version: &'static str,
}

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[tauri::command]
fn start(app: tauri::State<'_, AppState>) -> Status {
    let mut driver = lock(&app.driver);
    driver.start();
    driver.status()
}

#[tauri::command]
fn stop(app: tauri::State<'_, AppState>) -> Status {
    let mut driver = lock(&app.driver);
    driver.stop();
    driver.status()
}

#[tauri::command]
fn status(app: tauri::State<'_, AppState>) -> Status {
    lock(&app.driver).status()
}

#[tauri::command]
fn log_since(app: tauri::State<'_, AppState>, id: u64) -> Vec<Entry> {
    lock(&app.log).since(id)
}

#[tauri::command]
fn get_settings(app: tauri::State<'_, AppState>) -> Settings {
    lock(&app.settings).clone()
}

#[tauri::command]
fn set_settings(app: tauri::State<'_, AppState>, settings: Settings) -> Result<Settings, String> {
    settings::save(&settings)?;
    {
        let mut log = lock(&app.log);
        log.set_save(settings.save_logs);
        log.add(format!(
            "Settings: save logs {}, check for updates {}",
            if settings.save_logs { "on" } else { "off" },
            if settings.check_updates { "on" } else { "off" }
        ));
    }
    *lock(&app.settings) = settings.clone();
    Ok(settings)
}

#[tauri::command]
fn open_link(target: String) -> Result<(), String> {
    let (_, url) = LINKS
        .iter()
        .find(|(name, _)| *name == target)
        .ok_or("unknown link")?;
    shell::open_url(url)
}

#[tauri::command]
fn open_log_folder() -> Result<(), String> {
    let dir = EventLog::dir().ok_or("LOCALAPPDATA is not set")?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    shell::open_path(&dir)
}

#[tauri::command]
fn install_pack(app: tauri::State<'_, AppState>, edition: String) -> Result<String, String> {
    let result = match edition.as_str() {
        "bedrock" => packs::install_bedrock(),
        "java" => packs::install_java(),
        _ => Err("unknown edition".to_owned()),
    };
    lock(&app.log).add(match &result {
        Ok(_) => format!("Resource pack installed for {edition}"),
        Err(e) => format!("Resource pack not installed for {edition}: {e}"),
    });
    result
}

fn main() {
    let settings = settings::load();
    let log: SharedLog = Arc::new(Mutex::new(EventLog::new(settings.save_logs)));
    lock(&log).add(format!(
        "Minecraft Auto Fishing {} started",
        env!("CARGO_PKG_VERSION")
    ));
    let detection = Arc::new(detect::Shared::default());
    tauri::Builder::default()
        .manage(AppState {
            driver: Mutex::new(Driver::new(detection.clone(), log.clone())),
            log,
            settings: Mutex::new(settings),
        })
        .setup(move |app| {
            let handle = app.handle().clone();

            let on_seen = handle.clone();
            std::thread::Builder::new()
                .name("maf-detect".into())
                .spawn(move || {
                    detect::run(detection, move |seen| {
                        let state = on_seen.state::<AppState>();
                        let mut driver = lock(&state.driver);
                        match seen {
                            detect::Seen::Bite(bite) => driver.bite(bite),
                            detect::Seen::Line(event) => driver.line(event),
                        }
                    })
                })?;

            let on_press = handle.clone();
            let registered =
                hotkey::spawn(move || lock(&on_press.state::<AppState>().driver).toggle());
            {
                let state = handle.state::<AppState>();
                let mut driver = lock(&state.driver);
                match registered {
                    Ok(()) => driver.set_hotkey(hotkey::KEY_NAME),
                    Err(e) => {
                        driver.set_error(format!("hotkey {} unavailable: {e}", hotkey::KEY_NAME))
                    }
                }
            }

            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(TICK);
                    lock(&handle.state::<AppState>().driver).tick();
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            start,
            stop,
            status,
            log_since,
            get_settings,
            set_settings,
            open_link,
            open_log_folder,
            install_pack
        ])
        .run(tauri::generate_context!())
        .expect("error while running Minecraft Auto Fishing");
}
