//! Safe QRForge adapter over the statically bundled ZXing-C++ reader.
//!
//! All unsafe FFI implementation remains inside the third-party `zxing-cpp`
//! wrapper. This crate passes borrowed RGBA memory to its safe `ImageView` API.

use qrforge_application::{DecoderPort, PortError};
use qrforge_domain::{CapturedFrame, Detection, PixelFormat, Point, QrFormat};
use zxingcpp::{BarcodeFormat, BarcodeReader, ImageFormat, ImageView};

/// Production offline QR-family decoder selected by Phase 0.5 benchmarks.
#[derive(Default)]
pub struct ZxingDecoder;

impl DecoderPort for ZxingDecoder {
    fn decode(&self, frame: &CapturedFrame) -> Result<Vec<Detection>, PortError> {
        let image_format = match frame.format() {
            PixelFormat::Rgba8 => ImageFormat::RGBA,
        };
        let view =
            ImageView::from_slice(frame.pixels(), frame.width(), frame.height(), image_format)
                .map_err(|error| PortError::new("decode_image", error.to_string()))?;
        let reader = BarcodeReader::new()
            .formats([
                BarcodeFormat::QRCode,
                BarcodeFormat::MicroQRCode,
                BarcodeFormat::RMQRCode,
            ])
            .try_harder(true)
            .try_invert(true)
            .try_rotate(true)
            .try_downscale(true);
        let barcodes = reader
            .from(view)
            .map_err(|error| PortError::new("decode_qr", error.to_string()))?;
        Ok(barcodes
            .into_iter()
            .filter(zxingcpp::Barcode::is_valid)
            .map(|barcode| {
                let position = barcode.position();
                let format = match barcode.format() {
                    BarcodeFormat::MicroQRCode => QrFormat::MicroQrCode,
                    BarcodeFormat::RMQRCode => QrFormat::RectangularMicroQrCode,
                    _ => QrFormat::QrCode,
                };
                Detection::new(
                    barcode.bytes(),
                    format,
                    [
                        Point {
                            x: position.top_left.x,
                            y: position.top_left.y,
                        },
                        Point {
                            x: position.top_right.x,
                            y: position.top_right.y,
                        },
                        Point {
                            x: position.bottom_right.x,
                            y: position.bottom_right.y,
                        },
                        Point {
                            x: position.bottom_left.x,
                            y: position.bottom_left.y,
                        },
                    ],
                )
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> CapturedFrame {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../spikes/decoder-comparison/fixtures/generated")
            .join(name);
        let image = image::open(path).expect("fixture should load").to_rgba8();
        let (width, height) = image.dimensions();
        CapturedFrame::rgba8(width, height, image.into_raw()).expect("fixture dimensions")
    }

    #[test]
    fn regression_normal_fixture_preserves_bytes() {
        let results = ZxingDecoder
            .decode(&fixture("normal-screen.png"))
            .expect("decode normal fixture");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].raw_bytes, b"https://example.com/qrforge/normal");
    }

    #[test]
    fn regression_multi_fixture_returns_exact_set() {
        let mut payloads = ZxingDecoder
            .decode(&fixture("multiple.png"))
            .expect("decode multiple fixture")
            .into_iter()
            .map(|detection| detection.raw_bytes)
            .collect::<Vec<_>>();
        payloads.sort();
        assert_eq!(
            payloads,
            [
                b"https://example.org/multi-three".to_vec(),
                b"multi-one".to_vec(),
                b"multi-two".to_vec(),
            ]
        );
    }

    #[test]
    fn regression_inverted_and_false_positive_fixtures() {
        assert_eq!(
            ZxingDecoder
                .decode(&fixture("inverted.png"))
                .expect("decode inverted fixture")[0]
                .raw_bytes,
            b"inverted-code"
        );
        assert!(
            ZxingDecoder
                .decode(&fixture("false-positive.png"))
                .expect("decode false-positive fixture")
                .is_empty()
        );
    }
}
