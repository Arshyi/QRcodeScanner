/// A physical-pixel point in a captured frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Point {
    /// Horizontal coordinate.
    pub x: i32,
    /// Vertical coordinate.
    pub y: i32,
}

/// QR-family symbology reported by the decoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QrFormat {
    /// Standard QR Code, including model variants.
    QrCode,
    /// Micro QR Code.
    MicroQrCode,
    /// Rectangular Micro QR Code.
    RectangularMicroQrCode,
}

/// One typed QR detection returned by a decoder adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Detection {
    /// Exact decoded payload bytes where supplied by the engine.
    pub raw_bytes: Vec<u8>,
    /// UTF-8 interpretation when the payload is valid text.
    pub text: Option<String>,
    /// Detected QR-family format.
    pub format: QrFormat,
    /// Four corners in top-left, top-right, bottom-right, bottom-left order.
    pub corners: [Point; 4],
}

impl Detection {
    /// Constructs a detection while deriving its optional UTF-8 view.
    #[must_use]
    pub fn new(raw_bytes: Vec<u8>, format: QrFormat, corners: [Point; 4]) -> Self {
        let text = String::from_utf8(raw_bytes.clone()).ok();
        Self {
            raw_bytes,
            text,
            format,
            corners,
        }
    }
}
