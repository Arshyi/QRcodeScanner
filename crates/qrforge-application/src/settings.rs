use crate::{HotkeyPort, PortError, SettingsRepository, StartupPort};
use qrforge_domain::{AppSettings, Hotkey, SETTINGS_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, PoisonError, RwLock};
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
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn replace(&self, settings: AppSettings) {
        *self.0.write().unwrap_or_else(PoisonError::into_inner) = settings;
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
    pub fn update(&self, update: &SettingsUpdate) -> Result<SettingsSnapshot, SettingsError> {
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

    struct FlakyHotkeys {
        active: Mutex<Option<Hotkey>>,
        reject_next: Mutex<bool>,
    }

    impl HotkeyPort for FlakyHotkeys {
        fn active(&self) -> Option<Hotkey> {
            self.active.lock().expect("hotkey mutex").clone()
        }

        fn replace(&self, requested: &Hotkey) -> Result<(), PortError> {
            if *self.reject_next.lock().expect("reject mutex") {
                *self.reject_next.lock().expect("reject mutex") = false;
                return Err(PortError::new("hotkey", "transient registration failure"));
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

    struct FlakyStartup {
        set_should_fail: Mutex<bool>,
    }

    impl StartupPort for FlakyStartup {
        fn is_enabled(&self) -> Result<bool, PortError> {
            Ok(false)
        }

        fn set_enabled(&self, _enabled: bool) -> Result<(), PortError> {
            if *self.set_should_fail.lock().expect("startup mutex") {
                *self.set_should_fail.lock().expect("startup mutex") = false;
                return Err(PortError::new("startup", "registration refused"));
            }
            Ok(())
        }
    }

    struct FlakyRepository {
        save_should_fail: Mutex<bool>,
    }

    impl SettingsRepository for FlakyRepository {
        fn load(&self) -> Result<AppSettings, PortError> {
            Ok(AppSettings::default())
        }

        fn save(&self, _settings: &AppSettings) -> Result<(), PortError> {
            if *self.save_should_fail.lock().expect("save mutex") {
                *self.save_should_fail.lock().expect("save mutex") = false;
                return Err(PortError::new("repository", "disk full"));
            }
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

        let result = service.update(&SettingsUpdate {
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

    #[test]
    fn startup_registration_failure_rolls_back_hotkey() {
        let repository = Arc::new(Repository::default());
        // The hotkey adapter will accept "Ctrl+Alt+Y" but reject the default
        // registration that the rollback would reapply, simulating a
        // permanent conflict on the previous shortcut.
        let hotkeys = Arc::new(Hotkeys {
            active: Mutex::new(Some(Hotkey::default())),
            rejected: Hotkey::default(),
        });
        let startup = Arc::new(FlakyStartup {
            set_should_fail: Mutex::new(true),
        });
        let state = Arc::new(SettingsState::new(AppSettings::default()));
        let service =
            SettingsService::new(repository.clone(), hotkeys.clone(), startup, state.clone());

        let result = service.update(&SettingsUpdate {
            hotkey: "Ctrl+Alt+Y".to_owned(),
            launch_at_startup: true,
            auto_open_safe_urls: true,
            copy_non_url_payloads: true,
            notifications_enabled: true,
        });

        // Rollback re-registering the default hotkey is rejected, so the
        // service reports the more severe rollback failure.
        assert!(matches!(result, Err(SettingsError::RollbackFailed)));
        // The in-memory state still reflects the original settings because
        // no successful transaction completed.
        assert_eq!(state.get(), AppSettings::default());
    }

    #[test]
    fn startup_failure_with_recoverable_hotkey_rolls_back() {
        let repository = Arc::new(Repository::default());
        // Use a FlakyHotkeys that will accept both the new and old hotkey,
        // so the rollback can succeed and the service reports the original
        // startup error.
        let hotkeys = Arc::new(FlakyHotkeys {
            active: Mutex::new(Some(Hotkey::default())),
            reject_next: Mutex::new(false),
        });
        let startup = Arc::new(FlakyStartup {
            set_should_fail: Mutex::new(true),
        });
        let state = Arc::new(SettingsState::new(AppSettings::default()));
        let service =
            SettingsService::new(repository.clone(), hotkeys.clone(), startup, state.clone());

        let result = service.update(&SettingsUpdate {
            hotkey: "Ctrl+Alt+Y".to_owned(),
            launch_at_startup: true,
            auto_open_safe_urls: true,
            copy_non_url_payloads: true,
            notifications_enabled: true,
        });

        assert!(matches!(result, Err(SettingsError::Startup(_))));
        assert_eq!(hotkeys.active(), Some(Hotkey::default()));
        assert_eq!(state.get(), AppSettings::default());
    }

    #[test]
    fn persistence_failure_rolls_back_hotkey_and_startup() {
        let repository = Arc::new(FlakyRepository {
            save_should_fail: Mutex::new(true),
        });
        let hotkeys = Arc::new(FlakyHotkeys {
            active: Mutex::new(Some(Hotkey::default())),
            reject_next: Mutex::new(false),
        });
        let startup = Arc::new(FlakyStartup {
            set_should_fail: Mutex::new(false),
        });
        let state = Arc::new(SettingsState::new(AppSettings::default()));
        let service = SettingsService::new(repository, hotkeys.clone(), startup, state.clone());

        let result = service.update(&SettingsUpdate {
            hotkey: "Ctrl+Alt+Z".to_owned(),
            launch_at_startup: true,
            auto_open_safe_urls: true,
            copy_non_url_payloads: true,
            notifications_enabled: true,
        });

        assert!(matches!(result, Err(SettingsError::Persistence(_))));
        assert_eq!(hotkeys.active(), Some(Hotkey::default()));
        assert_eq!(state.get(), AppSettings::default());
    }

    #[test]
    fn default_hotkey_is_ctrl_shift_q() {
        assert_eq!(Hotkey::default().to_string(), "Ctrl+Shift+Q");
        assert_eq!(AppSettings::default().hotkey, Hotkey::default());
    }

    #[test]
    fn settings_snapshot_reports_registered_state() {
        let repository = Arc::new(Repository::default());
        let hotkeys = Arc::new(Hotkeys {
            active: Mutex::new(Some(Hotkey::default())),
            rejected: Hotkey::default(),
        });
        let state = Arc::new(SettingsState::new(AppSettings::default()));
        let service = SettingsService::new(
            repository,
            hotkeys.clone(),
            Arc::new(Startup(Mutex::new(false))),
            state,
        );

        let snapshot = service.snapshot();
        assert!(snapshot.hotkey_registered);
        assert_eq!(snapshot.active_hotkey.as_deref(), Some("Ctrl+Shift+Q"));
    }
}
