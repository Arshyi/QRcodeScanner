use qrforge_application::{FailureStage, ScanOutcome, ScanReport, SettingsSnapshot};
use serde_json::json;
use std::{
    collections::VecDeque,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Mutex, PoisonError},
    time::{Duration, Instant},
};

const LOG_NAME: &str = "diagnostics.jsonl";
const ARCHIVE_NAME: &str = "diagnostics.jsonl.1";
const MAX_LOG_BYTES: u64 = 256 * 1024;
const MAX_RECENT_ERRORS: usize = 8;
const MAX_FIELD_CHARS: usize = 80;

/// Inputs used to produce the user-copyable, privacy-safe diagnostic snapshot.
pub struct DiagnosticSnapshot<'a> {
    /// Current settings and actual hotkey registration state.
    pub settings: &'a SettingsSnapshot,
    /// Current number of enumerated displays.
    pub monitor_count: usize,
    /// Whether the configured display is currently present.
    pub configured_monitor_available: bool,
    /// Whether display enumeration failed.
    pub monitor_error: bool,
}

/// Explicitly enabled, payload-free local diagnostics for release verification.
pub struct Diagnostics {
    path: Option<PathBuf>,
    write_lock: Mutex<()>,
    recent_errors: Mutex<VecDeque<&'static str>>,
    process_started: Instant,
}

impl Diagnostics {
    /// Enables diagnostics only when `QRFORGE_DIAGNOSTICS=1` is present.
    #[must_use]
    pub fn new(app_data_dir: &Path, process_started: Instant) -> Self {
        let enabled = std::env::var("QRFORGE_DIAGNOSTICS").as_deref() == Ok("1");
        let path = enabled
            .then(|| app_data_dir.join(LOG_NAME))
            .filter(|path| initialize(path).is_ok());
        Self::with_path(path, process_started)
    }

    fn with_path(path: Option<PathBuf>, process_started: Instant) -> Self {
        Self {
            path,
            write_lock: Mutex::new(()),
            recent_errors: Mutex::new(VecDeque::with_capacity(MAX_RECENT_ERRORS)),
            process_started,
        }
    }

    /// Records a stable error category for the current process only.
    pub fn record_error(&self, category: &'static str) {
        let mut errors = self
            .recent_errors
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if errors.back().copied() == Some(category) {
            return;
        }
        if errors.len() == MAX_RECENT_ERRORS {
            errors.pop_front();
        }
        errors.push_back(category);
    }

    /// Records host startup latency without device or payload data.
    pub fn record_startup(&self, elapsed: Duration) {
        self.append(&json!({
            "event": "startup",
            "elapsedMs": duration_ms(elapsed),
            "pid": std::process::id()
        }));
    }

    /// Records a scan outcome and timings without QR content or display labels.
    pub fn record_scan(&self, trigger: &str, dispatch_elapsed: Duration, report: &ScanReport) {
        if let ScanOutcome::Failed { stage } = report.outcome {
            self.record_error(match stage {
                FailureStage::Capture => "scan_capture_failed",
                FailureStage::Decode => "scan_decode_failed",
                FailureStage::Browser => "browser_open_failed",
                FailureStage::Clipboard => "clipboard_write_failed",
            });
        }
        self.append(&json!({
            "event": "scan",
            "trigger": trigger,
            "hotkeyToResultMs": duration_ms(dispatch_elapsed),
            "outcome": report.outcome,
            "metrics": report.metrics,
            "capture": report.capture_metadata.as_ref().map(|capture| json!({
                "scaleFactorPercent": capture.scale_factor_percent,
                "pixelWidth": capture.pixel_width,
                "pixelHeight": capture.pixel_height
            }))
        }));
    }

    /// Records lazy settings-window lifecycle evidence.
    pub fn record_window(&self, event: &str, elapsed: Option<Duration>) {
        self.append(&json!({
            "event": event,
            "elapsedMs": elapsed.map(duration_ms)
        }));
    }

