//! QRForge use cases and replaceable port contracts.

mod ports;
mod scan;
mod settings;

pub use ports::{
    BrowserPort, CapturePort, ClipboardPort, ClockPort, DecoderPort, HotkeyPort, Notification,
    NotificationPort, PortError, SettingsRepository, StartupPort,
};
pub use scan::{
    CaptureMetadata, FailureStage, ScanMetrics, ScanOutcome, ScanPorts, ScanReport, ScanService,
};
pub use settings::{
    SettingsError, SettingsService, SettingsSnapshot, SettingsState, SettingsUpdate,
};
