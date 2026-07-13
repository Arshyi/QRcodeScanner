use qrforge_application::{ClipboardPort, PortError};

/// Native text clipboard adapter.
#[derive(Default)]
pub struct SystemClipboard;

impl ClipboardPort for SystemClipboard {
    fn set_text(&self, text: &str) -> Result<(), PortError> {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|error| PortError::new("clipboard_open", error.to_string()))?;
        clipboard
            .set_text(text.to_owned())
            .map_err(|error| PortError::new("clipboard_write", error.to_string()))
    }
}
