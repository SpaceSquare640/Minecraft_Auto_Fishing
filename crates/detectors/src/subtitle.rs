//! Scheme A (Java): the vanilla bite caption ("Fishing Bobber splashes" in English) read
//! by OCR. The caption text for the player's language comes from the game's own language
//! files at runtime, so this crate ships no game text.

use std::time::Duration;

use crate::{Bite, Detector, DetectorId, Onset};

/// OCR lines at least this similar to a bite caption count as one (Phase 0: OCR misreads
/// a letter or two; other captions are far apart).
pub const MIN_SIMILARITY: f32 = 0.8;

/// Lowercase letters and digits only: drops Java's `<` `>` arrows, spaces and punctuation.
/// Works for every script (`char::is_alphanumeric` covers CJK, Cyrillic, ...).
pub fn normalize(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// 1.0 for equal strings, 0.0 for nothing in common (Levenshtein over characters).
pub fn similarity(a: &str, b: &str) -> f32 {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let longest = a.len().max(b.len());
    if longest == 0 {
        return 1.0;
    }
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (above + 1)
                .min(row[j] + 1)
                .min(diagonal + usize::from(ca != cb));
            diagonal = above;
        }
    }
    1.0 - row[b.len()] as f32 / longest as f32
}

#[derive(Debug)]
pub struct SubtitleDetector {
    captions: Vec<String>,
    onset: Onset,
}

impl SubtitleDetector {
    /// `captions`: the bite caption in each language the game may show.
    pub fn new<S: AsRef<str>>(captions: impl IntoIterator<Item = S>) -> Self {
        let captions = captions
            .into_iter()
            .map(|c| normalize(c.as_ref()))
            .filter(|c| !c.is_empty())
            .collect();
        Self {
            captions,
            onset: Onset::default(),
        }
    }

    fn score(&self, line: &str) -> f32 {
        let line = normalize(line);
        self.captions
            .iter()
            .map(|c| similarity(&line, c))
            .fold(0.0, f32::max)
    }
}

impl Detector for SubtitleDetector {
    /// One string per OCR line.
    type Input = [String];

    fn id(&self) -> DetectorId {
        DetectorId::SubtitleOcr
    }

    fn feed(&mut self, lines: &[String], now: Duration) -> Option<Bite> {
        let best = lines.iter().map(|l| self.score(l)).fold(0.0, f32::max);
        let present = best >= MIN_SIMILARITY;
        self.onset.update(present, now).then(|| Bite {
            at: now,
            confidence: best,
            source: self.id(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn normalization_drops_arrows_and_case() {
        assert_eq!(
            normalize("< Fishing Bobber splashes >"),
            "fishingbobbersplashes"
        );
        assert_eq!(normalize("浮標濺水聲 >"), "浮標濺水聲");
    }

    #[test]
    fn ocr_misreads_still_match_but_other_captions_do_not() {
        let d = SubtitleDetector::new(["Fishing Bobber splashes"]);
        assert!(d.score("Fishing Bobber splashes >") > 0.99);
        assert!(d.score("Fishing Bobber spIashes") >= MIN_SIMILARITY);
        assert!(d.score("Splashing") < MIN_SIMILARITY);
        assert!(d.score("Bobber is retrieved") < MIN_SIMILARITY);
        assert!(d.score("Experience gained") < MIN_SIMILARITY);
    }

    #[test]
    fn bite_is_reported_when_the_caption_appears() {
        let mut d = SubtitleDetector::new(["Fishing Bobber splashes", "浮標濺水聲"]);
        let ms = Duration::from_millis;
        assert!(d.feed(&lines(&["Splashing"]), ms(0)).is_none());
        let bite = d
            .feed(&lines(&["Splashing", "浮標濺水聲 >"]), ms(100))
            .expect("bite");
        assert_eq!(bite.source, DetectorId::SubtitleOcr);
        assert!(d.feed(&lines(&["浮標濺水聲"]), ms(200)).is_none());
    }
}
