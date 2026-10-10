//! Hands a URL, file or folder to Windows, exactly like double-clicking it: links open in
//! the default browser, a `.mcpack` opens in Minecraft, a folder opens in Explorer.
//! Callers pass only fixed, app-defined targets (never text from the UI).
use std::path::Path;

use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{HSTRING, PCWSTR, w};

pub fn open_url(url: &str) -> Result<(), String> {
    shell_open(&HSTRING::from(url))
}

pub fn open_path(path: &Path) -> Result<(), String> {
    shell_open(&HSTRING::from(path.as_os_str()))
}

fn shell_open(target: &HSTRING) -> Result<(), String> {
    // SAFETY: all strings are valid, NUL-terminated wide strings that outlive the call.
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            target,
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    // Values above 32 mean success (documented ShellExecute convention).
    if result.0 as usize > 32 {
        Ok(())
    } else {
        Err(format!(
            "Windows could not open it (code {})",
            result.0 as usize
        ))
    }
}
