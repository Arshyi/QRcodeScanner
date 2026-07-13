use qrforge_application::{PortError, StartupPort};
use tauri::{AppHandle, Wry};
use tauri_plugin_autostart::ManagerExt;

/// Official Tauri launch-at-sign-in adapter.
pub struct TauriStartup {
    app: AppHandle<Wry>,
}

impl TauriStartup {
    /// Creates a startup adapter for the running host.
    #[must_use]
    pub fn new(app: AppHandle<Wry>) -> Self {
        Self { app }
    }
}

impl StartupPort for TauriStartup {
    fn is_enabled(&self) -> Result<bool, PortError> {
        self.app
            .autolaunch()
            .is_enabled()
            .map_err(|error| PortError::new("startup_query", error.to_string()))
    }

    fn set_enabled(&self, enabled: bool) -> Result<(), PortError> {
        let manager = self.app.autolaunch();
        if enabled {
            manager.enable()
        } else {
            manager.disable()
        }
        .map_err(|error| PortError::new("startup_update", error.to_string()))
    }
}
