//! Whether the mouse cursor is shown. Minecraft hides it while the player is in the game and
//! shows it whenever a menu (pause, inventory, chat) is open, so a visible cursor means a click
//! would land in a menu.
use windows::Win32::UI::WindowsAndMessaging::{CURSOR_SHOWING, CURSORINFO, GetCursorInfo};

/// True when the cursor is visible, or when it cannot be read (then clicking is not safe).
///
/// A game can hide the cursor by lowering the show count or by setting no cursor image, so
/// both are checked.
pub fn cursor_visible() -> bool {
    let mut info = CURSORINFO {
        cbSize: size_of::<CURSORINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: `info` is a valid CURSORINFO with cbSize set, as GetCursorInfo requires.
    if unsafe { GetCursorInfo(&mut info) }.is_err() {
        return true;
    }
    info.flags.0 & CURSOR_SHOWING.0 != 0 && !info.hCursor.is_invalid()
}