    /// Builds fixed-order, privacy-safe text intended for the clipboard.
    #[must_use]
    pub fn snapshot_text(&self, input: &DiagnosticSnapshot<'_>) -> String {
        let errors = self
            .recent_errors
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .copied()
            .collect::<Vec<_>>()
            .join(",");
        let selected = if input.settings.settings.scan_monitor_id.is_some() {
            "configured"
        } else {
            "automatic"
        };
        let logging = if self.path.is_some() {
            "enabled"
        } else {
            "disabled"
        };
        format!(
            concat!(
                "QRForge diagnostics\n",
                "formatVersion=1\n",
                "appVersion={}\n",
                "buildCommit={}\n",
                "target={}-{}\n",
                "windowsVersion={}\n",
                "settingsSchema={}\n",
                "hotkeyRegistered={}\n",
                "startupConfigured={}\n",
                "monitorSelection={}\n",
                "configuredMonitorAvailable={}\n",
                "monitorEnumeration={}\n",
                "monitorCount={}\n",
                "decoder=zxing-cpp-0.5.2\n",
                "diagnosticLogging={}\n",
                "logLocation=QRForge app data\\diagnostics.jsonl\n",
                "logRetention=256KiB plus one archive\n",
                "recentErrors={}\n",
                "pid={}\n",
                "uptimeSeconds={}\n"
            ),
            env!("CARGO_PKG_VERSION"),
            env!("QRFORGE_BUILD_COMMIT"),
            std::env::consts::OS,
            std::env::consts::ARCH,
            windows_version(),
            input.settings.settings.schema_version,
            input.settings.hotkey_registered,
            input.settings.settings.launch_at_startup,
            selected,
            input.configured_monitor_available,
            if input.monitor_error { "failed" } else { "ok" },
            input.monitor_count,
            logging,
            if errors.is_empty() { "none" } else { &errors },
            std::process::id(),
            self.process_started.elapsed().as_secs()
        )
    }

    fn append(&self, value: &serde_json::Value) {
        let Some(path) = self.path.as_ref() else {
            return;
        };
        let Ok(mut bytes) = serde_json::to_vec(value) else {
            return;
        };
        bytes.push(b'\n');
        let _guard = self
            .write_lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if rotate_if_needed(path, bytes.len() as u64).is_err() {
            return;
        }
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = file.write_all(&bytes);
        }
    }
}

fn initialize(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map(drop)
}

fn rotate_if_needed(path: &Path, incoming_bytes: u64) -> std::io::Result<()> {
    if incoming_bytes > MAX_LOG_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "diagnostic record exceeds the active log limit",
        ));
    }
    let current_bytes = fs::metadata(path).map_or(0, |metadata| metadata.len());
    if current_bytes == 0 || current_bytes.saturating_add(incoming_bytes) <= MAX_LOG_BYTES {
        return Ok(());
    }
    let archive = path.with_file_name(ARCHIVE_NAME);
    match fs::remove_file(&archive) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    // Phase 1.1 logs had no size limit. Do not preserve an arbitrarily large
    // legacy file as the single archive, because that would violate the
    // documented two-file retention bound indefinitely.
    if current_bytes > MAX_LOG_BYTES {
        fs::remove_file(path)?;
        return Ok(());
    }
    fs::rename(path, archive)
}

fn duration_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(windows)]
fn windows_version() -> String {
    use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};

    let current_version = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");
    let Ok(current_version) = current_version else {
        return "unavailable".to_owned();
    };
    let product: String = current_version
        .get_value("ProductName")
        .unwrap_or_else(|_| "Windows".to_owned());
    let release: String = current_version
        .get_value("DisplayVersion")
        .unwrap_or_else(|_| "unknown-release".to_owned());
    let build: String = current_version
        .get_value("CurrentBuildNumber")
        .unwrap_or_else(|_| "unknown-build".to_owned());
    sanitize_field(&format!("{product} {release} build {build}"))
}

#[cfg(not(windows))]
fn windows_version() -> String {
    "not-windows".to_owned()
}

