//! Connects the engine to the game: finds the window, turns focus changes into engine
//! events, performs the engine's right clicks and keeps a capture session for P1.3.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use maf_detectors::{Bite, DetectorId};
use maf_engine::{Action, Config, Engine, Event, State};
use maf_game_profile::{JavaProfile, ocr_language_tag};
use maf_platform_win::capture::Capture;
use maf_platform_win::{Edition, GameWindow, find_game_window, right_click};
use serde::Serialize;

use crate::detect::{self, Plan};

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
    detection: Arc<detect::Shared>,
    last_bite: Option<(DetectorId, Instant)>,
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
    detection: DetectionInfo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionInfo {
    /// Detectors in use for the current game.
    methods: Vec<&'static str>,
    ocr_language: Option<String>,
    last_bite_source: Option<&'static str>,
    last_bite_seconds_ago: Option<f32>,
}

#[derive(Serialize)]
pub struct GameInfo {
    edition: &'static str,
    title: String,
}

impl Driver {
    pub fn new(detection: Arc<detect::Shared>) -> Self {
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
            detection,
            last_bite: None,
        }
    }

    /// A detector saw a new bite caption.
    pub fn bite(&mut self, bite: Bite) {
        self.last_bite = Some((bite.source, Instant::now()));
        self.feed(Event::Bite);
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
        let fishing = matches!(
            self.engine.state(),
            State::Casting | State::Waiting | State::Reeling
        );
        self.detection.active.store(fishing, Ordering::Relaxed);
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
            detection: DetectionInfo {
                methods: match self.game.as_ref().map(|g| g.edition) {
                    Some(Edition::Java) => vec!["subtitles", "resource pack"],
                    Some(Edition::Bedrock) => vec!["resource pack"],
                    None => vec![],
                },
                ocr_language: self
                    .detection
                    .ocr_language
                    .lock()
                    .ok()
                    .and_then(|l| l.clone()),
                last_bite_source: self.last_bite.map(|(source, _)| match source {
                    DetectorId::ResourcePackMarker => "resource pack",
                    DetectorId::SubtitleOcr => "subtitles",
                    DetectorId::Audio => "audio",
                }),
                last_bite_seconds_ago: self.last_bite.map(|(_, at)| at.elapsed().as_secs_f32()),
            },
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
            self.set_plan(None);
        }
        let due = self.last_find.is_none_or(|t| t.elapsed() >= FIND_INTERVAL);
        let capture_down = self.capture.as_ref().is_none_or(|c| !c.is_running());
        if !due || (self.game.is_some() && !capture_down) {
            return;
        }
        self.last_find = Some(Instant::now());
        if self.game.is_none() {
            self.game = find_game_window();
            let plan = self.game.as_ref().map(|g| plan_for(g.edition));
            self.set_plan(plan);
        }
        if let Some(game) = &self.game {
            match Capture::start(game, self.detection.slot.clone()) {
                Ok(capture) => {
                    self.fps_mark = (Instant::now(), 0);
                    self.capture = Some(capture);
                }
                Err(e) => self.last_error = Some(format!("screen capture failed: {e}")),
            }
        }
    }

    fn set_plan(&self, plan: Option<Plan>) {
        if let Ok(mut current) = self.detection.plan.lock() {
            *current = plan;
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

/// Bedrock: the resource pack marker only (its vanilla caption is shared with every splash).
/// Java: the vanilla caption by OCR in the game's language, plus the marker if the pack is on.
fn plan_for(edition: Edition) -> Plan {
    match edition {
        Edition::Bedrock => Plan {
            captions: Vec::new(),
            ocr_languages: Vec::new(),
        },
        Edition::Java => {
            let profile = JavaProfile::default_location();
            let lang = profile
                .as_ref()
                .and_then(JavaProfile::language)
                .unwrap_or_else(|| "en_us".to_owned());
            let captions = profile.map(|p| p.bite_captions(&lang)).unwrap_or_default();
            let mut ocr_languages = vec![ocr_language_tag(&lang)];
            if lang != "en_us" {
                ocr_languages.push("en-US".to_owned());
            }
            Plan {
                captions,
                ocr_languages,
            }
        }
    }
}
