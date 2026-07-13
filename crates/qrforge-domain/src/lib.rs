//! Pure QRForge domain types and safety rules.
//!
//! This crate has no desktop-framework or operating-system dependencies.

mod capture;
mod detection;
mod hotkey;
mod payload;
mod settings;

pub use capture::{CapturedFrame, FrameError, PixelFormat};
pub use detection::{Detection, Point, QrFormat};
pub use hotkey::{Hotkey, HotkeyKey, HotkeyParseError};
pub use payload::{PayloadClass, SafeHttpUrl, classify_payload};
pub use settings::{AppSettings, SETTINGS_SCHEMA_VERSION};
