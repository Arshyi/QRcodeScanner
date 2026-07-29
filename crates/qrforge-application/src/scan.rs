use crate::{
    BrowserPort, CapturePort, ClassifiedResult, ClipboardPort, ClockPort, DecoderPort,
    Notification, NotificationPort, SettingsState, results::classify_results,
};
use qrforge_domain::{Detection, PayloadClass, classify_payload};
use serde::Serialize;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

/// Concrete port set required by the one-shot scan use case.
pub struct ScanPorts {
    /// Screen capture adapter.
    pub capture: Arc<dyn CapturePort>,
    /// Offline QR decoder adapter.
    pub decoder: Arc<dyn DecoderPort>,
    /// Default-browser adapter.
    pub browser: Arc<dyn BrowserPort>,
    /// Text clipboard adapter.
    pub clipboard: Arc<dyn ClipboardPort>,
    /// Native/tray feedback adapter.
    pub notifications: Arc<dyn NotificationPort>,
    /// Monotonic clock adapter.
    pub clock: Arc<dyn ClockPort>,
}

/// Stage at which a one-shot scan failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureStage {
    /// Screen capture.
    Capture,
    /// QR decoding.
    Decode,
    /// Default browser launch.
    Browser,
    /// Clipboard update.
    Clipboard,
}

/// Non-sensitive result of a scan.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScanOutcome {
    /// A scan request was rejected because one is already active.
    AlreadyInProgress,
    /// No QR-family symbol was found.
    NoCode,
    /// Multiple symbols were found and no automatic action occurred.
    MultipleCodes {
        /// Number of valid detections.
        count: usize,
    },
    /// One validated HTTP(S) URL was opened.
    UrlOpened,
    /// One safe URL was found, but automatic opening is disabled.
    UrlDetected,
    /// One plain-text payload was copied.
    TextCopied,
    /// One URL-like payload with a blocked scheme was copied but not opened.
    BlockedPayloadCopied,
    /// A blocked URL-like payload was detected but copying is disabled.
    BlockedPayloadDetected,
    /// Malformed HTTP-like text was copied but never opened.
    MalformedTextCopied,
    /// Malformed HTTP-like text was detected but copying is disabled.
    MalformedTextDetected,
    /// Binary or disabled-copy content was left untouched.
    UnsupportedPayload,
    /// An adapter failed.
    Failed {
        /// Non-sensitive failing stage.
        stage: FailureStage,
    },
}

/// Timing information for a completed scan request.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanMetrics {
    /// Capture duration in milliseconds.
    pub capture_ms: u64,
    /// Decoder duration in milliseconds.
    pub decode_ms: u64,
    /// Total use-case duration in milliseconds.
    pub total_ms: u64,
    /// Count of valid QR-family detections.
    pub detection_count: usize,
}

/// Outcome and measurements for diagnostics that never include payload bytes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    /// Policy outcome.
    pub outcome: ScanOutcome,
    /// Non-sensitive timing data.
    pub metrics: ScanMetrics,
    /// Optional capture metadata for diagnostics (monitor, scaling).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_metadata: Option<CaptureMetadata>,
    /// Native-only result data published to the chooser for multi-code scans.
    #[serde(skip)]
    pub result_items: Vec<ClassifiedResult>,
}

/// Lightweight capture metadata for diagnostics (no pixel data).
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureMetadata {
    /// Human-readable monitor identifier.
    pub monitor_label: Option<String>,
    /// Windows display scaling factor as a percentage (e.g. 100, 125, 200).
    pub scale_factor_percent: Option<u32>,
    /// Captured image width in physical pixels.
    pub pixel_width: u32,
    /// Captured image height in physical pixels.
    pub pixel_height: u32,
}

/// Thread-safe one-shot capture, decode, and payload-policy orchestrator.
pub struct ScanService {
    ports: ScanPorts,
    settings: Arc<SettingsState>,
    in_progress: AtomicBool,
}

impl ScanService {
    /// Creates a scan service with replaceable adapters.
    #[must_use]
    pub fn new(ports: ScanPorts, settings: Arc<SettingsState>) -> Self {
        Self {
            ports,
            settings,
            in_progress: AtomicBool::new(false),
        }
    }

    /// Executes a single scan. Callers should invoke this off UI and hotkey threads.
    pub fn scan(&self) -> ScanReport {
        if self
            .in_progress
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return self.already_in_progress();
        }
        let _lease = ScanLease(&self.in_progress);
        let started = self.ports.clock.now();

