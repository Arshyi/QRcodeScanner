use crate::{diagnostics::DiagnosticSnapshot, runtime::RuntimeState};
use qrforge_application::{
    Notification, PendingResultsView, ResultActionError, ResultActionOutcome, ResultActionRequest,
    SettingsError, SettingsSnapshot, SettingsUpdate,
};
use qrforge_domain::MonitorInfo;
use serde::Serialize;
use tauri::{AppHandle, State, Wry};

/// Complete typed settings response consumed by the Svelte UI.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    /// Validated settings and actual hotkey registration state.
    pub snapshot: SettingsSnapshot,
    /// Semantic application version.
    pub version: &'static str,
    /// Source commit embedded at compile time.
    pub commit: &'static str,
    /// Compile-time target identifier.
    pub build: String,
    /// Current native display topology.
    pub monitors: Vec<MonitorInfo>,
    /// Whether the configured display is present in the current topology.
    pub configured_monitor_available: bool,
    /// User-safe display enumeration error, if any.
    pub monitor_error: Option<&'static str>,
}

/// Sanitized typed IPC error.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    /// Stable machine-readable code.
    pub code: &'static str,
    /// User-safe message that never contains QR payloads.
    pub message: &'static str,
}

/// Confirmation for an explicit privacy-safe diagnostics copy action.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyDiagnosticsOutcome {
    /// User-safe status text.
    pub message: &'static str,
}

/// Returns the current validated settings view.
#[tauri::command]
// Tauri deserializes each command argument through its `CommandArg` trait,
// which requires owned values for non-`State` types. Taking references here
// would break the IPC deserializer.
#[allow(clippy::needless_pass_by_value)]
pub fn get_settings(state: State<'_, RuntimeState>) -> SettingsView {
    view(&state, state.settings.snapshot())
}

/// Applies a complete typed settings update transactionally.
#[tauri::command]
// `request` is deserialized by Tauri from the IPC payload, and `state` is
// provided by Tauri's managed-state injection. Both must be owned values.
#[allow(clippy::needless_pass_by_value)]
pub fn update_settings(
    request: SettingsUpdate,
    state: State<'_, RuntimeState>,
    app: AppHandle<Wry>,
) -> Result<SettingsView, CommandError> {
    match state.settings.update(&request) {
        Ok(snapshot) => {
            let _ = crate::tray::refresh_idle_tooltip(&app);
            Ok(view(&state, snapshot))
        }
        Err(SettingsError::InvalidHotkey(_)) => {
            state.diagnostics.record_error("settings_invalid_hotkey");
            Err(CommandError {
                code: "invalid_hotkey",
                message: "Use a non-reserved shortcut with a modifier and one letter, digit, or F-key.",
            })
        }
        Err(SettingsError::InvalidMonitor(_)) => {
            state.diagnostics.record_error("settings_invalid_monitor");
            Err(CommandError {
                code: "invalid_monitor",
                message: "That display selection is invalid. Refresh the display list and try again.",
            })
        }
        Err(SettingsError::HotkeyRegistration(_)) => {
            state.diagnostics.record_error("hotkey_registration_failed");
            let _ = state.notifications.notify(Notification::HotkeyConflict);
            Err(CommandError {
                code: "hotkey_conflict",
                message: "That shortcut is already in use. The previous shortcut is still active.",
            })
        }
        Err(_) => {
            state.diagnostics.record_error("settings_update_failed");
            Err(CommandError {
                code: "settings_update_failed",
                message: "Settings could not be saved. Existing settings remain active.",
            })
        }
    }
}

/// Marks the first-run local-processing introduction as complete.
#[tauri::command]
// Managed state is injected by Tauri as an owned command argument.
#[allow(clippy::needless_pass_by_value)]
pub fn complete_onboarding(state: State<'_, RuntimeState>) -> Result<SettingsView, CommandError> {
    state
        .settings
        .complete_onboarding()
        .map(|snapshot| view(&state, snapshot))
        .map_err(|_| {
            state.diagnostics.record_error("onboarding_save_failed");
            CommandError {
                code: "onboarding_save_failed",
                message: "QRForge could not save first-run completion. Please try again.",
            }
        })
}

