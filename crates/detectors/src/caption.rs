//! Caption lines from a crop of the game's subtitle area. Tuned on the Phase 0
//! recordings of both editions (App Design §7.3.2):
//!
//! 1. text mask: bright, neutral pixels inside a long run of dark caption background
//!    (scene textures such as stone are grey too, but mid-tones break their runs)
//! 2. bands: runs of rows that contain text
//! 3. words: split at gaps wider than a quarter of the band height; a narrow first or
//!    last word is Java's direction arrow and is dropped
//! 4. features: aspect (width / height) and the column profile resampled to fixed bins

/// Number of bins in a line's column profile.
pub const PROFILE_BINS: usize = 48;

const TEXT_MIN: u8 = 170; // darkest channel of a text pixel (fresh captions are white)
const TEXT_MAX_CHROMA: u8 = 25; // text is neutral grey or white
const DARK_MAX: u8 = 110; // brightest channel of the translucent caption background
const RUN_FRAC: f32 = 0.2; // caption rows are dark/text runs this share of the width
const MIN_BAND: usize = 10; // px; thinner bands are noise
const ROW_MIN_PIXELS: usize = 3; // a text row has at least this many text pixels

/// A borrowed BGRA image, as delivered by Windows.Graphics.Capture.
pub struct Bgra<'a> {
    pub width: usize,
    pub height: usize,
    /// Bytes per row (at least `width * 4`).
    pub stride: usize,
    pub data: &'a [u8],
}

/// One bit per pixel, row-major.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mask {
    pub width: usize,
    pub height: usize,
    pub bits: Vec<bool>,
}

impl Mask {
    pub fn get(&self, x: usize, y: usize) -> bool {
        self.bits[y * self.width + x]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub y0: usize,
    pub y1: usize,
    pub x0: usize,
    pub x1: usize,
    /// Width divided by height; independent of the GUI scale.
    pub aspect: f32,
    /// Column profile, L2-normalised; also independent of the GUI scale.
    pub profile: [f32; PROFILE_BINS],
}

/// Caption text pixels of `img` (see the module docs).
pub fn text_mask(img: &Bgra<'_>) -> Mask {
    let (w, h) = (img.width, img.height);
    let min_run = (RUN_FRAC * w as f32) as usize;
    let mut bits = vec![false; w * h];
    let mut text = vec![false; w];
    let mut allowed = vec![false; w];
    for y in 0..h {
        let row = &img.data[y * img.stride..y * img.stride + w * 4];
        for x in 0..w {
            let px = &row[x * 4..x * 4 + 3];
            let lo = px[0].min(px[1]).min(px[2]);
            let hi = px[0].max(px[1]).max(px[2]);
            text[x] = lo > TEXT_MIN && hi - lo < TEXT_MAX_CHROMA;
            allowed[x] = text[x] || hi < DARK_MAX;
        }
        let mut x = 0;
        while x < w {
            if !allowed[x] {
                x += 1;
                continue;
            }
            let start = x;
            while x < w && allowed[x] {
                x += 1;
            }
            if x - start >= min_run {
                for i in start..x {
                    bits[y * w + i] = text[i];
                }
            }
        }
    }
    Mask {
        width: w,
        height: h,
        bits,
    }
}

/// All caption-like text lines in `mask`, top to bottom.
pub fn lines(mask: &Mask) -> Vec<Line> {
    bands(mask)
        .into_iter()
        .filter_map(|(y0, y1)| line_in_band(mask, y0, y1))
        .collect()
}

fn bands(mask: &Mask) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = None;
    for y in 0..=mask.height {
        let on = y < mask.height
            && (0..mask.width).filter(|&x| mask.get(x, y)).count() >= ROW_MIN_PIXELS;
        match (on, start) {
            (true, None) => start = Some(y),
            (false, Some(s)) => {
                if y - s >= MIN_BAND {
                    out.push((s, y));
                }
                start = None;
            }
            _ => {}
        }
    }
    out
}

