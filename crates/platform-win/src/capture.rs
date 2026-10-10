//! Windows.Graphics.Capture of the game window (ADR-005). P1.2 only counts frames to show
//! that capture works; P1.3 adds the subtitle-area crop for the bite detectors.
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use windows_capture::capture::{CaptureControl, Context, GraphicsCaptureApiHandler};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::settings::{
    ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
    MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
};
use windows_capture::window::Window;

use crate::GameWindow;

struct FrameCounter {
    frames: Arc<AtomicU64>,
}

impl GraphicsCaptureApiHandler for FrameCounter {
    type Flags = Arc<AtomicU64>;
    type Error = String;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self { frames: ctx.flags })
    }

    fn on_frame_arrived(
        &mut self,
        _frame: &mut Frame,
        _control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        self.frames.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

/// A running capture session; stops when dropped.
pub struct Capture {
    control: Option<CaptureControl<FrameCounter, String>>,
    frames: Arc<AtomicU64>,
}

impl Capture {
    pub fn start(game: &GameWindow) -> Result<Self, String> {
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
                frames.clone(),
            );
            match FrameCounter::start_free_threaded(settings) {
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
