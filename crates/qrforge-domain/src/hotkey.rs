use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use std::{fmt, str::FromStr};
use thiserror::Error;

/// Supported non-modifier keys for a QRForge global hotkey.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HotkeyKey {
    /// An ASCII letter stored uppercase.
    Letter(char),
    /// An ASCII digit.
    Digit(char),
    /// A function key from F1 through F24.
    Function(u8),
}

impl fmt::Display for HotkeyKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Letter(value) | Self::Digit(value) => write!(formatter, "{value}"),
            Self::Function(value) => write!(formatter, "F{value}"),
        }
    }
}

/// A validated and canonically ordered global hotkey.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hotkey {
    control: bool,
    alt: bool,
    shift: bool,
    super_key: bool,
    key: HotkeyKey,
}

impl Hotkey {
    /// Creates a validated hotkey. At least one modifier is required.
    pub fn new(
        control: bool,
        alt: bool,
        shift: bool,
        super_key: bool,
        key: HotkeyKey,
    ) -> Result<Self, HotkeyParseError> {
        if !(control || alt || shift || super_key) {
            return Err(HotkeyParseError::ModifierRequired);
        }
        Ok(Self {
            control,
            alt,
            shift,
            super_key,
            key,
        })
    }

    /// Returns whether Control is required.
    #[must_use]
    pub const fn control(&self) -> bool {
        self.control
    }

    /// Returns whether Alt is required.
    #[must_use]
    pub const fn alt(&self) -> bool {
        self.alt
    }

    /// Returns whether Shift is required.
    #[must_use]
    pub const fn shift(&self) -> bool {
        self.shift
    }

    /// Returns whether the platform Super/Windows key is required.
    #[must_use]
    pub const fn super_key(&self) -> bool {
        self.super_key
    }

    /// Returns the non-modifier key.
    #[must_use]
    pub const fn key(&self) -> &HotkeyKey {
        &self.key
    }
}

impl Default for Hotkey {
    fn default() -> Self {
        Self::new(true, false, true, false, HotkeyKey::Letter('Q'))
            .expect("the built-in hotkey is valid")
    }
}

impl fmt::Display for Hotkey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::with_capacity(5);
        if self.control {
            parts.push("Ctrl".to_owned());
        }
        if self.alt {
            parts.push("Alt".to_owned());
        }
        if self.shift {
            parts.push("Shift".to_owned());
        }
        if self.super_key {
            parts.push("Super".to_owned());
        }
        parts.push(self.key.to_string());
        formatter.write_str(&parts.join("+"))
    }
}

impl FromStr for Hotkey {
    type Err = HotkeyParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() || value.trim() != value {
            return Err(HotkeyParseError::InvalidSyntax);
        }
        let mut control = false;
        let mut alt = false;
        let mut shift = false;
        let mut super_key = false;
        let mut key = None;

        for part in value.split('+') {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => set_once(&mut control)?,
                "alt" => set_once(&mut alt)?,
                "shift" => set_once(&mut shift)?,
                "super" | "win" | "meta" => set_once(&mut super_key)?,
                _ => {
                    if key.is_some() {
                        return Err(HotkeyParseError::MultipleKeys);
                    }
                    key = Some(parse_key(part)?);
                }
            }
        }

        Self::new(
            control,
            alt,
            shift,
            super_key,
            key.ok_or(HotkeyParseError::KeyRequired)?,
        )
    }
}

fn set_once(value: &mut bool) -> Result<(), HotkeyParseError> {
    if *value {
        return Err(HotkeyParseError::DuplicateModifier);
    }
    *value = true;
    Ok(())
}

fn parse_key(value: &str) -> Result<HotkeyKey, HotkeyParseError> {
    let upper = value.to_ascii_uppercase();
    let mut characters = upper.chars();
    if let (Some(character), None) = (characters.next(), characters.next()) {
        if character.is_ascii_alphabetic() {
            return Ok(HotkeyKey::Letter(character));
        }
        if character.is_ascii_digit() {
            return Ok(HotkeyKey::Digit(character));
        }
    }
    if let Some(number) = upper
        .strip_prefix('F')
        .and_then(|part| part.parse::<u8>().ok())
        && (1..=24).contains(&number)
    {
        return Ok(HotkeyKey::Function(number));
    }
    Err(HotkeyParseError::UnsupportedKey(value.to_owned()))
}

impl Serialize for Hotkey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Hotkey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
    }
}

/// Hotkey validation failure.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum HotkeyParseError {
    /// The representation is empty, padded, or otherwise malformed.
    #[error("invalid hotkey syntax")]
    InvalidSyntax,
    /// A modifier appears more than once.
    #[error("a hotkey modifier was repeated")]
    DuplicateModifier,
    /// More than one non-modifier key was supplied.
    #[error("a hotkey must contain exactly one key")]
    MultipleKeys,
    /// No non-modifier key was supplied.
    #[error("a hotkey key is required")]
    KeyRequired,
    /// At least one modifier is required to avoid capturing ordinary typing.
    #[error("at least one modifier is required")]
    ModifierRequired,
    /// The requested key is not in the supported portable subset.
    #[error("unsupported hotkey key: {0}")]
    UnsupportedKey(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_default_hotkey() {
        let parsed: Hotkey = "shift+control+q".parse().expect("hotkey should parse");
        assert_eq!(parsed, Hotkey::default());
        assert_eq!(parsed.to_string(), "Ctrl+Shift+Q");
    }

    #[test]
    fn rejects_unmodified_keys_and_duplicates() {
        assert_eq!(
            "Q".parse::<Hotkey>(),
            Err(HotkeyParseError::ModifierRequired)
        );
        assert_eq!(
            "Ctrl+Ctrl+Q".parse::<Hotkey>(),
            Err(HotkeyParseError::DuplicateModifier)
        );
    }

    #[test]
    fn serde_round_trip_uses_canonical_string() {
        let serialized = serde_json::to_string(&Hotkey::default()).expect("serialize");
        assert_eq!(serialized, "\"Ctrl+Shift+Q\"");
        let decoded: Hotkey = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(decoded, Hotkey::default());
    }
}
