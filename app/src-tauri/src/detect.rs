//! Detection worker: turns the latest subtitle-area frame into bite and bobber events.
//!
//! Runs on its own thread so OCR (about 25 ms per frame) never blocks the driver or the UI,
//! and only while fishing, so an idle app uses no CPU for detection.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use maf_detectors::caption::{Bgra, lines, text_mask};
use maf_detectors::marker::MarkerDetector;
use maf_detectors::subtitle::{LineEvent, LineWatcher, SubtitleDetector};
use maf_detectors::{Bite, Detector};
use maf_platform_win::capture::FrameSlot;
use maf_platform_win::ocr::Ocr;

/// About 15 checks per second; captions stay on screen for seconds.
const INTERVAL: Duration = Duration::from_millis(66);
/// OCR is the expensive part; ten times per second is plenty.
const OCR_INTERVAL: Duration = Duration::from_millis(100);

/// What to look for, decided by the driver when it finds the game window.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Plan {
    /// Vanilla bite captions to read with OCR (Java only); empty disables OCR.
    pub bite: Vec<String>,
    /// Java "Bobber is thrown" / "Bobber is retrieved" in the game's language.
    pub thrown: Vec<String>,
    pub retrieved: Vec<String>,
    /// Windows OCR language tags to try, best first.
    pub ocr_languages: Vec<String>,
}

/// Something the worker saw.
#[derive(Debug, Clone, Copy)]
pub enum Seen {
    Bite(Bite),
    Line(LineEvent),
}

/// State shared between the driver and the worker.
#[derive(Default)]
pub struct Shared {
    pub slot: FrameSlot,
    pub plan: Mutex<Option<Plan>>,
    /// Set by the driver while fishing.
    pub active: AtomicBool,
    /// OCR language in use, for the status display.
    pub ocr_language: Mutex<Option<String>>,
}

struct Subtitles {
    ocr: Ocr,
    bite: SubtitleDetector,
    line: LineWatcher,
}

struct Detectors {
    plan: Plan,
    marker: MarkerDetector,
    subtitles: Option<Subtitles>,
    last_ocr: Option<Instant>,
}

impl Detectors {
    fn new(plan: Plan, shared: &Shared) -> Self {
        let ocr = (!plan.bite.is_empty())
            .then(|| {
                plan.ocr_languages
                    .iter()
                    .find_map(|tag| Ocr::for_language(tag))
            })
            .flatten();
        if let Ok(mut language) = shared.ocr_language.lock() {
            *language = ocr.as_ref().map(|o| o.language.clone());
        }
        let subtitles = ocr.map(|ocr| Subtitles {
            ocr,
            bite: SubtitleDetector::new(&plan.bite),
            line: LineWatcher::new(&plan.thrown, &plan.retrieved),
        });
        Self {
            plan,
            marker: MarkerDetector::default(),
            subtitles,
            last_ocr: None,
        }
    }
}

/// Runs forever; `report` is called for everything new the worker sees.
pub fn run(shared: Arc<Shared>, report: impl Fn(Seen)) {
    let epoch = Instant::now();
    let mut detectors: Option<Detectors> = None;
    let mut last_seq = 0;
    loop {
        std::thread::sleep(INTERVAL);
        if !shared.active.load(Ordering::Relaxed) {
            continue;
        }
        let plan = shared.plan.lock().ok().and_then(|p| p.clone());
        let Some(plan) = plan else { continue };
        if detectors.as_ref().is_none_or(|d| d.plan != plan) {
            detectors = Some(Detectors::new(plan, &shared));
        }
        let Some(d) = detectors.as_mut() else {
            continue;
        };
        let frame = shared.slot.lock().ok().and_then(|s| s.clone());
        let Some(frame) = frame.filter(|f| f.seq != last_seq) else {
            continue;
        };
        last_seq = frame.seq;
        let now = epoch.elapsed();

        let image = Bgra {
            width: frame.width,
            height: frame.height,
            stride: frame.width * 4,
            data: &frame.bgra,
        };
        if let Some(bite) = d.marker.feed(&lines(&text_mask(&image)), now) {
            report(Seen::Bite(bite));
        }
        if let Some(s) = d.subtitles.as_mut()
            && d.last_ocr.is_none_or(|t| t.elapsed() >= OCR_INTERVAL)
        {
            d.last_ocr = Some(Instant::now());
            if let Ok(text) = s.ocr.recognize(frame.width, frame.height, &frame.bgra) {
                if let Some(event) = s.line.feed(&text, now) {
                    report(Seen::Line(event));
                }
                if let Some(bite) = s.bite.feed(&text, now) {
                    report(Seen::Bite(bite));
                }
            }
        }
    }
}
