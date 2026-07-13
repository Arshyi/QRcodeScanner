//! Atomic, versioned JSON settings storage.

use qrforge_application::{PortError, SettingsRepository};
use qrforge_domain::{AppSettings, Hotkey, SETTINGS_SCHEMA_VERSION};
use serde_json::{Map, Value};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

/// File-backed settings repository using same-directory atomic replacement.
pub struct FileSettingsRepository {
    path: PathBuf,
}

impl FileSettingsRepository {
    /// Creates a repository for the provided JSON document path.
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Returns the settings document path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn read_document(&self) -> Result<Option<Value>, PortError> {
        match fs::read(&self.path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|error| PortError::new("settings_read", error.to_string())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(PortError::new("settings_read", error.to_string())),
        }
    }
}

impl SettingsRepository for FileSettingsRepository {
    fn load(&self) -> Result<AppSettings, PortError> {
        let Some(document) = self.read_document()? else {
            return Ok(AppSettings::default());
        };
        let Some(object) = document.as_object() else {
            return Ok(AppSettings::default());
        };
        let version = object
            .get("schemaVersion")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(0);
        let settings = parse_fields(object, version);
        if version < SETTINGS_SCHEMA_VERSION {
            self.save(&settings)?;
        }
        Ok(settings)
    }

    fn save(&self, settings: &AppSettings) -> Result<(), PortError> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| PortError::new("settings_write", "settings path has no parent"))?;
        fs::create_dir_all(parent)
            .map_err(|error| PortError::new("settings_write", error.to_string()))?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)
            .map_err(|error| PortError::new("settings_write", error.to_string()))?;
        serde_json::to_writer_pretty(&mut temporary, settings)
            .map_err(|error| PortError::new("settings_write", error.to_string()))?;
        temporary
            .write_all(b"\n")
            .and_then(|()| temporary.as_file().sync_all())
            .map_err(|error| PortError::new("settings_write", error.to_string()))?;
        temporary
            .persist(&self.path)
            .map_err(|error| PortError::new("settings_write", error.error.to_string()))?;
        Ok(())
    }
}

fn parse_fields(object: &Map<String, Value>, version: u32) -> AppSettings {
    let defaults = AppSettings::default();
    let hotkey = string_field(object, "hotkey")
        .and_then(|value| value.parse::<Hotkey>().ok())
        .unwrap_or_else(|| defaults.hotkey.clone());
    let auto_open_key = if version == 0 {
        "autoOpen"
    } else {
        "autoOpenSafeUrls"
    };
    let copy_key = if version == 0 {
        "copyText"
    } else {
        "copyNonUrlPayloads"
    };
    let notifications_key = if version == 0 {
        "notifications"
    } else {
        "notificationsEnabled"
    };
    AppSettings {
        schema_version: SETTINGS_SCHEMA_VERSION,
        hotkey,
        launch_at_startup: bool_field(object, "launchAtStartup")
            .unwrap_or(defaults.launch_at_startup),
        auto_open_safe_urls: bool_field(object, auto_open_key)
            .or_else(|| bool_field(object, "autoOpenSafeUrls"))
            .unwrap_or(defaults.auto_open_safe_urls),
        copy_non_url_payloads: bool_field(object, copy_key)
            .or_else(|| bool_field(object, "copyNonUrlPayloads"))
            .unwrap_or(defaults.copy_non_url_payloads),
        notifications_enabled: bool_field(object, notifications_key)
            .or_else(|| bool_field(object, "notificationsEnabled"))
            .unwrap_or(defaults.notifications_enabled),
    }
}

fn bool_field(object: &Map<String, Value>, name: &str) -> Option<bool> {
    object.get(name).and_then(Value::as_bool)
}

fn string_field<'a>(object: &'a Map<String, Value>, name: &str) -> Option<&'a str> {
    object.get(name).and_then(Value::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn invalid_fields_fall_back_independently() {
        let directory = tempdir().expect("tempdir");
        let path = directory.path().join("settings.json");
        fs::write(
            &path,
            r#"{
              "schemaVersion": 1,
              "hotkey": "not valid",
              "launchAtStartup": true,
              "autoOpenSafeUrls": "yes",
              "copyNonUrlPayloads": false,
              "notificationsEnabled": 3
            }"#,
        )
        .expect("write fixture");
        let loaded = FileSettingsRepository::new(path)
            .load()
            .expect("load settings");
        assert_eq!(loaded.hotkey, Hotkey::default());
        assert!(loaded.launch_at_startup);
        assert!(loaded.auto_open_safe_urls);
        assert!(!loaded.copy_non_url_payloads);
        assert!(loaded.notifications_enabled);
    }

    #[test]
    fn migrates_version_zero_and_rewrites_current_schema() {
        let directory = tempdir().expect("tempdir");
        let path = directory.path().join("settings.json");
        fs::write(
            &path,
            r#"{
              "hotkey": "Ctrl+Alt+M",
              "autoOpen": false,
              "copyText": false,
              "notifications": false
            }"#,
        )
        .expect("write fixture");
        let repository = FileSettingsRepository::new(path.clone());
        let loaded = repository.load().expect("migrate settings");
        assert_eq!(loaded.schema_version, SETTINGS_SCHEMA_VERSION);
        assert_eq!(loaded.hotkey.to_string(), "Ctrl+Alt+M");
        assert!(!loaded.auto_open_safe_urls);
        let rewritten: Value = serde_json::from_slice(&fs::read(path).expect("read migrated file"))
            .expect("parse migrated file");
        assert_eq!(rewritten["schemaVersion"], SETTINGS_SCHEMA_VERSION);
        assert_eq!(rewritten["copyNonUrlPayloads"], false);
    }

    #[test]
    fn atomic_save_survives_reload() {
        let directory = tempdir().expect("tempdir");
        let path = directory.path().join("nested").join("settings.json");
        let repository = FileSettingsRepository::new(path);
        let mut settings = AppSettings::default();
        settings.notifications_enabled = false;
        repository.save(&settings).expect("atomic save");
        assert_eq!(repository.load().expect("reload"), settings);
    }
}
