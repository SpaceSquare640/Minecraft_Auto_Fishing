// No console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod detect;
mod driver;

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use driver::{Driver, Status};
use maf_platform_win::hotkey;
use tauri::Manager;

const TICK: Duration = Duration::from_millis(50);

struct AppState(Mutex<Driver>);

impl AppState {
    fn driver(&self) -> MutexGuard<'_, Driver> {
        // A panic while holding the lock must not brick the app.
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[tauri::command]
fn start(app: tauri::State<'_, AppState>) -> Status {
    let mut driver = app.driver();
    driver.start();
    driver.status()
}

#[tauri::command]
fn stop(app: tauri::State<'_, AppState>) -> Status {
    let mut driver = app.driver();
    driver.stop();
    driver.status()
}

#[tauri::command]
fn status(app: tauri::State<'_, AppState>) -> Status {
    app.driver().status()
}

fn main() {
    let detection = Arc::new(detect::Shared::default());
    tauri::Builder::default()
        .manage(AppState(Mutex::new(Driver::new(detection.clone()))))
        .setup(move |app| {
            let handle = app.handle().clone();

            let on_bite = handle.clone();
            std::thread::Builder::new()
                .name("maf-detect".into())
                .spawn(move || {
                    detect::run(detection, move |seen| {
                        let state = on_bite.state::<AppState>();
                        let mut driver = state.driver();
                        match seen {
                            detect::Seen::Bite(bite) => driver.bite(bite),
                            detect::Seen::Line(event) => driver.line(event),
                        }
                    })
                })?;

            let on_press = handle.clone();
            let registered = hotkey::spawn(move || on_press.state::<AppState>().driver().toggle());
            {
                let state = handle.state::<AppState>();
                let mut driver = state.driver();
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
                    handle.state::<AppState>().driver().tick();
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![start, stop, status])
        .run(tauri::generate_context!())
        .expect("error while running Minecraft Auto Fishing");
}
