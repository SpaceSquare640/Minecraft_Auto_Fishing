//! Windows platform layer: the only crate that calls Windows APIs (ADR-005 to ADR-007).
//!
//! - [`find_game_window`]: locate a Minecraft window by the process that owns it
//! - [`right_click`]: one right click, sent only while the game window is in the foreground
//! - [`hotkey::spawn`]: global start/stop hotkey via `RegisterHotKey` (no keyboard hook)
//! - [`capture::Capture`]: Windows.Graphics.Capture of the game window
#![cfg(windows)]
#![warn(clippy::undocumented_unsafe_blocks)]

pub mod capture;
pub mod hotkey;
mod input;
mod window;

pub use input::{InputError, right_click};
pub use window::{Edition, GameWindow, find_game_window};
