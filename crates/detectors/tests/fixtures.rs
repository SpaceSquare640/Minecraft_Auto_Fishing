//! Caption lines cut from the Phase 0 recordings (binarized, text pixels only):
//! the `[AMF] BITE` marker in Bedrock, Java (English) and Java (Traditional Chinese),
//! and other captions, including one of almost the same width as the marker.

use std::path::Path;

use maf_detectors::caption::{Mask, line_in_band};
use maf_detectors::marker::marker_similarity;

/// Plain PBM (P1) reader.
fn load(name: &str) -> Mask {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("{name}.pbm"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut tokens = text
        .lines()
        .filter(|l| !l.starts_with('#'))
        .flat_map(str::split_whitespace);
    assert_eq!(tokens.next(), Some("P1"));
    let width: usize = tokens.next().unwrap().parse().unwrap();
    let height: usize = tokens.next().unwrap().parse().unwrap();
    let bits: Vec<bool> = tokens.flat_map(str::chars).map(|c| c == '1').collect();
    assert_eq!(bits.len(), width * height, "{name}");
    Mask {
        width,
        height,
        bits,
    }
}

fn similarity(name: &str) -> Option<f32> {
    let mask = load(name);
    let line = line_in_band(&mask, 0, mask.height).expect("a line");
    marker_similarity(&line)
}

#[test]
fn the_marker_is_recognised_in_both_editions_and_any_language() {
    for name in ["marker_bedrock", "marker_java_en", "marker_java_zh_tw"] {
        let s = similarity(name).unwrap_or_else(|| panic!("{name} not recognised"));
        assert!(s > 0.99, "{name}: {s}");
    }
}

#[test]
fn other_captions_are_not_the_marker() {
    for name in [
        "other_bedrock_a",
        "other_bedrock_b",
        "other_java_a",
        "other_java_b",
        "other_java_c",
        "other_java_d",
        "other_java_zh_tw_same_width",
    ] {
        assert_eq!(similarity(name), None, "{name}");
    }
}
