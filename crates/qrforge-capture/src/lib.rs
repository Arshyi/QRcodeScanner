//! One-shot physical-display screen-capture adapter.

use qrforge_application::{CaptureOutput, CapturePort, PortError};
use qrforge_domain::{CapturedFrame, MonitorId, MonitorInfo};
use std::collections::BTreeMap;
use xcap::Monitor;

/// Phase 0.5-approved xcap adapter extended with explicit display selection.
#[derive(Default)]
pub struct XcapCapture;

impl CapturePort for XcapCapture {
    fn monitors(&self) -> Result<Vec<MonitorInfo>, PortError> {
        Ok(enumerate()?.into_iter().map(|entry| entry.info).collect())
    }

    fn capture(&self, requested: Option<&MonitorId>) -> Result<CaptureOutput, PortError> {
        let monitors = enumerate()?;
        let (index, used_fallback) = select_monitor_index(
            &monitors
                .iter()
                .map(|entry| entry.info.clone())
                .collect::<Vec<_>>(),
            requested,
        )
        .ok_or_else(|| PortError::new("capture_enumeration", "no monitor was found"))?;
        let selected = &monitors[index];
        let image = selected
            .native
            .capture_image()
            .map_err(|error| PortError::new("capture_selected_monitor", error.to_string()))?;
        let (width, height) = image.dimensions();
        if (width, height) != (selected.info.width, selected.info.height) {
            return Err(PortError::new(
                "capture_dimensions",
                "captured physical dimensions did not match the selected display",
            ));
        }
        let frame = CapturedFrame::rgba8_with_metadata(
            width,
            height,
            image.into_raw(),
            Some(selected.info.label.clone()),
            Some(selected.info.scale_factor_percent),
        )
        .map_err(|error| PortError::new("capture_selected_monitor", error.to_string()))?;
        Ok(CaptureOutput {
            frame,
            monitor: selected.info.clone(),
            used_fallback,
        })
    }
}

struct NativeMonitor {
    native: Monitor,
    info: MonitorInfo,
}

struct Candidate {
    native: Monitor,
    friendly_name: Option<String>,
    identity_base: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    scale_factor_percent: u32,
    rotation_degrees: u16,
    is_primary: bool,
}

fn enumerate() -> Result<Vec<NativeMonitor>, PortError> {
    let native =
        Monitor::all().map_err(|error| PortError::new("capture_enumeration", error.to_string()))?;
    let mut candidates = native
        .into_iter()
        .map(candidate)
        .collect::<Result<Vec<_>, _>>()?;
    candidates.sort_by_key(|candidate| {
        (
            candidate.x,
            candidate.y,
            candidate.width,
            candidate.height,
            candidate.friendly_name.clone(),
        )
    });

    let mut occurrences = BTreeMap::<String, usize>::new();
    let mut totals = BTreeMap::<String, usize>::new();
    for candidate in &candidates {
        *totals.entry(candidate.identity_base.clone()).or_default() += 1;
    }

    candidates
        .into_iter()
        .enumerate()
        .map(|(position, candidate)| {
            let occurrence = occurrences
                .entry(candidate.identity_base.clone())
                .or_default();
            *occurrence += 1;
            let suffix = if totals[&candidate.identity_base] > 1 {
                format!("-{}", *occurrence)
            } else {
                String::new()
            };
            let id = MonitorId::new(format!(
                "display-{:016x}{suffix}",
                stable_hash(candidate.identity_base.as_bytes())
            ))
            .map_err(|error| PortError::new("monitor_identifier", error.to_string()))?;
            let base_label = candidate
                .friendly_name
                .unwrap_or_else(|| format!("Display {}", position + 1));
            let primary = if candidate.is_primary {
                " — Primary"
            } else {
                ""
            };
            let label = format!(
                "{base_label} ({}×{}, {}%){primary}",
                candidate.width, candidate.height, candidate.scale_factor_percent
            );
            let info = MonitorInfo::new(
                id,
                label,
                candidate.x,
                candidate.y,
                candidate.width,
                candidate.height,
                candidate.scale_factor_percent,
                candidate.rotation_degrees,
                candidate.is_primary,
            )
            .map_err(|error| PortError::new("monitor_metadata", error.to_string()))?;
            Ok(NativeMonitor {
                native: candidate.native,
                info,
            })
        })
        .collect()
}