        let capture_started = self.ports.clock.now();
        let requested_monitor = self.settings.get().scan_monitor_id;
        let Ok(capture) = self.ports.capture.capture(requested_monitor.as_ref()) else {
            return self.failed(started, FailureStage::Capture);
        };
        let capture_ms = elapsed_ms(capture_started, self.ports.clock.now());
        if capture.used_fallback {
            self.feedback(Notification::SelectedMonitorUnavailable);
        }

        let frame = capture.frame;
        let capture_metadata = Some(CaptureMetadata {
            monitor_label: Some(capture.monitor.label),
            scale_factor_percent: Some(capture.monitor.scale_factor_percent),
            pixel_width: frame.width(),
            pixel_height: frame.height(),
        });

        let decode_started = self.ports.clock.now();
        let Ok(detections) = self.ports.decoder.decode(&frame) else {
            return self.failed(started, FailureStage::Decode);
        };
        let decode_ms = elapsed_ms(decode_started, self.ports.clock.now());
        let detection_count = detections.len();
        let (outcome, result_items) = self.apply_policy(&detections);

        ScanReport {
            outcome,
            metrics: ScanMetrics {
                capture_ms,
                decode_ms,
                total_ms: elapsed_ms(started, self.ports.clock.now()),
                detection_count,
            },
            capture_metadata,
            result_items,
        }
    }

    /// Reports a duplicate activation rejected before a worker was created.
    #[must_use]
    pub fn already_in_progress(&self) -> ScanReport {
        self.feedback(Notification::ScanAlreadyInProgress);
        ScanReport {
            outcome: ScanOutcome::AlreadyInProgress,
            metrics: ScanMetrics::default(),
            capture_metadata: None,
            result_items: Vec::new(),
        }
    }

    fn apply_policy(&self, detections: &[Detection]) -> (ScanOutcome, Vec<ClassifiedResult>) {
        match detections {
            [] => {
                self.feedback(Notification::NoQrFound);
                (ScanOutcome::NoCode, Vec::new())
            }
            [detection] => (self.apply_single_policy(detection), Vec::new()),
            many => {
                self.feedback(Notification::MultipleQrFound(many.len()));
                (
                    ScanOutcome::MultipleCodes { count: many.len() },
                    classify_results(many),
                )
            }
        }
    }

    fn apply_single_policy(&self, detection: &Detection) -> ScanOutcome {
        let settings = self.settings.get();
        match classify_payload(&detection.raw_bytes) {
            PayloadClass::SafeUrl(url) if settings.auto_open_safe_urls => {
                if self.ports.browser.open(&url).is_err() {
                    self.feedback(Notification::BrowserFailed);
                    return ScanOutcome::Failed {
                        stage: FailureStage::Browser,
                    };
                }
                self.feedback(Notification::QrOpened);
                ScanOutcome::UrlOpened
            }
            PayloadClass::SafeUrl(_) => {
                self.feedback(Notification::SafeUrlNotOpened);
                ScanOutcome::UrlDetected
            }
            PayloadClass::PlainText(text) if settings.copy_non_url_payloads => {
                if self.ports.clipboard.set_text(&text).is_err() {
                    self.feedback(Notification::ClipboardFailed);
                    return ScanOutcome::Failed {
                        stage: FailureStage::Clipboard,
                    };
                }
                self.feedback(Notification::TextCopied);
                ScanOutcome::TextCopied
            }
            PayloadClass::MalformedUrl { text } if settings.copy_non_url_payloads => {
                if self.ports.clipboard.set_text(&text).is_err() {
                    self.feedback(Notification::ClipboardFailed);
                    return ScanOutcome::Failed {
                        stage: FailureStage::Clipboard,
                    };
                }
                self.feedback(Notification::MalformedPayloadCopied);
                ScanOutcome::MalformedTextCopied
            }
            PayloadClass::BlockedScheme { text, .. } | PayloadClass::BlockedUrl { text }
                if settings.copy_non_url_payloads =>
            {
                if self.ports.clipboard.set_text(&text).is_err() {
                    self.feedback(Notification::ClipboardFailed);
                    return ScanOutcome::Failed {
                        stage: FailureStage::Clipboard,
                    };
                }
                self.feedback(Notification::BlockedPayloadCopied);
                ScanOutcome::BlockedPayloadCopied
            }
            PayloadClass::MalformedUrl { .. } => {
                self.feedback(Notification::MalformedPayloadDetected);
                ScanOutcome::MalformedTextDetected
            }
            PayloadClass::BlockedScheme { .. } | PayloadClass::BlockedUrl { .. } => {
                self.feedback(Notification::BlockedPayloadDetected);
                ScanOutcome::BlockedPayloadDetected
            }
            PayloadClass::UnsafeText | PayloadClass::PlainText(_) | PayloadClass::Binary => {
                self.feedback(Notification::UnsupportedPayload);
                ScanOutcome::UnsupportedPayload
            }
        }
    }

    fn failed(&self, started: Duration, stage: FailureStage) -> ScanReport {
        self.feedback(match stage {
            FailureStage::Capture => Notification::CaptureFailed,
            FailureStage::Decode => Notification::DecoderFailed,
            FailureStage::Browser => Notification::BrowserFailed,
            FailureStage::Clipboard => Notification::ClipboardFailed,
        });
        ScanReport {
            outcome: ScanOutcome::Failed { stage },
            metrics: ScanMetrics {
                total_ms: elapsed_ms(started, self.ports.clock.now()),
                ..ScanMetrics::default()
            },
            capture_metadata: None,
            result_items: Vec::new(),
        }
    }

    fn feedback(&self, notification: Notification) {
        if self.settings.get().notifications_enabled {
            let _ = self.ports.notifications.notify(notification);
        }
    }
}

