//! Bite detectors share one interface, so the engine never needs to know how a
//! bite was found (ADR-003). Planned implementations (P1.3):
//! - `ResourcePackMarker`: pixel match of the `[AMF] BITE` caption (Bedrock default)
//! - `SubtitleOcr`: Windows OCR of the vanilla bite caption (Java default)
#![forbid(unsafe_code)]

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
    /// What the detector consumes, e.g. a cropped frame or an audio chunk.
    type Input: ?Sized;

    fn id(&self) -> DetectorId;

    /// Returns a bite when this input completes one.
    fn feed(&mut self, input: &Self::Input, now: Duration) -> Option<Bite>;

    /// Forget any partial state (called after each cast).
    fn reset(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reports a bite the first time it sees `true`.
    struct Flag {
        fired: bool,
    }

    impl Detector for Flag {
        type Input = bool;

        fn id(&self) -> DetectorId {
            DetectorId::ResourcePackMarker
        }

        fn feed(&mut self, input: &bool, now: Duration) -> Option<Bite> {
            if *input && !self.fired {
                self.fired = true;
                return Some(Bite {
                    at: now,
                    confidence: 1.0,
                    source: self.id(),
                });
            }
            None
        }

        fn reset(&mut self) {
            self.fired = false;
        }
    }

    #[test]
    fn detectors_are_usable_through_the_trait() {
        let mut d = Flag { fired: false };
        assert!(d.feed(&false, Duration::ZERO).is_none());
        let bite = d.feed(&true, Duration::from_secs(1)).expect("bite");
        assert_eq!(bite.source, DetectorId::ResourcePackMarker);
        assert!(d.feed(&true, Duration::from_secs(2)).is_none());
        d.reset();
        assert!(d.feed(&true, Duration::from_secs(3)).is_some());
    }
}
