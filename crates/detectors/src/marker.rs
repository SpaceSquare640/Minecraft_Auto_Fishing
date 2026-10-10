//! Scheme C: the resource pack replaces the bite caption with `[AMF] BITE`. Its pixel
//! profile is the same in Bedrock and Java, in every game language and at every GUI
//! scale, so no OCR is needed. Phase 0: 10/10 bites, 0 false hits in 16,568 other lines.

use std::ops::RangeInclusive;
use std::time::Duration;

use crate::caption::{Line, PROFILE_BINS};
use crate::{Bite, Detector, DetectorId, Onset};

/// Column profile of `[AMF] BITE`, measured from the Phase 0 recordings.
pub const MARKER_PROFILE: [f32; PROFILE_BINS] = [
    0.308049, 0.103181, 0.086204, 0.072556, 0.225138, 0.096164, 0.093409, 0.180627, 0.130805,
    0.188840, 0.148560, 0.050498, 0.050072, 0.267634, 0.042194, 0.296346, 0.096478, 0.091973,
    0.050811, 0.041089, 0.035455, 0.094636, 0.185485, 0.172683, 0.007507, 0.008319, 0.008670,
    0.005479, 0.240716, 0.169037, 0.139651, 0.140583, 0.176386, 0.017467, 0.136256, 0.263011,
    0.068422, 0.024631, 0.055615, 0.180070, 0.178908, 0.050497, 0.020811, 0.224313, 0.180429,
    0.140556, 0.100424, 0.095890,
];
/// The marker measures 7.29; other captions of similar width differ in profile.
pub const MARKER_ASPECT: RangeInclusive<f32> = 6.8..=7.8;
/// Marker lines score >= 0.99; the best other caption scored 0.84.
pub const MIN_SIMILARITY: f32 = 0.95;

/// Similarity to the marker when `line` has the marker's shape.
pub fn marker_similarity(line: &Line) -> Option<f32> {
    if !MARKER_ASPECT.contains(&line.aspect) {
        return None;
    }
    let similarity: f32 = line
        .profile
        .iter()
        .zip(&MARKER_PROFILE)
        .map(|(a, b)| a * b)
        .sum();
    (similarity >= MIN_SIMILARITY).then_some(similarity)
}

#[derive(Debug, Default)]
pub struct MarkerDetector {
    onset: Onset,
}

impl Detector for MarkerDetector {
    type Input = [Line];

    fn id(&self) -> DetectorId {
        DetectorId::ResourcePackMarker
    }

    fn feed(&mut self, lines: &[Line], now: Duration) -> Option<Bite> {
        let best = lines.iter().filter_map(marker_similarity).reduce(f32::max);
        self.onset.update(best.is_some(), now).then(|| Bite {
            at: now,
            confidence: best.unwrap_or_default(),
            source: self.id(),
        })
    }
}
