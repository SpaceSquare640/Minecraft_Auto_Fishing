use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow,
    IsWindowVisible,
};
use windows::core::{BOOL, PWSTR};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edition {
    Java,
    Bedrock,
}

impl Edition {
    /// Which edition a process executable file name belongs to (case-insensitive).
    pub fn from_exe(file_name: &str) -> Option<Self> {
        match file_name.to_ascii_lowercase().as_str() {
            "javaw.exe" | "java.exe" => Some(Self::Java),
            "minecraft.windows.exe" => Some(Self::Bedrock),
            _ => None,
        }
    }
}

/// A visible top-level window owned by a Minecraft process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameWindow {
    /// Stored as an integer so the struct can move between threads.
    hwnd: isize,
    pub edition: Edition,
    pub title: String,
    pub pid: u32,
}

impl GameWindow {
    fn hwnd(&self) -> HWND {
        HWND(self.hwnd as *mut core::ffi::c_void)
    }

    pub fn raw_hwnd(&self) -> *mut core::ffi::c_void {
        self.hwnd as *mut core::ffi::c_void
    }

    /// False once the window has been closed.
    pub fn is_alive(&self) -> bool {
        // SAFETY: IsWindow accepts any handle value, including stale ones.
        unsafe { IsWindow(Some(self.hwnd())) }.as_bool()
    }

    /// True when the game window has keyboard focus and is not minimised.
    pub fn is_foreground(&self) -> bool {
        // SAFETY: both calls only read window-manager state; stale handles are allowed.
        unsafe { GetForegroundWindow() == self.hwnd() && !IsIconic(self.hwnd()).as_bool() }
    }
}

/// Finds a Minecraft window. When several are open, the one in the foreground wins.
pub fn find_game_window() -> Option<GameWindow> {
    let mut found: Vec<GameWindow> = Vec::new();
    // SAFETY: the callback only runs during this call, while `found` is alive and
    // exclusively borrowed through the pointer passed as LPARAM.
    let _ = unsafe { EnumWindows(Some(collect), LPARAM(&raw mut found as isize)) };
    let foreground = found.iter().position(GameWindow::is_foreground);
    match foreground {
        Some(i) => Some(found.swap_remove(i)),
        None => found.into_iter().next(),
    }
}

unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // SAFETY: lparam is the `&mut Vec<GameWindow>` created in find_game_window.
    let found = unsafe { &mut *(lparam.0 as *mut Vec<GameWindow>) };
    if let Some(window) = describe(hwnd) {
        found.push(window);
    }
    true.into()
}

fn describe(hwnd: HWND) -> Option<GameWindow> {
    // SAFETY: read-only queries on a handle supplied by EnumWindows.
    if !unsafe { IsWindowVisible(hwnd) }.as_bool() {
        return None;
    }
    let mut text = [0u16; 256];
    // SAFETY: the buffer is valid for its full length.
    let len = unsafe { GetWindowTextW(hwnd, &mut text) };
    let title = String::from_utf16_lossy(&text[..usize::try_from(len).unwrap_or(0)]);
    if !title.starts_with("Minecraft") {
        return None;
    }
    let mut pid = 0u32;
    // SAFETY: pid is a valid out-pointer for the duration of the call.
    unsafe { GetWindowThreadProcessId(hwnd, Some(&raw mut pid)) };
    let edition = Edition::from_exe(&exe_file_name(pid)?)?;
    Some(GameWindow {
        hwnd: hwnd.0 as isize,
        edition,
        title,
        pid,
    })
}

/// File name of a process executable, e.g. "javaw.exe".
fn exe_file_name(pid: u32) -> Option<String> {
    // SAFETY: limited query rights only; the handle is closed below.
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let mut buffer = [0u16; 1024];
    let mut size = buffer.len() as u32;
    // SAFETY: buffer and size describe the same valid allocation.
    let queried = unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &raw mut size,
        )
    };
    // SAFETY: the handle came from OpenProcess above and is not used afterwards.
    let _ = unsafe { CloseHandle(process) };
    queried.ok()?;
    let path = String::from_utf16_lossy(&buffer[..size as usize]);
    path.rsplit(['\\', '/']).next().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executables_map_to_editions() {
        assert_eq!(Edition::from_exe("javaw.exe"), Some(Edition::Java));
        assert_eq!(Edition::from_exe("JAVA.EXE"), Some(Edition::Java));
        assert_eq!(
            Edition::from_exe("Minecraft.Windows.exe"),
            Some(Edition::Bedrock)
        );
        assert_eq!(Edition::from_exe("MinecraftLauncher.exe"), None);
        assert_eq!(Edition::from_exe("chrome.exe"), None);
    }
}
