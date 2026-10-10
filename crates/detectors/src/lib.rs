//! Bite detectors share one interface, so the engine never needs to know how a bite
//! was found (ADR-003):
//! - [`marker::MarkerDetector`]: the `[AMF] BITE` caption of the resource pack, found by
//!   its pixel profile (Bedrock default, optional on Java)
//! - [`subtitle::SubtitleDetector`]: the vanilla bite caption read by OCR (Java default)
#![forbid(unsafe_code)]

pub mod caption;
pub mod marker;
pub mod subtitle;

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DetectorId {
    ResourcePackMarker,
    SubtitleOcr,
    Audio,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bite {
    pub at: Duration,
    /// 0.0 to 1.0
    pub confidence: f32,
    pub source: DetectorId,
}

pub trait Detector {
    /// What the detector consumes, e.g. the caption lines of one frame.
    type Input: ?Sized;

    fn id(&self) -> DetectorId;

    /// Returns a bite when a bite caption newly appears in this input.
    fn feed(&mut self, input: &Self::Input, now: Duration) -> Option<Bite>;
}

/// Turns "caption visible in this frame" into "caption newly appeared".
///
/// A caption counts as new only after it was absent for at least `DEBOUNCE`. This hides
/// one-frame flicker and, because nothing is reset on a cast, a caption that is still on
/// screen from the previous bite can never count as a new one (Phase 0, §7.2).
#[derive(Debug, Default, Clone)]
pub struct Onset {
    last_seen: Option<Duration>,
}

impl Onset {
    pub const DEBOUNCE: Duration = Duration::from_millis(500);

    pub fn update(&mut self, present: bool, now: Duration) -> bool {
        let new = present
            && self
                .last_seen
                .is_none_or(|seen| now.saturating_sub(seen) >= Self::DEBOUNCE);
        if present {
            self.last_seen = Some(now);
        }
        new
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(v: u64) -> Duration {
        Duration::from_millis(v)
    }

    #[test]
    fn onset_fires_once_per_appearance_and_ignores_flicker() {
        let mut o = Onset::default();
        assert!(!o.update(false, ms(0)));
        assert!(o.update(true, ms(100)));
        assert!(!o.update(true, ms(130)));
        assert!(!o.update(false, ms(160))); // one-frame flicker...
        assert!(!o.update(true, ms(190))); // ...is not a new caption
        assert!(!o.update(false, ms(400)));
        assert!(o.update(true, ms(800))); // absent for 610 ms: new caption
    }
}