fn sanitize_field(value: &str) -> String {
    let sanitized = value
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, ' ' | '.' | '-' | '(' | ')')
        })
        .take(MAX_FIELD_CHARS)
        .collect::<String>();
    if sanitized.is_empty() {
        "unavailable".to_owned()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrforge_domain::AppSettings;
    use tempfile::tempdir;

    fn settings() -> SettingsSnapshot {
        SettingsSnapshot {
            settings: AppSettings::default(),
            active_hotkey: Some("Ctrl+Shift+Q".to_owned()),
            hotkey_registered: true,
        }
    }

    #[test]
    fn copyable_snapshot_has_stable_order_and_no_sensitive_values() {
        let diagnostics = Diagnostics::with_path(None, Instant::now());
        diagnostics.record_error("scan_decode_failed");
        let text = diagnostics.snapshot_text(&DiagnosticSnapshot {
            settings: &settings(),
            monitor_count: 2,
            configured_monitor_available: true,
            monitor_error: false,
        });

        assert!(text.starts_with("QRForge diagnostics\nformatVersion=1\nappVersion="));
        assert!(text.contains("recentErrors=scan_decode_failed\n"));
        assert!(text.contains("logLocation=QRForge app data\\diagnostics.jsonl\n"));
        for forbidden in ["C:\\Users", "PRIVATE-QR", "USERNAME=", "TOKEN="] {
            assert!(!text.contains(forbidden));
        }
    }

    #[test]
    fn recent_errors_are_deduplicated_and_bounded() {
        let diagnostics = Diagnostics::with_path(None, Instant::now());
        for category in [
            "one", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
        ] {
            diagnostics.record_error(category);
        }
        let errors = diagnostics.recent_errors.lock().expect("recent errors");
        assert_eq!(errors.len(), MAX_RECENT_ERRORS);
        assert_eq!(errors.front().copied(), Some("two"));
        assert_eq!(errors.back().copied(), Some("nine"));
    }

    #[test]
    fn logging_is_preserved_and_rotates_to_exactly_one_archive() {
        let directory = tempdir().expect("tempdir");
        let path = directory.path().join(LOG_NAME);
        fs::write(&path, b"prior-session\n").expect("seed log");
        initialize(&path).expect("initialize existing log");
        assert_eq!(
            fs::read(&path).expect("read existing log"),
            b"prior-session\n"
        );

        let diagnostics = Diagnostics::with_path(Some(path.clone()), Instant::now());
        for _ in 0..6_000 {
            diagnostics.record_window("settings_window_created", Some(Duration::from_millis(1)));
        }

        let archive = path.with_file_name(ARCHIVE_NAME);
        assert!(archive.is_file());
        assert!(fs::metadata(&path).expect("active metadata").len() <= MAX_LOG_BYTES);
        assert!(fs::metadata(&archive).expect("archive metadata").len() <= MAX_LOG_BYTES);
        assert!(!directory.path().join("diagnostics.jsonl.2").exists());
    }

    #[test]
    fn oversized_legacy_log_is_discarded_before_the_next_record() {
        let directory = tempdir().expect("tempdir");
        let path = directory.path().join(LOG_NAME);
        let max_log_bytes = usize::try_from(MAX_LOG_BYTES).expect("log limit fits usize");
        fs::write(&path, vec![b'x'; max_log_bytes + 1]).expect("seed legacy log");
        let diagnostics = Diagnostics::with_path(Some(path.clone()), Instant::now());

        diagnostics.record_window("settings_window_created", Some(Duration::from_millis(1)));

        let active = fs::read_to_string(&path).expect("read bounded active log");
        assert!(active.len() < max_log_bytes);
        let record: serde_json::Value =
            serde_json::from_str(active.trim()).expect("active log remains JSONL");
        assert_eq!(record["event"], "settings_window_created");
        assert!(!path.with_file_name(ARCHIVE_NAME).exists());
    }

    #[test]
    fn rotation_rejects_a_record_larger_than_the_log_limit() {
        let directory = tempdir().expect("tempdir");
        let path = directory.path().join(LOG_NAME);

        let error = rotate_if_needed(&path, MAX_LOG_BYTES + 1)
            .expect_err("oversized record must be rejected");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(!path.exists());
    }

    #[test]
    fn registry_fields_are_strictly_sanitized_and_bounded() {
        let value = sanitize_field("Windows\nC:\\Users\\alice TOKEN=secret / path");
        assert!(!value.contains(['\\', '/', '=', '\n']));
        assert!(value.chars().count() <= MAX_FIELD_CHARS);
    }
}
