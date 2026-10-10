//! Windows.Graphics.Capture of the game window (ADR-005). Each frame is cropped to the
//! subtitle area and kept as the latest frame for the detectors; nothing is saved.
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use windows_capture::capture::{CaptureControl, Context, GraphicsCaptureApiHandler};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::settings::{
    ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
    MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
};
use windows_capture::window::Window;

use crate::GameWindow;

/// Subtitle area as fractions of the captured window (x0, y0, x1, y1). Covers the Java
/// subtitle box (bottom right) and Bedrock captions (right, middle) - Phase 0 recordings.
pub const SUBTITLE_AREA: (f32, f32, f32, f32) = (0.55, 0.45, 1.0, 1.0);

/// A cropped BGRA frame.
pub struct FrameData {
    /// Increases by one for every captured frame.
    pub seq: u64,
    pub width: usize,
    pub height: usize,
    /// Tightly packed BGRA rows (`width * 4` bytes each).
    pub bgra: Vec<u8>,
}

/// The newest frame, shared between the capture thread and the detectors.
pub type FrameSlot = Arc<Mutex<Option<Arc<FrameData>>>>;

struct Shared {
    frames: Arc<AtomicU64>,
    slot: FrameSlot,
}

struct Handler {
    shared: Shared,
    scratch: Vec<u8>,
}

impl GraphicsCaptureApiHandler for Handler {
    type Flags = Shared;
    type Error = String;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self {
            shared: ctx.flags,
            scratch: Vec::new(),
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        let seq = self.shared.frames.fetch_add(1, Ordering::Relaxed) + 1;
        let (w, h) = (frame.width() as f32, frame.height() as f32);
        let (fx0, fy0, fx1, fy1) = SUBTITLE_AREA;
        let (x0, y0) = ((w * fx0) as u32, (h * fy0) as u32);
        let (x1, y1) = ((w * fx1) as u32, (h * fy1) as u32);
        let Ok(buffer) = frame.buffer_crop(x0, y0, x1, y1) else {
            return Ok(()); // window too small for now; try the next frame
        };
        let data = FrameData {
            seq,
            width: (x1 - x0) as usize,
            height: (y1 - y0) as usize,
            bgra: buffer.as_nopadding_buffer(&mut self.scratch).to_vec(),
        };
        if let Ok(mut slot) = self.shared.slot.lock() {
            *slot = Some(Arc::new(data));
        }
        Ok(())
    }
}

/// A running capture session; stops when dropped.
pub struct Capture {
    control: Option<CaptureControl<Handler, String>>,
    frames: Arc<AtomicU64>,
}

impl Capture {
    /// Starts capturing `game`; every cropped frame replaces the content of `slot`.
    pub fn start(game: &GameWindow, slot: FrameSlot) -> Result<Self, String> {
        let frames = Arc::new(AtomicU64::new(0));
        // Hiding the yellow capture border is not supported on every Windows build.
        let mut last_error = String::new();
        for border in [
            DrawBorderSettings::WithoutBorder,
            DrawBorderSettings::Default,
        ] {
            let settings = Settings::new(
                Window::from_raw_hwnd(game.raw_hwnd()),
                CursorCaptureSettings::WithoutCursor,
                border,
                SecondaryWindowSettings::Default,
                MinimumUpdateIntervalSettings::Default,
                DirtyRegionSettings::Default,
                ColorFormat::Bgra8,
                Shared {
                    frames: frames.clone(),
                    slot: slot.clone(),
                },
            );
            match Handler::start_free_threaded(settings) {
                Ok(control) => {
                    return Ok(Self {
                        control: Some(control),
                        frames,
                    });
                }
                Err(e) => last_error = format!("{e:?}"),
            }
        }
        Err(last_error)
    }

    /// Total frames received so far.
    pub fn frames(&self) -> u64 {
        self.frames.load(Ordering::Relaxed)
    }

    /// False once the capture thread has ended, e.g. because the window closed.
    pub fn is_running(&self) -> bool {
        self.control.as_ref().is_some_and(|c| !c.is_finished())
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        if let Some(control) = self.control.take() {
            let _ = control.stop();
        }
    }
}
