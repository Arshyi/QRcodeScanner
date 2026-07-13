use crate::diagnostics::Diagnostics;
use qrforge_application::{NotificationPort, ScanService, SettingsService};
use std::{sync::Arc, time::Instant};

/// Application services retained by the native tray host.
pub struct RuntimeState {
    /// One-shot scan use case.
    pub scan: Arc<ScanService>,
    /// Transactional settings use case.
    pub settings: Arc<SettingsService>,
    /// Native/tray feedback adapter.
    pub notifications: Arc<dyn NotificationPort>,
    /// Opt-in local diagnostics recorder.
    pub diagnostics: Arc<Diagnostics>,
}

/// Dispatches capture and decode away from the Tauri and hotkey callback thread.
pub fn spawn_scan(scan: Arc<ScanService>, diagnostics: Arc<Diagnostics>, trigger: &'static str) {
    let triggered = Instant::now();
    std::thread::spawn(move || {
        let report = scan.scan();
        diagnostics.record_scan(trigger, triggered.elapsed(), &report);
    });
}
