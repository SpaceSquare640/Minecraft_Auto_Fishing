//! Connects the engine to the game: finds the window, turns focus changes into engine
//! events, performs the engine's right clicks and keeps a capture session for P1.3.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use maf_detectors::subtitle::LineEvent;
use maf_detectors::{Bite, DetectorId};
use maf_engine::{Action, Config, Engine, Event, State, Stats};
use maf_game_profile::{BITE_KEY, JavaProfile, RETRIEVE_KEY, THROW_KEY, ocr_language_tag};
use maf_platform_win::capture::Capture;
use maf_platform_win::{Edition, GameWindow, find_game_window, right_click};
use serde::Serialize;

use crate::detect::{self, Plan};
use crate::eventlog::EventLog;

pub type SharedLog = Arc<std::sync::Mutex<EventLog>>;

/// How often to look for the game window while none is known.
const FIND_INTERVAL: Duration = Duration::from_secs(1);

/// Timeouts in a row without a bite before the app suggests checking the subtitle setup.
const NO_BITE_HINT_AFTER: u32 = 2;

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
    no_bite: NoBiteWatch,
    log: SharedLog,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    state: &'static str,
    casts: u32,
    bites: u32,
    timeouts: u32,
    resyncs: u32,
    game: Option<GameInfo>,
    focused: bool,
    capture_fps: Option<f32>,
    hotkey: Option<&'static str>,
    last_error: Option<String>,
    detection: DetectionInfo,
    /// The bite subtitle was not seen for several waits in a row.
    no_bite_hint: bool,
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
    pub fn new(detection: Arc<detect::Shared>, log: SharedLog) -> Self {
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
            no_bite: NoBiteWatch::default(),
            log,
        }
    }

    /// A detector saw a new bite caption.
    /// The game showed the bobber being thrown or pulled in.
    pub fn line(&mut self, event: LineEvent) {
        self.log(match event {
            LineEvent::Out => "Game: bobber thrown",
            LineEvent::In => "Game: bobber retrieved",
        });
        self.feed(match event {
            LineEvent::Out => Event::LineOut,
            LineEvent::In => Event::LineIn,
        });
    }

    pub fn bite(&mut self, bite: Bite) {
        let source = match bite.source {
            DetectorId::ResourcePackMarker => "resource pack",
            DetectorId::SubtitleOcr => "subtitles",
            DetectorId::Audio => "audio",
        };
        self.log(format!("Bite seen ({source}, {:.2})", bite.confidence));
        self.last_bite = Some((bite.source, Instant::now()));
        self.feed(Event::Bite);
    }

    pub fn set_hotkey(&mut self, key: &'static str) {
        self.hotkey = Some(key);
    }

    pub fn set_error(&mut self, message: String) {
        self.log(format!("Error: {message}"));
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
            state: state_name(self.engine.state()),
            casts: stats.casts,
            bites: stats.bites,
            timeouts: stats.timeouts,
            resyncs: stats.resyncs,
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
            no_bite_hint: self.no_bite.showing(),
        }
    }

    fn now(&self) -> Duration {
        self.epoch.elapsed()
    }

    fn feed(&mut self, event: Event) {
        let now = self.now();
        let (before, stats) = (self.engine.state(), self.engine.stats());
        for action in self.engine.step(event, now) {
            match action {
                Action::RightClick => self.click(now),
            }
        }
        let after = self.engine.state();
        if self.no_bite.observe(event, stats, self.engine.stats()) {
            self.log("No bite subtitle seen in the last waits: check the subtitle setup");
        }
        if self.engine.stats().resyncs > stats.resyncs {
            self.log("Bobber state corrected to match the game");
        }
        if after != before {
            self.log(format!(
                "State: {} -> {}",
                state_name(before),
                state_name(after)
            ));
        }
    }

    fn log(&self, text: impl Into<String>) {
        if let Ok(mut log) = self.log.lock() {
            log.add(text);
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
                self.log(format!("Right click not sent: {message}"));
                self.last_error = Some(message);
                self.engine.input_failed(now);
            }
        }
    }

    fn track_game(&mut self) {
        if self.game.as_ref().is_some_and(|g| !g.is_alive()) {
            self.log("Minecraft window closed");
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
            if let Some(game) = &self.game {
                self.log(format!("Found {:?} Edition: {}", game.edition, game.title));
            }
            let plan = self.game.as_ref().map(|g| plan_for(g.edition));
            self.set_plan(plan);
        }
        if let Some(game) = &self.game {
            match Capture::start(game, self.detection.slot.clone()) {
                Ok(capture) => {
                    self.fps_mark = (Instant::now(), 0);
                    self.capture = Some(capture);
                }
                Err(e) => {
                    self.log(format!("Screen capture failed: {e}"));
                    self.last_error = Some(format!("screen capture failed: {e}"));
                }
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
/// Java: the vanilla caption by OCR in the game's language, plus the marker if the pack is on,
/// and the bobber's own captions to follow its real state.
fn plan_for(edition: Edition) -> Plan {
    match edition {
        Edition::Bedrock => Plan::default(),
        Edition::Java => {
            let profile = JavaProfile::default_location();
            let lang = profile
                .as_ref()
                .and_then(JavaProfile::language)
                .unwrap_or_else(|| "en_us".to_owned());
            let caption = |key| {
                profile
                    .as_ref()
                    .map(|p| p.captions(key, &lang))
                    .unwrap_or_default()
            };
            let mut ocr_languages = vec![ocr_language_tag(&lang)];
            if lang != "en_us" {
                ocr_languages.push("en-US".to_owned());
            }
            Plan {
                bite: caption(BITE_KEY),
                thrown: caption(THROW_KEY),
                retrieved: caption(RETRIEVE_KEY),
                ocr_languages,
            }
        }
    }
}

/// Counts waits that ended without a bite. Several in a row usually means the game shows no
/// bite subtitle (pack off, subtitles off or the bobber too far away).
#[derive(Default)]
struct NoBiteWatch {
    streak: u32,
}

impl NoBiteWatch {
    /// Returns true when the hint starts showing.
    fn observe(&mut self, event: Event, before: Stats, after: Stats) -> bool {
        if event == Event::Start || after.bites > before.bites {
            self.streak = 0;
            return false;
        }
        let was = self.showing();
        self.streak += after.timeouts - before.timeouts;
        !was && self.showing()
    }

    fn showing(&self) -> bool {
        self.streak >= NO_BITE_HINT_AFTER
    }
}
fn state_name(state: State) -> &'static str {
    match state {
        State::Idle => "idle",
        State::Starting => "starting",
        State::Casting => "casting",
        State::Waiting => "waiting",
        State::Reeling => "reeling",
        State::Paused => "paused",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(bites: u32, timeouts: u32) -> Stats {
        Stats {
            bites,
            timeouts,
            ..Stats::default()
        }
    }

    #[test]
    fn hint_after_two_timeouts_in_a_row() {
        let mut watch = NoBiteWatch::default();
        assert!(!watch.observe(Event::Tick, stats(0, 0), stats(0, 1)));
        assert!(!watch.showing());
        assert!(watch.observe(Event::Tick, stats(0, 1), stats(0, 2)));
        assert!(watch.showing());
        // Reported once, but keeps showing.
        assert!(!watch.observe(Event::Tick, stats(0, 2), stats(0, 3)));
        assert!(watch.showing());
    }

    #[test]
    fn bite_or_start_clears_the_hint() {
        let mut watch = NoBiteWatch::default();
        watch.observe(Event::Tick, stats(0, 0), stats(0, 2));
        assert!(watch.showing());
        watch.observe(Event::Bite, stats(0, 2), stats(1, 2));
        assert!(!watch.showing());

        watch.observe(Event::Tick, stats(1, 2), stats(1, 4));
        assert!(watch.showing());
        watch.observe(Event::Start, stats(1, 4), stats(1, 4));
        assert!(!watch.showing());
    }

    #[test]
    fn a_bite_between_timeouts_restarts_the_count() {
        let mut watch = NoBiteWatch::default();
        watch.observe(Event::Tick, stats(0, 0), stats(0, 1));
        watch.observe(Event::Bite, stats(0, 1), stats(1, 1));
        watch.observe(Event::Tick, stats(1, 1), stats(1, 2));
        assert!(!watch.showing());
    }
}
