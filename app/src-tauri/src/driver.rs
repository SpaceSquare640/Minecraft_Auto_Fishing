//! Connects the engine to the game: finds the window, turns focus changes into engine
//! events, performs the engine's right clicks and keeps a capture session for P1.3.

use std::time::{Duration, Instant};

use maf_engine::{Action, Config, Engine, Event, State};
use maf_platform_win::capture::Capture;
use maf_platform_win::{Edition, GameWindow, find_game_window, right_click};
use serde::Serialize;

/// How often to look for the game window while none is known.
const FIND_INTERVAL: Duration = Duration::from_secs(1);

pub struct Driver {
    engine: Engine,
    epoch: Instant,
    game: Option<GameWindow>,
    capture: Option<Capture>,
    focused: bool,
    last_find: Option<Instant>,
    fps: Option<f32>,
    fps_mark: (Instant, u64),
    hotkey: Option<&'static str>,
    last_error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    state: &'static str,
    casts: u32,
    bites: u32,
    timeouts: u32,
    game: Option<GameInfo>,
    focused: bool,
    capture_fps: Option<f32>,
    hotkey: Option<&'static str>,
    last_error: Option<String>,
}

#[derive(Serialize)]
pub struct GameInfo {
    edition: &'static str,
    title: String,
}

impl Driver {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            engine: Engine::new(Config::default()),
            epoch: now,
            game: None,
            capture: None,
            focused: false,
            last_find: None,
            fps: None,
            fps_mark: (now, 0),
            hotkey: None,
            last_error: None,
        }
    }

    pub fn set_hotkey(&mut self, key: &'static str) {
        self.hotkey = Some(key);
    }

    pub fn set_error(&mut self, message: String) {
        self.last_error = Some(message);
    }

    pub fn start(&mut self) {
        self.feed(Event::Start);
    }

    pub fn stop(&mut self) {
        self.feed(Event::Stop);
    }

    /// Hotkey: start when stopped or paused, otherwise stop.
    pub fn toggle(&mut self) {
        match self.engine.state() {
            State::Idle | State::Paused => self.start(),
            _ => self.stop(),
        }
    }

    /// Called on every clock tick.
    pub fn tick(&mut self) {
        self.track_game();
        self.update_fps();
        let focused = self.game.as_ref().is_some_and(GameWindow::is_foreground);
        if focused != self.focused {
            self.focused = focused;
            self.feed(if focused {
                Event::FocusGained
            } else {
                Event::FocusLost
            });
        }
        self.feed(Event::Tick);
    }

    pub fn status(&self) -> Status {
        let stats = self.engine.stats();
        Status {
            state: match self.engine.state() {
                State::Idle => "idle",
                State::Casting => "casting",
                State::Waiting => "waiting",
                State::Reeling => "reeling",
                State::Paused => "paused",
            },
            casts: stats.casts,
            bites: stats.bites,
            timeouts: stats.timeouts,
            game: self.game.as_ref().map(|g| GameInfo {
                edition: match g.edition {
                    Edition::Java => "java",
                    Edition::Bedrock => "bedrock",
                },
                title: g.title.clone(),
            }),
            focused: self.focused,
            capture_fps: self.fps,
            hotkey: self.hotkey,
            last_error: self.last_error.clone(),
        }
    }

    fn now(&self) -> Duration {
        self.epoch.elapsed()
    }

    fn feed(&mut self, event: Event) {
        let now = self.now();
        for action in self.engine.step(event, now) {
            match action {
                Action::RightClick => self.click(now),
            }
        }
    }

    fn click(&mut self, now: Duration) {
        let result = match &self.game {
            Some(game) => right_click(game).map_err(|e| e.to_string()),
            None => Err("Minecraft window not found".to_owned()),
        };
        match result {
            Ok(()) => self.last_error = None,
            Err(message) => {
                self.last_error = Some(message);
                self.engine.input_failed(now);
            }
        }
    }

    fn track_game(&mut self) {
        if self.game.as_ref().is_some_and(|g| !g.is_alive()) {
            self.game = None;
            self.capture = None;
        }
        let due = self.last_find.is_none_or(|t| t.elapsed() >= FIND_INTERVAL);
        let capture_down = self.capture.as_ref().is_none_or(|c| !c.is_running());
        if !due || (self.game.is_some() && !capture_down) {
            return;
        }
        self.last_find = Some(Instant::now());
        if self.game.is_none() {
            self.game = find_game_window();
        }
        if let Some(game) = &self.game {
            match Capture::start(game) {
                Ok(capture) => {
                    self.fps_mark = (Instant::now(), 0);
                    self.capture = Some(capture);
                }
                Err(e) => self.last_error = Some(format!("screen capture failed: {e}")),
            }
        }
    }

    fn update_fps(&mut self) {
        let Some(capture) = &self.capture else {
            self.fps = None;
            return;
        };
        let (since, frames) = self.fps_mark;
        let elapsed = since.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let total = capture.frames();
            self.fps = Some((total - frames) as f32 / elapsed.as_secs_f32());
            self.fps_mark = (Instant::now(), total);
        }
    }
}
