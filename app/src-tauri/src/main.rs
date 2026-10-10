// No console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use maf_engine::{Config, Engine, Event, State};
use serde::Serialize;
use tauri::Manager;

const TICK: Duration = Duration::from_millis(50);

struct AppState {
    engine: Mutex<Engine>,
    epoch: Instant,
}

impl AppState {
    fn now(&self) -> Duration {
        self.epoch.elapsed()
    }

    fn engine(&self) -> MutexGuard<'_, Engine> {
        // A panic while holding the lock must not brick the UI.
        self.engine
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// P1.1: the returned actions are not executed yet; the Windows input layer
    /// (SendInput to the focused game window) arrives in P1.2.
    fn apply(&self, engine: &mut Engine, event: Event) {
        let _actions = engine.step(event, self.now());
    }
}

#[derive(Serialize)]
struct Status {
    state: &'static str,
    casts: u32,
    bites: u32,
    timeouts: u32,
}

fn status_of(engine: &Engine) -> Status {
    let stats = engine.stats();
    Status {
        state: match engine.state() {
            State::Idle => "idle",
            State::Casting => "casting",
            State::Waiting => "waiting",
            State::Reeling => "reeling",
            State::Paused => "paused",
        },
        casts: stats.casts,
        bites: stats.bites,
        timeouts: stats.timeouts,
    }
}

#[tauri::command]
fn start(app: tauri::State<'_, AppState>) -> Status {
    let mut engine = app.engine();
    // P1.1 has no game-window tracking yet, so the game counts as focused.
    // P1.2 replaces this with real foreground-window events.
    app.apply(&mut engine, Event::FocusGained);
    app.apply(&mut engine, Event::Start);
    status_of(&engine)
}

#[tauri::command]
fn stop(app: tauri::State<'_, AppState>) -> Status {
    let mut engine = app.engine();
    app.apply(&mut engine, Event::Stop);
    status_of(&engine)
}

#[tauri::command]
fn status(app: tauri::State<'_, AppState>) -> Status {
    status_of(&app.engine())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            engine: Mutex::new(Engine::new(Config::default())),
            epoch: Instant::now(),
        })
        .setup(|app| {
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(TICK);
                    let state = handle.state::<AppState>();
                    let mut engine = state.engine();
                    state.apply(&mut engine, Event::Tick);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![start, stop, status])
        .run(tauri::generate_context!())
        .expect("error while running Minecraft Auto Fishing");
}
