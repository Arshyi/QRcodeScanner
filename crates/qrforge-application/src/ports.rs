use qrforge_domain::{AppSettings, CapturedFrame, Detection, Hotkey, SafeHttpUrl};
use std::{fmt, time::Duration};

/// A sanitized adapter failure. Payload content must never be placed in this value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortError {
    operation: &'static str,
    detail: String,
}

impl PortError {
    /// Creates an adapter error with a non-sensitive operation and detail.
    #[must_use]
    pub fn new(operation: &'static str, detail: impl Into<String>) -> Self {
        Self {
            operation,
            detail: detail.into(),
        }
    }

    /// Returns the stable operation identifier.
    #[must_use]
    pub const fn operation(&self) -> &'static str {
        self.operation
    }
}

impl fmt::Display for PortError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} failed: {}", self.operation, self.detail)
    }
}

impl std::error::Error for PortError {}

/// Captures the currently visible primary screen into memory.
pub trait CapturePort: Send + Sync {
    /// Performs exactly one capture.
    fn capture_primary(&self) -> Result<CapturedFrame, PortError>;
}

/// Decodes QR-family symbols from a borrowed in-memory frame.
pub trait DecoderPort: Send + Sync {
    /// Returns every valid QR-family detection found in the frame.
    fn decode(&self, frame: &CapturedFrame) -> Result<Vec<Detection>, PortError>;
}

/// Opens only a domain-validated HTTP(S) URL.
pub trait BrowserPort: Send + Sync {
    /// Delegates a safe URL to the operating system's default browser.
    fn open(&self, url: &SafeHttpUrl) -> Result<(), PortError>;
}

/// Writes user-approved text to the system clipboard.
pub trait ClipboardPort: Send + Sync {
    /// Replaces clipboard text without interpreting it.
    fn set_text(&self, text: &str) -> Result<(), PortError>;
}

/// Non-sensitive user feedback emitted by application policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Notification {
    /// One safe URL was opened.
    QrOpened,
    /// Plain text was copied.
    TextCopied,
    /// A blocked URL-like payload was copied but not opened.
    BlockedPayloadCopied,
    /// No QR-family symbol was found.
    NoQrFound,
    /// More than one symbol was found and automatic action was suppressed.
    MultipleQrFound(usize),
    /// A previous scan still owns the single-scan lock.
    ScanAlreadyInProgress,
    /// The configured global hotkey could not be registered.
    HotkeyConflict,
    /// A binary or otherwise unsupported payload was detected.
    UnsupportedPayload,
    /// A safe URL was detected while automatic opening was disabled.
    SafeUrlNotOpened,
    /// Capture, decode, browser, or clipboard work failed.
    ScanFailed,
}

/// Delivers native or tray feedback without including QR payloads.
pub trait NotificationPort: Send + Sync {
    /// Shows one non-sensitive feedback event.
    fn notify(&self, notification: Notification) -> Result<(), PortError>;
}

/// Monotonic clock used for metrics and deterministic tests.
pub trait ClockPort: Send + Sync {
    /// Returns elapsed monotonic time since an adapter-defined origin.
    fn now(&self) -> Duration;
}

/// Persists the complete validated settings document atomically.
pub trait SettingsRepository: Send + Sync {
    /// Loads settings with schema migration and per-field fallback.
    fn load(&self) -> Result<AppSettings, PortError>;
    /// Atomically replaces the stored document.
    fn save(&self, settings: &AppSettings) -> Result<(), PortError>;
}

/// Owns the active operating-system global hotkey registration.
pub trait HotkeyPort: Send + Sync {
    /// Returns the currently registered hotkey, if registration succeeded.
    fn active(&self) -> Option<Hotkey>;
    /// Replaces the registration, restoring the previous hotkey on failure.
    fn replace(&self, requested: &Hotkey) -> Result<(), PortError>;
}

/// Controls launch-at-sign-in behavior.
pub trait StartupPort: Send + Sync {
    /// Returns the current operating-system registration state.
    fn is_enabled(&self) -> Result<bool, PortError>;
    /// Enables or disables launch at sign-in.
    fn set_enabled(&self, enabled: bool) -> Result<(), PortError>;
}
