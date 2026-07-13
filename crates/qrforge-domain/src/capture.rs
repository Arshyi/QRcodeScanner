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
    pixels: Vec<u8>,
}

impl CapturedFrame {
    /// Creates a validated RGBA frame without copying the provided buffer.
    pub fn rgba8(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, FrameError> {
        let expected = usize::try_from(width)
            .ok()
            .and_then(|width| {
                usize::try_from(height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .and_then(|pixels| pixels.checked_mul(4))
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
            pixels,
        })
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

    /// Borrows the pixel buffer.
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

/// Validation failures for captured pixel buffers.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum FrameError {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_rgba_buffer_length() {
        assert!(CapturedFrame::rgba8(2, 2, vec![0; 16]).is_ok());
        assert_eq!(
            CapturedFrame::rgba8(2, 2, vec![0; 15]),
            Err(FrameError::InvalidBufferLength {
                expected: 16,
                actual: 15
            })
        );
    }
}
