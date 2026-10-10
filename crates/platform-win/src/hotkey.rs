//! Global hotkey through `RegisterHotKey`: Windows reports only this key combination,
//! so the app never sees any other keystrokes (no low-level keyboard hook, ADR-007).
use std::sync::mpsc;
use std::thread;

use windows::Win32::UI::Input::KeyboardAndMouse::{MOD_NOREPEAT, RegisterHotKey, VK_F8};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

/// Shown to the player.
pub const KEY_NAME: &str = "F8";
const HOTKEY_ID: i32 = 0x4D41; // "MA"

/// Registers the hotkey on a dedicated thread and calls `on_press` for every press.
/// Fails if another program already owns the key.
pub fn spawn<F: Fn() + Send + 'static>(on_press: F) -> Result<(), String> {
    let (ready, registered) = mpsc::channel();
    thread::Builder::new()
        .name("maf-hotkey".into())
        .spawn(move || {
            // SAFETY: no window handle: the hotkey is bound to this thread's message queue.
            let result =
                unsafe { RegisterHotKey(None, HOTKEY_ID, MOD_NOREPEAT, u32::from(VK_F8.0)) };
            let ok = result.is_ok();
            let _ = ready.send(result.map_err(|e| e.message()));
            if !ok {
                return;
            }
            let mut msg = MSG::default();
            // GetMessageW returns 0 on WM_QUIT and -1 on error; stop on both.
            // SAFETY: `msg` is a valid out-pointer; None reads every message of this thread.
            while unsafe { GetMessageW(&raw mut msg, None, 0, 0) }.0 > 0 {
                if msg.message == WM_HOTKEY && msg.wParam.0 == HOTKEY_ID as usize {
                    on_press();
                }
            }
        })
        .map_err(|e| e.to_string())?;
    registered.recv().map_err(|e| e.to_string())?
}
