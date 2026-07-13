//! One-shot screen-capture adapters.

use qrforge_application::{CapturePort, PortError};
use qrforge_domain::CapturedFrame;
use xcap::Monitor;

/// Phase 0.5-approved xcap adapter for one-shot primary-screen capture.
#[derive(Default)]
pub struct XcapCapture;

impl CapturePort for XcapCapture {
    fn capture_primary(&self) -> Result<CapturedFrame, PortError> {
        let monitors = Monitor::all()
            .map_err(|error| PortError::new("capture_enumeration", error.to_string()))?;
        let monitor = monitors
            .iter()
            .find(|candidate| candidate.is_primary().unwrap_or(false))
            .or_else(|| monitors.first())
            .ok_or_else(|| PortError::new("capture_enumeration", "no monitor was found"))?;
        let image = monitor
            .capture_image()
            .map_err(|error| PortError::new("capture_primary", error.to_string()))?;
        let (width, height) = image.dimensions();
        CapturedFrame::rgba8(width, height, image.into_raw())
            .map_err(|error| PortError::new("capture_primary", error.to_string()))
    }
}
