//! QRForge use cases and replaceable port contracts.

mod ports;
mod results;
mod scan;
mod settings;

pub use ports::{
    BrowserPort, CaptureOutput, CapturePort, ClipboardPort, ClockPort, DecoderPort, HotkeyPort,
    Notification, NotificationPort, PortError, SettingsRepository, StartupPort,
};
pub use results::{
    ClassifiedResult, PendingResultsView, ResultActionError, ResultActionKind, ResultActionOutcome,
    ResultActionRequest, ResultItemView, ResultKind, ResultService,
};
pub use scan::{
    CaptureMetadata, FailureStage, ScanMetrics, ScanOutcome, ScanPorts, ScanReport, ScanService,
};
pub use settings::{
    SettingsError, SettingsService, SettingsSnapshot, SettingsState, SettingsUpdate,
};
