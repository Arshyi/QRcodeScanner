use crate::Hotkey;
use serde::{Deserialize, Serialize};

/// Current on-disk settings schema version.
pub const SETTINGS_SCHEMA_VERSION: u32 = 1;

/// Validated user settings shared by the host and typed IPC view.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Schema version written to disk.
    pub schema_version: u32,
    /// Active scan shortcut preference.
    pub hotkey: Hotkey,
    /// Whether the operating system starts QRForge after sign-in.
    pub launch_at_startup: bool,
    /// Whether a single validated HTTP(S) result is opened automatically.
    pub auto_open_safe_urls: bool,
    /// Whether UTF-8 payloads that are not openable are copied.
    pub copy_non_url_payloads: bool,
    /// Whether native notifications and tray feedback are shown.
    pub notifications_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            hotkey: Hotkey::default(),
            launch_at_startup: false,
            auto_open_safe_urls: true,
            copy_non_url_payloads: true,
            notifications_enabled: true,
        }
    }
}
