use qrforge_application::{Notification, NotificationPort, PortError};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tauri::{AppHandle, Wry};
use tauri_plugin_notification::NotificationExt;

/// Native notification adapter with a tray-tooltip fallback.
pub struct TauriNotifications {
    app: AppHandle<Wry>,
    generation: Arc<AtomicU64>,
}

impl TauriNotifications {
    /// Creates an adapter for the running Tauri host.
    #[must_use]
    pub fn new(app: AppHandle<Wry>) -> Self {
        Self {
            app,
            generation: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl NotificationPort for TauriNotifications {
    fn notify(&self, notification: Notification) -> Result<(), PortError> {
        let message = message(notification);
        let native_result = self
            .app
            .notification()
            .builder()
            .title("QRForge")
            .body(&message)
            .show();
        let tray_result = self.app.tray_by_id("main-tray").map_or_else(
            || Err(tauri::Error::AssetNotFound("main-tray".to_owned())),
            |tray| tray.set_tooltip(Some(format!("QRForge — {message}"))),
        );
        if tray_result.is_ok() {
            let generation = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
            let counter = self.generation.clone();
            let app = self.app.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(4));
                if counter.load(Ordering::Acquire) == generation
                    && let Some(tray) = app.tray_by_id("main-tray")
                {
                    let _ = tray.set_tooltip(Some("QRForge — Ctrl+Shift+Q to scan"));
                }
            });
        }
        if native_result.is_err() && tray_result.is_err() {
            return Err(PortError::new(
                "notification",
                "native and tray feedback were unavailable",
            ));
        }
        Ok(())
    }
}

fn message(notification: Notification) -> String {
    match notification {
        Notification::QrOpened => "QR link opened".to_owned(),
        Notification::TextCopied => "QR text copied".to_owned(),
        Notification::BlockedPayloadCopied => {
            "Blocked link type copied; nothing was opened".to_owned()
        }
        Notification::NoQrFound => "No QR code found".to_owned(),
        Notification::MultipleQrFound(count) => {
            format!("Found {count} QR codes; no automatic action taken")
        }
        Notification::ScanAlreadyInProgress => "A scan is already in progress".to_owned(),
        Notification::HotkeyConflict => {
            "The scan hotkey is unavailable; choose another in Settings".to_owned()
        }
        Notification::UnsupportedPayload => {
            "QR content is unsupported or copying is disabled".to_owned()
        }
        Notification::SafeUrlNotOpened => {
            "A safe link was found; automatic opening is disabled".to_owned()
        }
        Notification::ScanFailed => "The scan could not be completed".to_owned(),
    }
}