struct ScanLease<'a>(&'a AtomicBool);

impl Drop for ScanLease<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

fn elapsed_ms(start: Duration, end: Duration) -> u64 {
    u64::try_from(end.saturating_sub(start).as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CaptureOutput;
    use crate::{PortError, SettingsState};
    use qrforge_domain::{
        AppSettings, CapturedFrame, MonitorId, MonitorInfo, Point, QrFormat, SafeHttpUrl,
    };
    use std::sync::{Barrier, Mutex, mpsc};

    #[derive(Default)]
    struct Calls {
        browser: Mutex<Vec<String>>,
        clipboard: Mutex<Vec<String>>,
        notifications: Mutex<Vec<Notification>>,
    }

    struct FixedCapture;

    impl CapturePort for FixedCapture {
        fn monitors(&self) -> Result<Vec<MonitorInfo>, PortError> {
            Ok(vec![test_monitor()])
        }

        fn capture(&self, _requested: Option<&MonitorId>) -> Result<CaptureOutput, PortError> {
            Ok(CaptureOutput {
                frame: CapturedFrame::rgba8(1, 1, vec![255; 4])
                    .map_err(|error| PortError::new("capture", error.to_string()))?,
                monitor: test_monitor(),
                used_fallback: false,
            })
        }
    }

    fn test_monitor() -> MonitorInfo {
        MonitorInfo::new(
            "display-test".parse().expect("monitor id"),
            "Test display".to_owned(),
            0,
            0,
            1,
            1,
            100,
            0,
            true,
        )
        .expect("monitor")
    }

    struct FixedDecoder(Vec<Detection>);

    impl DecoderPort for FixedDecoder {
        fn decode(&self, _frame: &CapturedFrame) -> Result<Vec<Detection>, PortError> {
            Ok(self.0.clone())
        }
    }

    impl BrowserPort for Calls {
        fn open(&self, url: &SafeHttpUrl) -> Result<(), PortError> {
            self.browser
                .lock()
                .expect("browser mutex")
                .push(url.as_str().to_owned());
            Ok(())
        }
    }

    impl ClipboardPort for Calls {
        fn set_text(&self, text: &str) -> Result<(), PortError> {
            self.clipboard
                .lock()
                .expect("clipboard mutex")
                .push(text.to_owned());
            Ok(())
        }
    }

    impl NotificationPort for Calls {
        fn notify(&self, notification: Notification) -> Result<(), PortError> {
            self.notifications
                .lock()
                .expect("notifications mutex")
                .push(notification);
            Ok(())
        }
    }

    struct StepClock(Mutex<Duration>);

    impl ClockPort for StepClock {
        fn now(&self) -> Duration {
            let mut value = self.0.lock().expect("clock mutex");
            *value += Duration::from_millis(1);
            *value
        }
    }

    fn detection(payload: &[u8]) -> Detection {
        Detection::new(
            payload.to_vec(),
            QrFormat::QrCode,
            [Point { x: 0, y: 0 }; 4],
        )
    }

    fn service(detections: Vec<Detection>, calls: Arc<Calls>) -> ScanService {
        ScanService::new(
            ScanPorts {
                capture: Arc::new(FixedCapture),
                decoder: Arc::new(FixedDecoder(detections)),
                browser: calls.clone(),
                clipboard: calls.clone(),
                notifications: calls,
                clock: Arc::new(StepClock(Mutex::new(Duration::ZERO))),
            },
            Arc::new(SettingsState::new(AppSettings::default())),
        )
    }

    fn service_with_settings(
        detections: Vec<Detection>,
        calls: Arc<Calls>,
        settings: AppSettings,
    ) -> ScanService {
        ScanService::new(
            ScanPorts {
                capture: Arc::new(FixedCapture),
                decoder: Arc::new(FixedDecoder(detections)),
                browser: calls.clone(),
                clipboard: calls.clone(),
                notifications: calls,
                clock: Arc::new(StepClock(Mutex::new(Duration::ZERO))),
            },
            Arc::new(SettingsState::new(settings)),
        )
    }

    #[test]
    fn orchestrates_capture_decode_and_safe_url_policy() {
        let calls = Arc::new(Calls::default());
        let report = service(
            vec![detection(b"https://example.com/qrforge")],
            calls.clone(),
        )
        .scan();
        assert_eq!(report.outcome, ScanOutcome::UrlOpened);
        assert_eq!(report.metrics.detection_count, 1);
        assert_eq!(calls.browser.lock().expect("browser mutex").len(), 1);
        assert!(calls.clipboard.lock().expect("clipboard mutex").is_empty());
    }

    #[test]
    fn plain_text_is_copied_and_never_opened() {
        let calls = Arc::new(Calls::default());
        let report = service(vec![detection(b"private plain text")], calls.clone()).scan();
        assert_eq!(report.outcome, ScanOutcome::TextCopied);
        assert!(calls.browser.lock().expect("browser mutex").is_empty());
        assert_eq!(
            calls.clipboard.lock().expect("clipboard mutex").as_slice(),
            ["private plain text"]
        );
    }

    #[test]
    fn control_sequences_are_neither_copied_nor_opened() {
        let calls = Arc::new(Calls::default());
        let report = service(vec![detection(b"first line\r\nsecond line")], calls.clone()).scan();
        assert_eq!(report.outcome, ScanOutcome::UnsupportedPayload);
        assert!(calls.browser.lock().expect("browser mutex").is_empty());
        assert!(calls.clipboard.lock().expect("clipboard mutex").is_empty());
    }

    #[test]
    fn blocked_scheme_is_copied_but_never_opened() {
        let calls = Arc::new(Calls::default());
        let report = service(vec![detection(b"javascript:alert(1)")], calls.clone()).scan();
        assert_eq!(report.outcome, ScanOutcome::BlockedPayloadCopied);
        assert!(calls.browser.lock().expect("browser mutex").is_empty());
        assert_eq!(
            calls.clipboard.lock().expect("clipboard mutex").as_slice(),
            ["javascript:alert(1)"]
        );
    }

    #[test]
    fn spoofable_http_url_is_copied_but_never_opened() {
        let calls = Arc::new(Calls::default());
        let report = service(
            vec![detection(b"https://trusted.example@evil.example/")],
            calls.clone(),
        )
        .scan();
        assert_eq!(report.outcome, ScanOutcome::BlockedPayloadCopied);
        assert!(calls.browser.lock().expect("browser mutex").is_empty());
        assert_eq!(
            calls.clipboard.lock().expect("clipboard mutex").as_slice(),
            ["https://trusted.example@evil.example/"]
        );
    }

    #[test]
    fn multiple_results_never_trigger_browser_or_clipboard() {
        let calls = Arc::new(Calls::default());
        let report = service(
            vec![
                detection(b"https://example.com/one"),
                detection(b"https://example.com/two"),
            ],
            calls.clone(),
        )
        .scan();
        assert_eq!(report.outcome, ScanOutcome::MultipleCodes { count: 2 });
        assert!(calls.browser.lock().expect("browser mutex").is_empty());
        assert!(calls.clipboard.lock().expect("clipboard mutex").is_empty());
    }

    #[test]
    fn no_result_has_explicit_feedback() {
        let calls = Arc::new(Calls::default());
        let report = service(Vec::new(), calls.clone()).scan();
        assert_eq!(report.outcome, ScanOutcome::NoCode);
        assert_eq!(
            calls
                .notifications
                .lock()
                .expect("notifications mutex")
                .as_slice(),
            [Notification::NoQrFound]
        );
    }

    struct FallbackCapture;

    impl CapturePort for FallbackCapture {
        fn monitors(&self) -> Result<Vec<MonitorInfo>, PortError> {
            Ok(vec![test_monitor()])
        }

        fn capture(&self, requested: Option<&MonitorId>) -> Result<CaptureOutput, PortError> {
            assert_eq!(
                requested.map(MonitorId::as_str),
                Some("disconnected-display")
            );
            Ok(CaptureOutput {
                frame: CapturedFrame::rgba8(1, 1, vec![255; 4])
                    .map_err(|error| PortError::new("capture", error.to_string()))?,
                monitor: test_monitor(),
                used_fallback: true,
            })
        }
    }

    #[test]
    fn disconnected_monitor_falls_back_with_explicit_feedback_and_metadata() {
        let calls = Arc::new(Calls::default());
        let scan = ScanService::new(
            ScanPorts {
                capture: Arc::new(FallbackCapture),
                decoder: Arc::new(FixedDecoder(Vec::new())),
                browser: calls.clone(),
                clipboard: calls.clone(),
                notifications: calls.clone(),
                clock: Arc::new(StepClock(Mutex::new(Duration::ZERO))),
            },
            Arc::new(SettingsState::new(AppSettings {
                scan_monitor_id: Some("disconnected-display".parse().expect("monitor id")),
                ..AppSettings::default()
            })),
        );

        let report = scan.scan();
        assert_eq!(report.outcome, ScanOutcome::NoCode);
        assert_eq!(
            report
                .capture_metadata
                .as_ref()
                .and_then(|metadata| metadata.monitor_label.as_deref()),
            Some("Test display")
        );
        assert_eq!(
            calls
                .notifications
                .lock()
                .expect("notifications mutex")
                .as_slice(),
            [
                Notification::SelectedMonitorUnavailable,
                Notification::NoQrFound
            ]
        );
    }

    struct FailingCapture;

    impl CapturePort for FailingCapture {
        fn monitors(&self) -> Result<Vec<MonitorInfo>, PortError> {
            Ok(vec![test_monitor()])
        }

        fn capture(&self, _requested: Option<&MonitorId>) -> Result<CaptureOutput, PortError> {
            Err(PortError::new(
                "capture_selected_monitor",
                "sensitive operating-system detail",
            ))
        }
    }

    struct FailingDecoder;

    impl DecoderPort for FailingDecoder {
        fn decode(&self, _frame: &CapturedFrame) -> Result<Vec<Detection>, PortError> {
            Err(PortError::new(
                "decode_qr",
                "sensitive decoder implementation detail",
            ))
        }
    }

    #[test]
    fn capture_and_decoder_failures_have_distinct_payload_free_feedback() {
        for (capture, decoder, outcome, notification) in [
            (
                Arc::new(FailingCapture) as Arc<dyn CapturePort>,
                Arc::new(FixedDecoder(Vec::new())) as Arc<dyn DecoderPort>,
                ScanOutcome::Failed {
                    stage: FailureStage::Capture,
                },
                Notification::CaptureFailed,
            ),
            (
                Arc::new(FixedCapture) as Arc<dyn CapturePort>,
                Arc::new(FailingDecoder) as Arc<dyn DecoderPort>,
                ScanOutcome::Failed {
                    stage: FailureStage::Decode,
                },
                Notification::DecoderFailed,
            ),
        ] {
            let calls = Arc::new(Calls::default());
            let scan = ScanService::new(
                ScanPorts {
                    capture,
                    decoder,
                    browser: calls.clone(),
                    clipboard: calls.clone(),
                    notifications: calls.clone(),
                    clock: Arc::new(StepClock(Mutex::new(Duration::ZERO))),
                },
                Arc::new(SettingsState::new(AppSettings::default())),
            );
            let report = scan.scan();
            assert_eq!(report.outcome, outcome);
            assert_eq!(
                calls
                    .notifications
                    .lock()
                    .expect("notifications mutex")
                    .as_slice(),
                [notification]
            );
        }
    }

    struct BlockingCapture {
        entered: Mutex<Option<mpsc::Sender<()>>>,
        release: Arc<Barrier>,
    }

    impl CapturePort for BlockingCapture {
        fn monitors(&self) -> Result<Vec<MonitorInfo>, PortError> {
            Ok(vec![test_monitor()])
        }

        fn capture(&self, _requested: Option<&MonitorId>) -> Result<CaptureOutput, PortError> {
            if let Some(sender) = self.entered.lock().expect("entered mutex").take() {
                sender.send(()).expect("test receiver should remain");
            }
            self.release.wait();
            Ok(CaptureOutput {
                frame: CapturedFrame::rgba8(1, 1, vec![255; 4])
                    .map_err(|error| PortError::new("capture", error.to_string()))?,
                monitor: test_monitor(),
                used_fallback: false,
            })
        }
    }

    #[test]
    fn overlapping_scan_is_rejected() {
        let (entered_sender, entered_receiver) = mpsc::channel();
        let release = Arc::new(Barrier::new(2));
        let calls = Arc::new(Calls::default());
        let scan = Arc::new(ScanService::new(
            ScanPorts {
                capture: Arc::new(BlockingCapture {
                    entered: Mutex::new(Some(entered_sender)),
                    release: release.clone(),
                }),
                decoder: Arc::new(FixedDecoder(Vec::new())),
                browser: calls.clone(),
                clipboard: calls.clone(),
                notifications: calls,
                clock: Arc::new(StepClock(Mutex::new(Duration::ZERO))),
            },
            Arc::new(SettingsState::new(AppSettings::default())),
        ));
        let background = {
            let scan = scan.clone();
            std::thread::spawn(move || scan.scan())
        };
        entered_receiver.recv().expect("capture should start");
        assert_eq!(scan.scan().outcome, ScanOutcome::AlreadyInProgress);
        release.wait();
        assert_eq!(
            background.join().expect("scan thread").outcome,
            ScanOutcome::NoCode
        );
    }

    #[test]
    fn auto_open_disabled_detects_safe_url_without_opening_or_clipboard() {
        let calls = Arc::new(Calls::default());
        let report = service_with_settings(
            vec![detection(b"https://example.com/qrforge")],
            calls.clone(),
            AppSettings {
                auto_open_safe_urls: false,
                ..AppSettings::default()
            },
        )
        .scan();
        assert_eq!(report.outcome, ScanOutcome::UrlDetected);
        assert!(calls.browser.lock().expect("browser mutex").is_empty());
        assert!(calls.clipboard.lock().expect("clipboard mutex").is_empty());
    }

    #[test]
    fn clipboard_disabled_treats_plain_text_as_unsupported() {
        let calls = Arc::new(Calls::default());
        let report = service_with_settings(
            vec![detection(b"private plain text")],
            calls.clone(),
            AppSettings {
                copy_non_url_payloads: false,
                ..AppSettings::default()
            },
        )
        .scan();
        assert_eq!(report.outcome, ScanOutcome::UnsupportedPayload);
        assert!(calls.browser.lock().expect("browser mutex").is_empty());
        assert!(calls.clipboard.lock().expect("clipboard mutex").is_empty());
    }

    #[test]
    fn notifications_disabled_suppresses_feedback() {
        let calls = Arc::new(Calls::default());
        let report = service_with_settings(
            Vec::new(),
            calls.clone(),
            AppSettings {
                notifications_enabled: false,
                ..AppSettings::default()
            },
        )
        .scan();
        assert_eq!(report.outcome, ScanOutcome::NoCode);
        assert!(
            calls
                .notifications
                .lock()
                .expect("notifications mutex")
                .is_empty()
        );
    }

    #[test]
    fn unicode_payload_is_classified_as_plain_text() {
        let calls = Arc::new(Calls::default());
        let report = service(vec![detection("héllo 🌍 wörld".as_bytes())], calls.clone()).scan();
        assert_eq!(report.outcome, ScanOutcome::TextCopied);
        assert_eq!(
            calls.clipboard.lock().expect("clipboard mutex").as_slice(),
            ["héllo 🌍 wörld"]
        );
        assert!(calls.browser.lock().expect("browser mutex").is_empty());
    }

    #[test]
    fn malformed_url_like_text_has_a_distinct_outcome() {
        let calls = Arc::new(Calls::default());
        let report = service(vec![detection(b"http:///missing-host")], calls.clone()).scan();
        assert_eq!(report.outcome, ScanOutcome::MalformedTextCopied);
        assert!(calls.browser.lock().expect("browser mutex").is_empty());
        assert_eq!(
            calls.clipboard.lock().expect("clipboard mutex").as_slice(),
            ["http:///missing-host"]
        );
    }
}
