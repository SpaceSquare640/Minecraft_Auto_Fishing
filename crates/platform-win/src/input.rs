use std::fmt;
use std::thread;
use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_MOUSE, MOUSE_EVENT_FLAGS, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEINPUT, SendInput,
};

use crate::GameWindow;

/// How long the button stays down. Bedrock reads the mouse once per frame, so a press and
/// release sent together can fall into the same frame and be missed.
const HOLD: Duration = Duration::from_millis(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputError {
    /// The game window is not in the foreground, so nothing was sent.
    NotForeground,
    /// Windows accepted fewer events than requested.
    Blocked,
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotForeground => "the game window is not in the foreground",
            Self::Blocked => "Windows blocked the input",
        })
    }
}

impl std::error::Error for InputError {}

/// Presses the right mouse button, holds it for [`HOLD`] and releases it, only if `game` is in
/// the foreground. Blocks the calling thread for the hold.
///
/// SendInput always goes to the foreground window, so the check right before sending is what
/// keeps clicks from landing in another program. Note: if the game runs as administrator,
/// Windows (UIPI) drops the events without reporting an error.
pub fn right_click(game: &GameWindow) -> Result<(), InputError> {
    if !game.is_foreground() {
        return Err(InputError::NotForeground);
    }
    send(MOUSEEVENTF_RIGHTDOWN)?;
    thread::sleep(HOLD);
    // Release even if the focus moved during the hold, so the button is never left down.
    send(MOUSEEVENTF_RIGHTUP)
}

/// Sends only a right-button release, only if `game` is in the foreground. Using an item
/// happens on press, so a lone release does nothing in the game. Bedrock drops the first mouse
/// button event after its window gets the focus back; this release is meant to be that event.
pub fn release_right(game: &GameWindow) -> Result<(), InputError> {
    if !game.is_foreground() {
        return Err(InputError::NotForeground);
    }
    send(MOUSEEVENTF_RIGHTUP)
}

fn send(flags: MOUSE_EVENT_FLAGS) -> Result<(), InputError> {
    let inputs = [mouse(flags)];
    // SAFETY: `inputs` is a valid slice and cbsize is the size of one INPUT, as required.
    let sent = unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
    if sent as usize == inputs.len() {
        Ok(())
    } else {
        Err(InputError::Blocked)
    }
}

fn mouse(flags: MOUSE_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
