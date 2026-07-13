use crate::{HotkeyPort, PortError, SettingsRepository, StartupPort};
use qrforge_domain::{AppSettings, Hotkey, SETTINGS_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Shared in-memory settings used on the scan path without disk I/O.
pub struct SettingsState(RwLock<AppSettings>);

impl SettingsState {
    /// Creates state from validated persisted settings.
    #[must_use]
    pub fn new(settings: AppSettings) -> Self {
        Self(RwLock::new(settings))
    }

    /// Returns a consistent settings snapshot.
    #[must_use]
    pub fn get(&self) -> AppSettings {
        self.0
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn replace(&self, settings: AppSettings) {
        *self
            .0
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = settings;
    }
}

/// Typed settings update accepted at the IPC boundary.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsUpdate {
    /// Canonical or parseable requested hotkey.
    pub hotkey: String,
    /// Requested launch-at-sign-in state.
    pub launch_at_startup: bool,
    /// Requested single-safe-URL behavior.
    pub auto_open_safe_urls: bool,
    /// Requested non-URL clipboard behavior.
    pub copy_non_url_payloads: bool,
    /// Requested notification behavior.
    pub notifications_enabled: bool,
}

/// Typed settings snapshot returned to the frontend.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsSnapshot {
    /// Validated preferences.
    pub settings: AppSettings,
    /// Actual registered hotkey, if any.
    pub active_hotkey: Option<String>,
    /// Whether a global hotkey is currently registered.
    pub hotkey_registered: bool,
}

/// Transactional settings and platform-registration use case.
pub struct SettingsService {
    repository: Arc<dyn SettingsRepository>,
    hotkeys: Arc<dyn HotkeyPort>,
    startup: Arc<dyn StartupPort>,
    state: Arc<SettingsState>,
}

impl SettingsService {
    /// Creates a settings service over validated state and replaceable adapters.
    #[must_use]
    pub fn new(
        repository: Arc<dyn SettingsRepository>,
        hotkeys: Arc<dyn HotkeyPort>,
        startup: Arc<dyn StartupPort>,
        state: Arc<SettingsState>,
    ) -> Self {
        Self {
            repository,
            hotkeys,
            startup,
            state,
        }
    }

    /// Returns preferences plus actual hotkey and startup state.
    #[must_use]
    pub fn snapshot(&self) -> SettingsSnapshot {
        let mut settings = self.state.get();
        if let Ok(enabled) = self.startup.is_enabled() {
            settings.launch_at_startup = enabled;
        }
        let active = self.hotkeys.active();
        SettingsSnapshot {
            settings,
            active_hotkey: active.as_ref().map(ToString::to_string),
            hotkey_registered: active.is_some(),
        }
    }

    /// Applies platform changes and atomically persists them, rolling back on failure.
    pub fn update(&self, update: SettingsUpdate) -> Result<SettingsSnapshot, SettingsError> {
        let requested_hotkey: Hotkey = update
            .hotkey
            .parse()
            .map_err(SettingsError::InvalidHotkey)?;
        let old = self.state.get();
        let old_startup = self.startup.is_enabled().unwrap_or(old.launch_at_startup);
        let desired = AppSettings {
            schema_version: SETTINGS_SCHEMA_VERSION,
            hotkey: requested_hotkey,
            launch_at_startup: update.launch_at_startup,
            auto_open_safe_urls: update.auto_open_safe_urls,
            copy_non_url_payloads: update.copy_non_url_payloads,
            notifications_enabled: update.notifications_enabled,
        };

        let hotkey_changed = desired.hotkey != old.hotkey || self.hotkeys.active().is_none();
        if hotkey_changed {
            self.hotkeys
                .replace(&desired.hotkey)
                .map_err(SettingsError::HotkeyRegistration)?;
        }

        let startup_changed = desired.launch_at_startup != old_startup;
        if startup_changed && let Err(error) = self.startup.set_enabled(desired.launch_at_startup) {
            if hotkey_changed && self.hotkeys.replace(&old.hotkey).is_err() {
                return Err(SettingsError::RollbackFailed);
            }
            return Err(SettingsError::Startup(error));
        }

        if let Err(error) = self.repository.save(&desired) {
            let startup_rollback_failed =
                startup_changed && self.startup.set_enabled(old_startup).is_err();
            let hotkey_rollback_failed =
                hotkey_changed && self.hotkeys.replace(&old.hotkey).is_err();
            if startup_rollback_failed || hotkey_rollback_failed {
                return Err(SettingsError::RollbackFailed);
            }
            return Err(SettingsError::Persistence(error));
        }

        self.state.replace(desired);
        Ok(self.snapshot())
    }
}

