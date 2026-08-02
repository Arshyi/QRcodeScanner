use thiserror::Error;

/// Pixel layouts accepted by the production decoder boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PixelFormat {
    /// Eight-bit red, green, blue, and alpha channels in byte order.
    Rgba8,
}

/// An in-memory screen capture.
///
/// The buffer is owned native memory and is never encoded or persisted by this
/// type. Dropping the value releases the pixels.
#[derive(Debug, Eq, PartialEq)]
pub struct CapturedFrame {
    width: u32,
    height: u32,
    format: PixelFormat,
    stride_bytes: usize,
    pixels: Vec<u8>,
    /// Optional diagnostic label for the monitor captured (e.g. "Primary", "DP-1").
    /// Not part of core frame validation; used for logging only.
    monitor_label: Option<String>,
    /// Optional Windows display scaling factor (e.g. 100, 125, 150, 200).
    /// Not part of core frame validation; used for logging only.
    scale_factor_percent: Option<u32>,
}

impl CapturedFrame {
    /// Creates a validated RGBA frame without copying the provided buffer.
    pub fn rgba8(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, FrameError> {
        let stride_bytes = usize::try_from(width)
            .ok()
            .and_then(|width| width.checked_mul(4))
            .ok_or(FrameError::DimensionsOverflow)?;
        Self::rgba8_strided(width, height, stride_bytes, pixels)
    }

    /// Creates a validated tightly packed RGBA frame with an explicit stride.
    ///
    /// The safe ZXing slice boundary currently supports only tightly packed
    /// rows. Adapters must repack padded native rows before constructing a
    /// frame instead of passing ambiguous memory across the decoder boundary.
    pub fn rgba8_strided(
        width: u32,
        height: u32,
        stride_bytes: usize,
        pixels: Vec<u8>,
    ) -> Result<Self, FrameError> {
        if width == 0 || height == 0 {
            return Err(FrameError::ZeroDimension);
        }
        let packed_stride = usize::try_from(width)
            .ok()
            .and_then(|width| width.checked_mul(4))
            .ok_or(FrameError::DimensionsOverflow)?;
        if stride_bytes != packed_stride {
            return Err(FrameError::UnsupportedStride {
                expected: packed_stride,
                actual: stride_bytes,
            });
        }
        let expected = usize::try_from(height)
            .ok()
            .and_then(|height| stride_bytes.checked_mul(height))
            .ok_or(FrameError::DimensionsOverflow)?;
        if pixels.len() != expected {
            return Err(FrameError::InvalidBufferLength {
                expected,
                actual: pixels.len(),
            });
        }
        Ok(Self {
            width,
            height,
            format: PixelFormat::Rgba8,
            stride_bytes,
            pixels,
            monitor_label: None,
            scale_factor_percent: None,
        })
    }

    /// Creates a validated RGBA frame with optional diagnostic metadata.
    pub fn rgba8_with_metadata(
        width: u32,
        height: u32,
        pixels: Vec<u8>,
        monitor_label: Option<String>,
        scale_factor_percent: Option<u32>,
    ) -> Result<Self, FrameError> {
        let mut frame = Self::rgba8(width, height, pixels)?;
        frame.monitor_label = monitor_label.filter(|label| !label.trim().is_empty());
        frame.scale_factor_percent = scale_factor_percent.filter(|factor| *factor > 0);
        Ok(frame)
    }

    /// Returns the frame width in physical pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the frame height in physical pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the frame pixel layout.
    #[must_use]
    pub const fn format(&self) -> PixelFormat {
        self.format
    }

    /// Returns the validated byte distance between adjacent physical rows.
    #[must_use]
    pub const fn stride_bytes(&self) -> usize {
        self.stride_bytes
    }

    /// Borrows the pixel buffer.
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Returns the optional non-sensitive monitor label supplied by capture.
    #[must_use]
    pub fn monitor_label(&self) -> Option<&str> {
        self.monitor_label.as_deref()
    }

    /// Returns the optional display scaling percentage supplied by capture.
    #[must_use]
    pub const fn scale_factor_percent(&self) -> Option<u32> {
        self.scale_factor_percent
    }
}

/// Validation failures for captured pixel buffers.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum FrameError {
    /// A decoder frame must have non-zero width and height.
    #[error("frame dimensions must be non-zero")]
    ZeroDimension,
    /// Width, height, and channel count overflowed the host address space.
    #[error("frame dimensions overflow the address space")]
    DimensionsOverflow,
    /// The supplied byte length does not match the declared dimensions.
    #[error("frame requires {expected} bytes but received {actual}")]
    InvalidBufferLength {
        /// Required byte count.
        expected: usize,
        /// Supplied byte count.
        actual: usize,
    },
    /// The frame uses row padding that must be removed by the adapter.
    #[error("RGBA stride must be {expected} bytes but received {actual}")]
    UnsupportedStride {
        /// Required tightly packed byte stride.
        expected: usize,
        /// Supplied byte stride.
        actual: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_rgba_buffer_length() {
        assert!(CapturedFrame::rgba8(2, 2, vec![0; 16]).is_ok());
        assert_eq!(
            CapturedFrame::rgba8(0, 2, Vec::new()),
            Err(FrameError::ZeroDimension)
        );
        assert_eq!(
            CapturedFrame::rgba8(2, 2, vec![0; 15]),
            Err(FrameError::InvalidBufferLength {
                expected: 16,
                actual: 15
            })
        );
        assert_eq!(
            CapturedFrame::rgba8_strided(2, 2, 12, vec![0; 24]),
            Err(FrameError::UnsupportedStride {
                expected: 8,
                actual: 12,
            })
        );
    }
}
