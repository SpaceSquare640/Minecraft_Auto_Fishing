//! Windows.Media.Ocr (built into Windows; no extra download besides the language pack the
//! player already has). Used for the vanilla Java bite caption (scheme A).
use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;
use windows::core::HSTRING;

pub struct Ocr {
    engine: OcrEngine,
    /// BCP-47 tag of the language actually used, e.g. "zh-Hant-TW".
    pub language: String,
}

impl Ocr {
    /// An engine for `tag` (BCP-47), or `None` when that OCR language is not installed.
    pub fn for_language(tag: &str) -> Option<Self> {
        let language = Language::CreateLanguage(&HSTRING::from(tag)).ok()?;
        if !OcrEngine::IsLanguageSupported(&language).unwrap_or(false) {
            return None;
        }
        let engine = OcrEngine::TryCreateFromLanguage(&language).ok()?;
        Some(Self {
            engine,
            language: tag.to_owned(),
        })
    }

    /// BCP-47 tags of every OCR language installed on this PC.
    pub fn installed_languages() -> Vec<String> {
        let Ok(list) = OcrEngine::AvailableRecognizerLanguages() else {
            return Vec::new();
        };
        (0..list.Size().unwrap_or(0))
            .filter_map(|i| {
                list.GetAt(i)
                    .ok()?
                    .LanguageTag()
                    .ok()
                    .map(|t| t.to_string())
            })
            .collect()
    }

    /// Text lines in a tightly packed BGRA image.
    pub fn recognize(
        &self,
        width: usize,
        height: usize,
        bgra: &[u8],
    ) -> Result<Vec<String>, String> {
        let err = |e: windows::core::Error| e.message();
        let writer = DataWriter::new().map_err(err)?;
        writer.WriteBytes(bgra).map_err(err)?;
        let buffer = writer.DetachBuffer().map_err(err)?;
        let bitmap = SoftwareBitmap::CreateCopyFromBuffer(
            &buffer,
            BitmapPixelFormat::Bgra8,
            width as i32,
            height as i32,
        )
        .map_err(err)?;
        let result = self
            .engine
            .RecognizeAsync(&bitmap)
            .map_err(err)?
            .join()
            .map_err(err)?;
        let lines = result.Lines().map_err(err)?;
        Ok((0..lines.Size().map_err(err)?)
            .filter_map(|i| lines.GetAt(i).ok()?.Text().ok().map(|t| t.to_string()))
            .collect())
    }
}