/// Settings validation, platform, persistence, or rollback failure.
#[derive(Debug, Error)]
pub enum SettingsError {
    /// Hotkey syntax was invalid.
    #[error("invalid hotkey: {0}")]
    InvalidHotkey(qrforge_domain::HotkeyParseError),
    /// Operating-system registration rejected the new hotkey.
    #[error("hotkey registration failed: {0}")]
    HotkeyRegistration(PortError),
    /// Launch-at-startup registration failed.
    #[error("startup registration failed: {0}")]
    Startup(PortError),
    /// Atomic settings persistence failed.
    #[error("settings persistence failed: {0}")]
    Persistence(PortError),
    /// A compensating platform action failed after another operation failed.
    #[error("settings rollback failed; restart QRForge before retrying")]
    RollbackFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct Repository {
        saved: Mutex<Vec<AppSettings>>,
    }

    impl SettingsRepository for Repository {
        fn load(&self) -> Result<AppSettings, PortError> {
            Ok(AppSettings::default())
        }

        fn save(&self, settings: &AppSettings) -> Result<(), PortError> {
            self.saved
                .lock()
                .expect("repository mutex")
                .push(settings.clone());
            Ok(())
        }
    }

    struct Hotkeys {
        active: Mutex<Option<Hotkey>>,
        rejected: Hotkey,
    }

    impl HotkeyPort for Hotkeys {
        fn active(&self) -> Option<Hotkey> {
            self.active.lock().expect("hotkey mutex").clone()
        }

        fn replace(&self, requested: &Hotkey) -> Result<(), PortError> {
            if requested == &self.rejected {
                return Err(PortError::new("hotkey", "shortcut is already in use"));
            }
            *self.active.lock().expect("hotkey mutex") = Some(requested.clone());
            Ok(())
        }
    }

    struct Startup(Mutex<bool>);

    impl StartupPort for Startup {
        fn is_enabled(&self) -> Result<bool, PortError> {
            Ok(*self.0.lock().expect("startup mutex"))
        }

        fn set_enabled(&self, enabled: bool) -> Result<(), PortError> {
            *self.0.lock().expect("startup mutex") = enabled;
            Ok(())
        }
    }

    #[test]
    fn hotkey_conflict_leaves_previous_registration_and_settings_intact() {
        let rejected: Hotkey = "Ctrl+Alt+X".parse().expect("test hotkey");
        let repository = Arc::new(Repository::default());
        let hotkeys = Arc::new(Hotkeys {
            active: Mutex::new(Some(Hotkey::default())),
            rejected: rejected.clone(),
        });
        let state = Arc::new(SettingsState::new(AppSettings::default()));
        let service = SettingsService::new(
            repository.clone(),
            hotkeys.clone(),
            Arc::new(Startup(Mutex::new(false))),
            state.clone(),
        );

        let result = service.update(SettingsUpdate {
            hotkey: rejected.to_string(),
            launch_at_startup: false,
            auto_open_safe_urls: true,
            copy_non_url_payloads: true,
            notifications_enabled: true,
        });

        assert!(matches!(result, Err(SettingsError::HotkeyRegistration(_))));
        assert_eq!(hotkeys.active(), Some(Hotkey::default()));
        assert_eq!(state.get(), AppSettings::default());
        assert!(
            repository
                .saved
                .lock()
                .expect("repository mutex")
                .is_empty()
        );
    }
}
