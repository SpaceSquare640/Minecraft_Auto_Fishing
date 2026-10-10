use std::fmt;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_MOUSE, MOUSE_EVENT_FLAGS, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEINPUT, SendInput,
};

use crate::GameWindow;

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

/// Presses and releases the right mouse button once, only if `game` is in the foreground.
///
/// SendInput always goes to the foreground window, so the check right before sending is what
/// keeps clicks from landing in another program. Note: if the game runs as administrator,
/// Windows (UIPI) drops the events without reporting an error.
pub fn right_click(game: &GameWindow) -> Result<(), InputError> {
    if !game.is_foreground() {
        return Err(InputError::NotForeground);
    }
    let inputs = [mouse(MOUSEEVENTF_RIGHTDOWN), mouse(MOUSEEVENTF_RIGHTUP)];
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