/// Copies a fixed-format support snapshot that excludes user content and paths.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn copy_diagnostics(
    state: State<'_, RuntimeState>,
) -> Result<CopyDiagnosticsOutcome, CommandError> {
    let snapshot = state.settings.snapshot();
    let monitor_result = state.capture.monitors();
    let (monitor_count, configured_monitor_available, monitor_error) = match monitor_result {
        Ok(monitors) => {
            let available = snapshot
                .settings
                .scan_monitor_id
                .as_ref()
                .is_none_or(|id| monitors.iter().any(|monitor| &monitor.id == id));
            (monitors.len(), available, false)
        }
        Err(_) => (0, snapshot.settings.scan_monitor_id.is_none(), true),
    };
    let text = state.diagnostics.snapshot_text(&DiagnosticSnapshot {
        settings: &snapshot,
        monitor_count,
        configured_monitor_available,
        monitor_error,
    });
    state.clipboard.set_text(&text).map_err(|_| {
        state.diagnostics.record_error("diagnostics_copy_failed");
        CommandError {
            code: "diagnostics_copy_failed",
            message: "Windows could not copy the privacy-safe diagnostics.",
        }
    })?;
    Ok(CopyDiagnosticsOutcome {
        message: "Privacy-safe diagnostics copied.",
    })
}

/// Returns the current native multi-code chooser session.
#[tauri::command]
// Managed state is injected by Tauri as an owned command argument.
#[allow(clippy::needless_pass_by_value)]
pub fn get_pending_results(
    state: State<'_, RuntimeState>,
) -> Result<PendingResultsView, CommandError> {
    state.results.snapshot().ok_or(CommandError {
        code: "results_unavailable",
        message: "These scan results are no longer available. Scan again to refresh them.",
    })
}

/// Performs an explicit Rust-side open, copy, copy-all, or dismiss action.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn perform_result_action(
    request: ResultActionRequest,
    state: State<'_, RuntimeState>,
    app: AppHandle<Wry>,
) -> Result<ResultActionOutcome, CommandError> {
    let outcome = state.results.perform(&request).map_err(|error| {
        let safe = result_error(&error);
        state.diagnostics.record_error(safe.code);
        safe
    })?;
    if outcome.close {
        let _ = crate::window::close_results(&app);
    }
    Ok(outcome)
}

fn result_error(error: &ResultActionError) -> CommandError {
    match error {
        ResultActionError::StaleSession => CommandError {
            code: "stale_results",
            message: "These results were replaced or dismissed. Scan again to continue.",
        },
        ResultActionError::InvalidRequest | ResultActionError::InvalidIndex => CommandError {
            code: "invalid_result_action",
            message: "The requested result action was invalid.",
        },
        ResultActionError::NotOpenable => CommandError {
            code: "result_not_openable",
            message: "Rust safety policy does not allow this result to be opened.",
        },
        ResultActionError::NotCopyable => CommandError {
            code: "result_not_copyable",
            message: "This result cannot be copied safely.",
        },
        ResultActionError::Browser(_) => CommandError {
            code: "browser_failed",
            message: "Windows could not open the approved link.",
        },
        ResultActionError::Clipboard(_) => CommandError {
            code: "clipboard_failed",
            message: "Windows could not update the clipboard.",
        },
    }
}

fn view(state: &RuntimeState, snapshot: SettingsSnapshot) -> SettingsView {
    let (monitors, monitor_error) = state.capture.monitors().map_or_else(
        |_| (Vec::new(), Some("Displays could not be refreshed.")),
        |monitors| (monitors, None),
    );
    let configured_monitor_available = snapshot
        .settings
        .scan_monitor_id
        .as_ref()
        .is_none_or(|id| monitors.iter().any(|monitor| &monitor.id == id));
    SettingsView {
        snapshot,
        version: env!("CARGO_PKG_VERSION"),
        commit: env!("QRFORGE_BUILD_COMMIT"),
        build: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        monitors,
        configured_monitor_available,
        monitor_error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrforge_application::PortError;

    #[test]
    fn result_errors_are_sanitized_before_crossing_ipc() {
        let secret = "private payload and C:\\Users\\name\\machine-path";
        for error in [
            ResultActionError::Browser(PortError::new("browser", secret)),
            ResultActionError::Clipboard(PortError::new("clipboard", secret)),
        ] {
            let safe = result_error(&error);
            assert!(!safe.message.contains(secret));
            assert!(!safe.message.contains("C:\\"));
            assert!(matches!(safe.code, "browser_failed" | "clipboard_failed"));
        }
    }
}
