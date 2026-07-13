//! Cross-platform desktop adapters that do not own QRForge lifecycle or UI.

mod browser;
mod clipboard;
mod clock;
mod hotkey;

pub use browser::SystemBrowser;
pub use clipboard::SystemClipboard;
pub use clock::SystemClock;
pub use hotkey::TauriHotkey;