fn candidate(native: Monitor) -> Result<Candidate, PortError> {
    let raw_name = native.name().unwrap_or_default();
    let friendly_name = friendly_name(&raw_name);
    let x = native
        .x()
        .map_err(|error| PortError::new("monitor_x", error.to_string()))?;
    let y = native
        .y()
        .map_err(|error| PortError::new("monitor_y", error.to_string()))?;
    let width = native
        .width()
        .map_err(|error| PortError::new("monitor_width", error.to_string()))?;
    let height = native
        .height()
        .map_err(|error| PortError::new("monitor_height", error.to_string()))?;
    let scale_factor_percent = scale_percent(native.scale_factor().unwrap_or(1.0));
    let rotation_degrees = rotation(native.rotation().unwrap_or(0.0));
    let is_primary = native.is_primary().unwrap_or(false);
    let is_builtin = native.is_builtin().unwrap_or(false);
    let identity_base = format!(
        "{}|{width}x{height}|r{rotation_degrees}|builtin={is_builtin}",
        friendly_name.as_deref().unwrap_or("display")
    )
    .to_ascii_lowercase();
    Ok(Candidate {
        native,
        friendly_name,
        identity_base,
        x,
        y,
        width,
        height,
        scale_factor_percent,
        rotation_degrees,
        is_primary,
    })
}

fn friendly_name(raw: &str) -> Option<String> {
    let name = raw.trim();
    if name.is_empty() || name.to_ascii_lowercase().starts_with("unknown monitor ") {
        None
    } else {
        Some(name.chars().take(80).collect())
    }
}

fn scale_percent(scale: f32) -> u32 {
    if scale.is_finite() && scale > 0.0 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let percent = (scale * 100.0).round() as u32;
        percent.max(1)
    } else {
        100
    }
}

fn rotation(value: f32) -> u16 {
    if (45.0..135.0).contains(&value) {
        90
    } else if (135.0..225.0).contains(&value) {
        180
    } else if (225.0..315.0).contains(&value) {
        270
    } else {
        0
    }
}

fn stable_hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn select_monitor_index(
    monitors: &[MonitorInfo],
    requested: Option<&MonitorId>,
) -> Option<(usize, bool)> {
    if let Some(requested) = requested
        && let Some(index) = monitors.iter().position(|monitor| &monitor.id == requested)
    {
        return Some((index, false));
    }
    let index = monitors
        .iter()
        .position(|monitor| monitor.is_primary)
        .or_else(|| (!monitors.is_empty()).then_some(0))?;
    Some((index, requested.is_some()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor(id: &str, x: i32, y: i32, primary: bool) -> MonitorInfo {
        MonitorInfo::new(
            id.parse().expect("id"),
            format!("Display at {x},{y}"),
            x,
            y,
            1_920,
            1_080,
            125,
            0,
            primary,
        )
        .expect("monitor")
    }

    #[test]
    fn unavailable_selection_falls_back_to_primary() {
        let monitors = [
            monitor("left", -1_920, 0, false),
            monitor("primary", 0, 0, true),
        ];
        assert_eq!(
            select_monitor_index(&monitors, Some(&"missing".parse().expect("id"))),
            Some((1, true))
        );
        assert_eq!(
            select_monitor_index(&monitors, Some(&"left".parse().expect("id"))),
            Some((0, false))
        );
    }

    #[test]
    fn labels_hide_unstable_raw_handles_and_math_handles_dpi_rotation() {
        assert_eq!(friendly_name("Unknown Monitor 65537"), None);
        assert_eq!(friendly_name("  Color LCD  "), Some("Color LCD".to_owned()));
        assert_eq!(scale_percent(1.25), 125);
        assert_eq!(scale_percent(1.5), 150);
        assert_eq!(scale_percent(2.0), 200);
        assert_eq!(scale_percent(f32::NAN), 100);
        assert_eq!(rotation(90.0), 90);
        assert_eq!(rotation(270.0), 270);
    }

    #[test]
    fn stable_identifier_hash_is_deterministic() {
        assert_eq!(
            stable_hash(b"display|1920x1080|r0|builtin=false"),
            stable_hash(b"display|1920x1080|r0|builtin=false")
        );
        assert_ne!(stable_hash(b"display-a"), stable_hash(b"display-b"));
    }
}
