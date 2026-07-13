use qrforge_application::ScanReport;
use serde_json::json;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Duration,
};

/// Explicitly enabled, payload-free local diagnostics for release verification.
pub struct Diagnostics {
    path: Option<PathBuf>,
    write_lock: Mutex<()>,
}

impl Diagnostics {
    /// Enables diagnostics only when `QRFORGE_DIAGNOSTICS=1` is present.
    #[must_use]
    pub fn new(app_data_dir: &Path) -> Self {
        let enabled = std::env::var("QRFORGE_DIAGNOSTICS").as_deref() == Ok("1");
        let path = enabled
            .then(|| app_data_dir.join("diagnostics.jsonl"))
            .filter(|path| initialize(path).is_ok());
        Self {
            path,
            write_lock: Mutex::new(()),
        }
    }

    /// Records host startup latency without device or payload data.
    pub fn record_startup(&self, elapsed: Duration) {
        self.append(&json!({
            "event": "startup",
            "elapsedMs": duration_ms(elapsed),
            "pid": std::process::id()
        }));
    }

    /// Records a scan outcome and timings without QR content.
    pub fn record_scan(&self, trigger: &str, dispatch_elapsed: Duration, report: &ScanReport) {
        self.append(&json!({
            "event": "scan",
            "trigger": trigger,
            "hotkeyToResultMs": duration_ms(dispatch_elapsed),
            "report": report
        }));
    }

    /// Records lazy settings-window lifecycle evidence.
    pub fn record_window(&self, event: &str, elapsed: Option<Duration>) {
        self.append(&json!({
            "event": event,
            "elapsedMs": elapsed.map(duration_ms)
        }));
    }

    fn append(&self, value: &serde_json::Value) {
        let Some(path) = self.path.as_ref() else {
            return;
        };
        let _guard = self
            .write_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = serde_json::to_writer(&mut file, value);
            let _ = file.write_all(b"\n");
        }
    }
}

fn initialize(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    File::create(path).map(drop)
}

fn duration_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}
