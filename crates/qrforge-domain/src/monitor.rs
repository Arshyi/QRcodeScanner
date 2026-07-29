use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use std::{fmt, str::FromStr};
use thiserror::Error;

/// Maximum persisted monitor identifier length accepted at trust boundaries.
pub const MAX_MONITOR_ID_LEN: usize = 96;

/// Stable, opaque identifier for a physical display where the capture adapter
/// can derive one.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MonitorId(String);

impl MonitorId {
    /// Creates a validated opaque monitor identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, MonitorIdError> {
        let value = value.into();
        if value.is_empty() || value.len() > MAX_MONITOR_ID_LEN {
            return Err(MonitorIdError::InvalidLength);
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
        {
            return Err(MonitorIdError::InvalidCharacter);
        }
        Ok(Self(value))
    }

    /// Returns the opaque identifier value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MonitorId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for MonitorId {
    type Err = MonitorIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl Serialize for MonitorId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for MonitorId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

/// A native display available for one-shot capture.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    /// Opaque stable identifier used in persisted settings.
    pub id: MonitorId,
    /// Human-readable label that excludes raw native handles.
    pub label: String,
    /// Physical-pixel horizontal origin in the virtual desktop.
    pub x: i32,
    /// Physical-pixel vertical origin in the virtual desktop.
    pub y: i32,
    /// Physical-pixel width after orientation is applied.
    pub width: u32,
    /// Physical-pixel height after orientation is applied.
    pub height: u32,
    /// Effective Windows scale as an integer percentage.
    pub scale_factor_percent: u32,
    /// Clockwise display rotation in degrees.
    pub rotation_degrees: u16,
    /// Whether Windows identifies this as the primary display.
    pub is_primary: bool,
}

impl MonitorInfo {
    /// Validates physical display metadata returned by a native adapter.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: MonitorId,
        label: impl Into<String>,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        scale_factor_percent: u32,
        rotation_degrees: u16,
        is_primary: bool,
    ) -> Result<Self, MonitorInfoError> {
        let label = label.into();
        let label = label.trim().to_owned();
        if label.is_empty() {
            return Err(MonitorInfoError::EmptyLabel);
        }
        if width == 0 || height == 0 {
            return Err(MonitorInfoError::ZeroDimension);
        }
        if scale_factor_percent == 0 {
            return Err(MonitorInfoError::ZeroScale);
        }
        if !matches!(rotation_degrees, 0 | 90 | 180 | 270) {
            return Err(MonitorInfoError::InvalidRotation(rotation_degrees));
        }
        Ok(Self {
            id,
            label,
            x,
            y,
            width,
            height,
            scale_factor_percent,
            rotation_degrees,
            is_primary,
        })
    }
}

/// Validation failure for an opaque monitor identifier.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum MonitorIdError {
    /// Identifier is empty or exceeds the trust-boundary limit.
    #[error("monitor identifier has an invalid length")]
    InvalidLength,
    /// Identifier contains a character outside the portable opaque subset.
    #[error("monitor identifier contains an invalid character")]
    InvalidCharacter,
}

/// Validation failure for native display metadata.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum MonitorInfoError {
    /// A user-facing display label must not be empty.
    #[error("monitor label must not be empty")]
    EmptyLabel,
    /// Physical capture dimensions must be non-zero.
    #[error("monitor dimensions must be non-zero")]
    ZeroDimension,
    /// Display scale must be a positive percentage.
    #[error("monitor scale must be positive")]
    ZeroScale,
    /// Windows display orientation must be a right-angle rotation.
    #[error("unsupported monitor rotation: {0}")]
    InvalidRotation(u16),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_metadata_preserves_negative_virtual_coordinates() {
        let monitor = MonitorInfo::new(
            "display-a".parse().expect("id"),
            "Portrait display".to_owned(),
            -1_440,
            -320,
            1_440,
            2_560,
            200,
            90,
            false,
        )
        .expect("monitor");
        assert_eq!((monitor.x, monitor.y), (-1_440, -320));
        assert_eq!((monitor.width, monitor.height), (1_440, 2_560));
        assert_eq!(monitor.scale_factor_percent, 200);
        assert_eq!(monitor.rotation_degrees, 90);
    }

    #[test]
    fn monitor_ids_reject_unbounded_or_path_like_values() {
        assert!("display:abc-123".parse::<MonitorId>().is_ok());
        assert_eq!(
            "../display".parse::<MonitorId>(),
            Err(MonitorIdError::InvalidCharacter)
        );
        assert_eq!(
            "x".repeat(MAX_MONITOR_ID_LEN + 1).parse::<MonitorId>(),
            Err(MonitorIdError::InvalidLength)
        );
    }
}
