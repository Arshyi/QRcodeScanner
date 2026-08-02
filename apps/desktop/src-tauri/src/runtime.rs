use crate::{diagnostics::Diagnostics, tray, window};
use qrforge_application::{
    CapturePort, NotificationPort, ResultService, ScanService, SettingsService,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};
use tauri::{AppHandle, Wry};

/// Application services retained by the native tray host.
pub struct RuntimeState {
    /// Bounded one-shot scan dispatcher.
    pub scans: Arc<ScanDispatcher>,
    /// Transactional settings use case.
    pub settings: Arc<SettingsService>,
    /// Current physical display enumeration and capture adapter.
    pub capture: Arc<dyn CapturePort>,
    /// Pending multi-code result policy and native actions.
    pub results: Arc<ResultService>,
    /// Native/tray feedback adapter.
    pub notifications: Arc<dyn NotificationPort>,
    /// Opt-in local diagnostics recorder used by window lifecycle events.
    pub diagnostics: Arc<Diagnostics>,
}

/// Ensures at most one native scan worker exists at a time.
pub struct ScanDispatcher {
    scan: Arc<ScanService>,
    diagnostics: Arc<Diagnostics>,
    results: Arc<ResultService>,
    app: AppHandle<Wry>,
    worker: WorkerGate,
}

impl ScanDispatcher {
    /// Creates a dispatcher for a one-shot scan service.
    #[must_use]
    pub fn new(
        scan: Arc<ScanService>,
        diagnostics: Arc<Diagnostics>,
        results: Arc<ResultService>,
        app: AppHandle<Wry>,
    ) -> Self {
        Self {
            scan,
            diagnostics,
            results,
            app,
            worker: WorkerGate::default(),
        }
    }

    /// Dispatches capture and decode away from the Tauri and hotkey callback thread.
    ///
    /// Duplicate activation is rejected without creating another OS thread.
    pub fn spawn(self: &Arc<Self>, trigger: &'static str) {
        let triggered = Instant::now();
        let _ = tray::show_scan_started(&self.app);
        let Some(permit) = self.worker.try_acquire() else {
            let report = self.scan.already_in_progress();
            self.diagnostics
                .record_scan(trigger, triggered.elapsed(), &report);
            return;
        };
        let dispatcher = self.clone();
        std::thread::spawn(move || {
            let _permit = permit;
            let mut report = dispatcher.scan.scan();
            if !report.result_items.is_empty() {
                let pending = dispatcher
                    .results
                    .publish(std::mem::take(&mut report.result_items));
                clear_results_if_chooser_unavailable(
                    &dispatcher.results,
                    pending.session_id,
                    window::open_results(&dispatcher.app).is_ok(),
                );
            }
            dispatcher
                .diagnostics
                .record_scan(trigger, triggered.elapsed(), &report);
        });
    }
}

fn clear_results_if_chooser_unavailable(
    results: &ResultService,
    session_id: u64,
    chooser_available: bool,
) {
    if !chooser_available {
        results.clear(Some(session_id));
    }
}

#[derive(Default)]
struct WorkerGate(Arc<AtomicBool>);

impl WorkerGate {
    fn try_acquire(&self) -> Option<WorkerPermit> {
        self.0
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()
            .map(|_| WorkerPermit(self.0.clone()))
    }
}

struct WorkerPermit(Arc<AtomicBool>);

impl Drop for WorkerPermit {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::{WorkerGate, clear_results_if_chooser_unavailable};
    use qrforge_application::{BrowserPort, ClipboardPort, PortError, ResultService};
    use qrforge_domain::SafeHttpUrl;
    use std::sync::Arc;

    struct UnusedSystemPort;

    impl BrowserPort for UnusedSystemPort {
        fn open(&self, _url: &SafeHttpUrl) -> Result<(), PortError> {
            unreachable!("lifecycle test does not open URLs")
        }
    }

    impl ClipboardPort for UnusedSystemPort {
        fn set_text(&self, _text: &str) -> Result<(), PortError> {
            unreachable!("lifecycle test does not use the clipboard")
        }
    }

    #[test]
    fn worker_gate_never_admits_overlapping_workers() {
        let gate = WorkerGate::default();
        let first = gate.try_acquire().expect("first worker should start");
        assert!(gate.try_acquire().is_none());
        drop(first);
        assert!(gate.try_acquire().is_some());
    }

    #[test]
    fn chooser_failure_discards_only_its_pending_result_session() {
        let port = Arc::new(UnusedSystemPort);
        let results = ResultService::new(port.clone(), port);
        let first = results.publish(Vec::new());
        let replacement = results.publish(Vec::new());

        clear_results_if_chooser_unavailable(&results, first.session_id, false);
        assert_eq!(
            results.snapshot().map(|pending| pending.session_id),
            Some(replacement.session_id)
        );

        clear_results_if_chooser_unavailable(&results, replacement.session_id, false);
        assert!(results.snapshot().is_none());
    }
}