/// Features of the text between rows `y0..y1`, after dropping direction arrows.
pub fn line_in_band(mask: &Mask, y0: usize, y1: usize) -> Option<Line> {
    let h = y1 - y0;
    let cols: Vec<bool> = (0..mask.width)
        .map(|x| (y0..y1).any(|y| mask.get(x, y)))
        .collect();
    let mut words = words(&cols, 2.max(h / 4));
    let narrow = |w: &(usize, usize)| (w.1 - w.0) as f32 <= 0.8 * h as f32;
    if words.len() > 1 && narrow(&words[0]) {
        words.remove(0);
    }
    if words.len() > 1 && narrow(&words[words.len() - 1]) {
        words.pop();
    }
    let (x0, x1) = (words.first()?.0, words.last()?.1);
    Some(Line {
        y0,
        y1,
        x0,
        x1,
        aspect: (x1 - x0) as f32 / h as f32,
        profile: profile(mask, y0, y1, x0, x1),
    })
}

/// Column spans separated by more than `gap` empty columns.
fn words(cols: &[bool], gap: usize) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    for (x, _) in cols.iter().enumerate().filter(|(_, on)| **on) {
        match out.last_mut() {
            // Same word while the empty run since the previous text column is at most `gap`.
            Some(last) if x < last.1 + gap => last.1 = x + 1,
            _ => out.push((x, x + 1)),
        }
    }
    out
}

/// Column sums of the box, area-resampled to `PROFILE_BINS` and L2-normalised.
fn profile(mask: &Mask, y0: usize, y1: usize, x0: usize, x1: usize) -> [f32; PROFILE_BINS] {
    let width = x1 - x0;
    let mut cumulative = vec![0f64; width + 1];
    for x in 0..width {
        let column = (y0..y1).filter(|&y| mask.get(x0 + x, y)).count();
        cumulative[x + 1] = cumulative[x] + column as f64;
    }
    let at = |pos: f64| {
        let i = (pos.floor() as usize).min(width - 1);
        cumulative[i] + (pos - i as f64) * (cumulative[i + 1] - cumulative[i])
    };
    let mut out = [0f32; PROFILE_BINS];
    let step = width as f64 / PROFILE_BINS as f64;
    for (bin, value) in out.iter_mut().enumerate() {
        *value = (at((bin + 1) as f64 * step) - at(bin as f64 * step)) as f32;
    }
    let norm = out.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        out.iter_mut().for_each(|v| *v /= norm);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 60x24 BGRA image: a dark caption band with a white bar, above a grey "stone" area.
    fn image() -> Vec<u8> {
        let (w, h) = (60, 24);
        let mut data = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let v = if y < 12 {
                    if (2..10).contains(&y) && (10..40).contains(&x) {
                        255
                    } else {
                        30
                    }
                } else if x.is_multiple_of(3) {
                    200 // bright grey stone pixels...
                } else {
                    140 // ...broken up by mid-tones
                };
                data[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[v, v, v, 255]);
            }
        }
        data
    }

    #[test]
    fn text_inside_caption_background_is_kept_and_scene_texture_dropped() {
        let data = image();
        let mask = text_mask(&Bgra {
            width: 60,
            height: 24,
            stride: 240,
            data: &data,
        });
        assert!(mask.get(20, 5));
        assert!(!mask.get(5, 5)); // dark background
        assert!((12..24).all(|y| (0..60).all(|x| !mask.get(x, y)))); // stone
    }

    #[test]
    fn profile_is_scale_independent() {
        let make = |scale: usize| {
            let (w, h) = (10 * scale, 4 * scale);
            let mut bits = vec![false; w * h];
            for y in 0..h {
                for x in 0..w {
                    bits[y * w + x] = (x / scale).is_multiple_of(3) || y / scale == 0;
                }
            }
            line_in_band(
                &Mask {
                    width: w,
                    height: h,
                    bits,
                },
                0,
                h,
            )
            .unwrap()
        };
        let (a, b) = (make(2), make(4));
        assert!((a.aspect - b.aspect).abs() < 1e-6);
        let dot: f32 = a.profile.iter().zip(&b.profile).map(|(x, y)| x * y).sum();
        assert!(dot > 0.999, "{dot}");
    }
}
