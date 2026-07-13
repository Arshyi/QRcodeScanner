use crate::runtime::RuntimeState;
use qrforge_application::{Notification, SettingsError, SettingsSnapshot, SettingsUpdate};
use serde::Serialize;
use tauri::State;

/// Complete typed settings response consumed by the Svelte UI.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    /// Validated settings and actual hotkey registration state.
    pub snapshot: SettingsSnapshot,
    /// Semantic application version.
    pub version: &'static str,
    /// Compile-time target identifier.
    pub build: String,
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

/// Returns the current validated settings view.
#[tauri::command]
// Tauri deserializes each command argument through its `CommandArg` trait,
// which requires owned values for non-`State` types. Taking references here
// would break the IPC deserializer.
#[allow(clippy::needless_pass_by_value)]
pub fn get_settings(state: State<'_, RuntimeState>) -> SettingsView {
    view(state.settings.snapshot())
}

/// Applies a complete typed settings update transactionally.
#[tauri::command]
// `request` is deserialized by Tauri from the IPC payload, and `state` is
// provided by Tauri's managed-state injection. Both must be owned values.
#[allow(clippy::needless_pass_by_value)]
pub fn update_settings(
    request: SettingsUpdate,
    state: State<'_, RuntimeState>,
) -> Result<SettingsView, CommandError> {
    match state.settings.update(&request) {
        Ok(snapshot) => Ok(view(snapshot)),
        Err(SettingsError::InvalidHotkey(_)) => Err(CommandError {
            code: "invalid_hotkey",
            message: "Use at least one modifier and one letter, digit, or F-key.",
        }),
        Err(SettingsError::HotkeyRegistration(_)) => {
            let _ = state.notifications.notify(Notification::HotkeyConflict);
            Err(CommandError {
                code: "hotkey_conflict",
                message: "That shortcut is already in use. The previous shortcut is still active.",
            })
        }
        Err(_) => Err(CommandError {
            code: "settings_update_failed",
            message: "Settings could not be saved. Existing settings remain active.",
        }),
    }
}

fn view(snapshot: SettingsSnapshot) -> SettingsView {
    SettingsView {
        snapshot,
        version: env!("CARGO_PKG_VERSION"),
        build: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
    }
}
