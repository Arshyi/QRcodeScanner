use image::{DynamicImage, GrayImage, Luma, Rgb, RgbImage, imageops};
use memory_stats::memory_stats;
use qrcode::{EcLevel, QrCode};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use zxingcpp::{BarcodeFormat, BarcodeReader};

const ITERATIONS: usize = 30;

#[derive(Clone)]
struct Fixture {
    name: &'static str,
    category: &'static str,
    saved_image: DynamicImage,
    benchmark_image: GrayImage,
    expected: BTreeSet<Vec<u8>>,
}

#[derive(Serialize)]
struct Stats {
    min: f64,
    median: f64,
    p95: f64,
    max: f64,
}

#[derive(Serialize)]
struct CategoryResult {
    fixture: String,
    category: String,
    expected_codes: usize,
    detected_codes: usize,
    correct_iterations: usize,
    iterations: usize,
    latency_ms: Stats,
    failure: Option<String>,
}

#[derive(Serialize)]
struct EngineReport {
    engine: &'static str,
    categories_passed: usize,
    categories_total: usize,
    aggregate_latency_ms: Stats,
    physical_memory_before_mib: Option<f64>,
    physical_memory_after_mib: Option<f64>,
    physical_memory_delta_mib: Option<f64>,
    results: Vec<CategoryResult>,
}

#[derive(Serialize)]
struct Report {
    spike: &'static str,
    corpus_path: String,
    fixtures: usize,
    iterations_per_fixture: usize,
    identical_inputs: bool,
    qr_family_only: bool,
    accepted_zxing_failures: Vec<&'static str>,
    quircs: EngineReport,
    zxing_cpp: EngineReport,
    zxing_build: ZxingBuildCost,
}

fn grayscale_fixture(
    name: &'static str,
    category: &'static str,
    image: GrayImage,
    expected: BTreeSet<Vec<u8>>,
) -> Fixture {
    Fixture {
        name,
        category,
        saved_image: DynamicImage::ImageLuma8(image.clone()),
        benchmark_image: image,
        expected,
    }
}

fn color_fixture(
    name: &'static str,
    category: &'static str,
    image: RgbImage,
    expected: BTreeSet<Vec<u8>>,
) -> Fixture {
    let saved_image = DynamicImage::ImageRgb8(image);
    let benchmark_image = saved_image.to_luma8();
    Fixture {
        name,
        category,
        saved_image,
        benchmark_image,
        expected,
    }
}

#[derive(Serialize)]
struct ZxingBuildCost {
    license: &'static str,
    linkage: &'static str,
    compiler_requirement: &'static str,
    ffi_boundary: &'static str,
}

fn qr(payload: &[u8], scale: u32) -> GrayImage {
    QrCode::with_error_correction_level(payload, EcLevel::Q)
        .expect("fixture payload must encode")
        .render::<Luma<u8>>()
        .quiet_zone(true)
        .module_dimensions(scale, scale)
        .build()
}

fn canvas(width: u32, height: u32, value: u8) -> GrayImage {
    GrayImage::from_pixel(width, height, Luma([value]))
}

fn centered(background: &mut GrayImage, foreground: &GrayImage) {
    let x = i64::from((background.width() - foreground.width()) / 2);
    let y = i64::from((background.height() - foreground.height()) / 2);
    imageops::overlay(background, foreground, x, y);
}

fn expected(payloads: &[&[u8]]) -> BTreeSet<Vec<u8>> {
    payloads.iter().map(|payload| payload.to_vec()).collect()
}

fn perspective_like(source: &GrayImage) -> GrayImage {
    let output_width = source.width() + source.height() / 3;
    let mut output = canvas(output_width, source.height(), 255);
    for y in 0..source.height() {
        let inset = (source.height() - y) / 6;
        let target_width = source.width().saturating_sub(inset * 2).max(1);
        let row = imageops::crop_imm(source, 0, y, source.width(), 1).to_image();
        let resized = imageops::resize(&row, target_width, 1, imageops::FilterType::Nearest);
        let x = (output_width - target_width) / 2 + y / 8;
        imageops::overlay(&mut output, &resized, i64::from(x), i64::from(y));
    }
    output
}

#[allow(clippy::too_many_lines)]
fn corpus() -> Vec<Fixture> {
    let normal_payload = b"https://example.com/qrforge/normal";
    let plain_text_payload = b"hello from QRForge";
    let unicode_payload = "QRForge: こんにちは • Привет • مرحبا".as_bytes();
    let binary_payload: &[u8] = &[0x00, 0x01, 0x7f, 0x80, 0xfe, 0xff, b'Q', b'R'];
    let unusual_url = b"javascript:alert('qrforge-test')";
    let malformed_url = b"https://[invalid";

    let mut normal = canvas(1280, 720, 238);
    for y in 0..normal.height() {
        let shade = 225 + u8::try_from(y % 25).expect("modulo fits u8");
        for x in 0..normal.width() {
            normal.put_pixel(x, y, Luma([shade]));
        }
    }
    centered(&mut normal, &qr(normal_payload, 8));

    let multi_payloads: [&[u8]; 3] = [
        b"multi-one",
        b"multi-two",
        b"https://example.org/multi-three",
    ];
    let mut multiple = canvas(1920, 1080, 245);
    imageops::overlay(&mut multiple, &qr(multi_payloads[0], 6), 80, 80);
    imageops::overlay(&mut multiple, &qr(multi_payloads[1], 5), 780, 180);
    imageops::overlay(&mut multiple, &qr(multi_payloads[2], 7), 1240, 560);

    let rotated_symbol = imageops::rotate90(&qr(b"rotated-90-degrees", 7));
    let mut rotated = canvas(1000, 800, 255);
    centered(&mut rotated, &rotated_symbol);

    let mut inverted_symbol = qr(b"inverted-code", 8);
    imageops::invert(&mut inverted_symbol);
    let mut inverted = canvas(1000, 800, 0);
    centered(&mut inverted, &inverted_symbol);

    let mut low_contrast_symbol = qr(b"low-contrast-code", 8);
    for pixel in low_contrast_symbol.pixels_mut() {
        pixel.0[0] = if pixel.0[0] < 128 { 90 } else { 185 };
    }
    let mut low_contrast = canvas(1000, 800, 185);
    centered(&mut low_contrast, &low_contrast_symbol);

    let blurred_symbol = imageops::blur(&qr(b"blurred-code", 8), 1.6);
    let mut blurred = canvas(1000, 800, 255);
    centered(&mut blurred, &blurred_symbol);

    let mut damaged_symbol = qr(b"partially-damaged-code", 9);
    let damage_y = damaged_symbol.height() * 2 / 3;
    for y in damage_y..(damage_y + 10).min(damaged_symbol.height()) {
        for x in damaged_symbol.width() / 3..damaged_symbol.width() * 2 / 3 {
            damaged_symbol.put_pixel(x, y, Luma([255]));
        }
    }
    let mut damaged = canvas(1100, 850, 255);
    centered(&mut damaged, &damaged_symbol);

    let perspective_symbol = perspective_like(&qr(b"perspective-distorted", 8));
    let mut perspective = canvas(1200, 900, 255);
    centered(&mut perspective, &perspective_symbol);

    let mut small_high_res = canvas(2880, 1800, 242);
    let small_symbol = qr(b"small-in-high-resolution", 2);
    imageops::overlay(&mut small_high_res, &small_symbol, 2340, 1420);

    let mut unicode = canvas(1200, 850, 255);
    centered(&mut unicode, &qr(unicode_payload, 7));

    let mut plain_text = canvas(1100, 800, 255);
    centered(&mut plain_text, &qr(plain_text_payload, 8));

    let mut binary = canvas(1000, 800, 255);
    centered(&mut binary, &qr(binary_payload, 9));

    let mut unusual = canvas(1100, 800, 250);
    centered(&mut unusual, &qr(unusual_url, 8));

    let mut malformed = canvas(1100, 800, 250);
    centered(&mut malformed, &qr(malformed_url, 8));

    let downscaled_source = qr(b"downscaled-screen-code", 12);
    let downscaled_symbol = imageops::resize(
        &downscaled_source,
        downscaled_source.width() / 3,
        downscaled_source.height() / 3,
        imageops::FilterType::Lanczos3,
    );
    let mut downscaled = canvas(960, 720, 247);
    centered(&mut downscaled, &downscaled_symbol);

    let mut browser_rendered = canvas(1365, 768, 234);
    for y in 0..74 {
        for x in 0..browser_rendered.width() {
            browser_rendered.put_pixel(x, y, Luma([if y < 42 { 214 } else { 245 }]));
        }
    }
    for x in 112..870 {
        for y in 12..34 {
            browser_rendered.put_pixel(x, y, Luma([250]));
        }
    }
    let browser_symbol = qr(b"https://example.com/browser-rendered", 7);
    let mut browser_card = canvas(
        browser_symbol.width() + 88,
        browser_symbol.height() + 88,
        255,
    );
    centered(&mut browser_card, &browser_symbol);
    imageops::overlay(&mut browser_rendered, &browser_card, 760, 180);

    let compressed_source = imageops::blur(&qr(b"screenshot-compressed-code", 8), 0.65);
    let mut screenshot_compressed = canvas(1280, 720, 243);
    centered(&mut screenshot_compressed, &compressed_source);
    for pixel in screenshot_compressed.pixels_mut() {
        pixel.0[0] = (pixel.0[0] / 12) * 12;
    }

    let color_symbol = qr(b"colored-foreground-background", 8);
    let mut colored = RgbImage::from_pixel(1200, 850, Rgb([232, 244, 237]));
    let color_x = (colored.width() - color_symbol.width()) / 2;
    let color_y = (colored.height() - color_symbol.height()) / 2;
    for (x, y, pixel) in color_symbol.enumerate_pixels() {
        let rgb = if pixel.0[0] < 128 {
            Rgb([15, 76, 134])
        } else {
            Rgb([250, 232, 174])
        };
        colored.put_pixel(color_x + x, color_y + y, rgb);
    }

    let mut high_dpi = canvas(3840, 2160, 248);
    let high_dpi_symbol = qr(b"high-dpi-rendering-200-percent", 14);
    centered(&mut high_dpi, &high_dpi_symbol);

    let mut dense_ui = canvas(1600, 1000, 239);
    for row in 0..12 {
        let y = 45 + row * 72;
        for column in 0..6 {
            let x = 42 + column * 250;
            let shade = if (row + column) % 2 == 0 { 72 } else { 160 };
            for py in y..(y + 18) {
                for px in x..(x + 170) {
                    dense_ui.put_pixel(px, py, Luma([shade]));
                }
            }
        }
    }
    let dense_symbol = qr(b"dense-ui-background-code", 8);
    let mut dense_card = canvas(dense_symbol.width() + 72, dense_symbol.height() + 72, 255);
    centered(&mut dense_card, &dense_symbol);
    imageops::overlay(&mut dense_ui, &dense_card, 1040, 600);

    let mut false_positive = canvas(1280, 720, 248);
    for y in (40..680).step_by(48) {
        for x in (40..1240).step_by(48) {
            if (x / 48 + y / 48) % 3 == 0 {
                for py in y..(y + 22) {
                    for px in x..(x + 22) {
                        false_positive.put_pixel(px, py, Luma([15]));
                    }
                }
            }
        }
    }

    vec![
        grayscale_fixture(
            "normal-screen",
            "normal_screen",
            normal,
            expected(&[normal_payload]),
        ),
        grayscale_fixture("multiple", "multiple", multiple, expected(&multi_payloads)),
        grayscale_fixture(
            "rotated",
            "rotated",
            rotated,
            expected(&[b"rotated-90-degrees"]),
        ),
        grayscale_fixture(
            "inverted",
            "inverted",
            inverted,
            expected(&[b"inverted-code"]),
        ),
        grayscale_fixture(
            "low-contrast",
            "low_contrast",
            low_contrast,
            expected(&[b"low-contrast-code"]),
        ),
        grayscale_fixture("blurred", "blurred", blurred, expected(&[b"blurred-code"])),
        grayscale_fixture(
            "damaged",
            "partially_obscured",
            damaged,
            expected(&[b"partially-damaged-code"]),
        ),
        grayscale_fixture(
            "perspective",
            "perspective_distorted",
            perspective,
            expected(&[b"perspective-distorted"]),
        ),
        grayscale_fixture(
            "small-high-res",
            "small_high_resolution",
            small_high_res,
            expected(&[b"small-in-high-resolution"]),
        ),
        grayscale_fixture("unicode", "unicode", unicode, expected(&[unicode_payload])),
        grayscale_fixture(
            "plain-text",
            "plain_text",
            plain_text,
            expected(&[plain_text_payload]),
        ),
        grayscale_fixture("binary", "binary", binary, expected(&[binary_payload])),
        grayscale_fixture(
            "unusual-url",
            "dangerous_scheme",
            unusual,
            expected(&[unusual_url]),
        ),
        grayscale_fixture(
            "malformed-url",
            "malformed_url_like_text",
            malformed,
            expected(&[malformed_url]),
        ),
        grayscale_fixture(
            "downscaled",
            "downscaled",
            downscaled,
            expected(&[b"downscaled-screen-code"]),
        ),
        grayscale_fixture(
            "browser-rendered",
            "browser_rendered",
            browser_rendered,
            expected(&[b"https://example.com/browser-rendered"]),
        ),
        grayscale_fixture(
            "screenshot-compressed",
            "screenshot_compressed",
            screenshot_compressed,
            expected(&[b"screenshot-compressed-code"]),
        ),
        color_fixture(
            "colored",
            "colored_foreground_background",
            colored,
            expected(&[b"colored-foreground-background"]),
        ),
        grayscale_fixture(
            "high-dpi",
            "high_dpi_rendering",
            high_dpi,
            expected(&[b"high-dpi-rendering-200-percent"]),
        ),
        grayscale_fixture(
            "dense-ui",
            "dense_ui_background",
            dense_ui,
            expected(&[b"dense-ui-background-code"]),
        ),
        grayscale_fixture(
            "false-positive",
            "no_code_false_positive",
            false_positive,
            BTreeSet::new(),
        ),
    ]
}

fn write_corpus(fixtures: &[Fixture], output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    for fixture in fixtures {
        fixture
            .saved_image
            .save(output.join(format!("{}.png", fixture.name)))?;
    }
    Ok(())
}

fn decode_quircs(image: &GrayImage) -> BTreeSet<Vec<u8>> {
    let mut decoder = quircs::Quirc::default();
    decoder.resize(image.width() as usize, image.height() as usize);
    decoder
        .identify(
            image.width() as usize,
            image.height() as usize,
            image.as_raw(),
        )
        .filter_map(Result::ok)
        .filter_map(|code| code.decode().ok())
        .map(|decoded| decoded.payload)
        .collect()
}

fn benchmark_engine<F>(engine: &'static str, fixtures: &[Fixture], mut decode: F) -> EngineReport
where
    F: FnMut(&GrayImage) -> BTreeSet<Vec<u8>>,
{
    let memory_before = memory_stats().map(|stats| stats.physical_mem);
    let mut all_timings = Vec::with_capacity(fixtures.len() * ITERATIONS);
    let mut results = Vec::with_capacity(fixtures.len());

    for fixture in fixtures {
        let _ = decode(&fixture.benchmark_image);
        let mut timings = Vec::with_capacity(ITERATIONS);
        let mut latest = BTreeSet::new();
        let mut correct = 0;
        for _ in 0..ITERATIONS {
            let started = Instant::now();
            latest = decode(&fixture.benchmark_image);
            let elapsed = started.elapsed().as_secs_f64() * 1_000.0;
            timings.push(elapsed);
            all_timings.push(elapsed);
            if latest == fixture.expected {
                correct += 1;
            }
        }
        let failure = (correct != ITERATIONS).then(|| {
            format!(
                "expected {} code(s), detected {}; correct {correct}/{ITERATIONS}",
                fixture.expected.len(),
                latest.len()
            )
        });
        results.push(CategoryResult {
            fixture: format!("{}.png", fixture.name),
            category: fixture.category.to_owned(),
            expected_codes: fixture.expected.len(),
            detected_codes: latest.len(),
            correct_iterations: correct,
            iterations: ITERATIONS,
            latency_ms: summarize(timings),
            failure,
        });
    }

    let memory_after = memory_stats().map(|stats| stats.physical_mem);
    let categories_passed = results
        .iter()
        .filter(|result| result.failure.is_none())
        .count();
    EngineReport {
        engine,
        categories_passed,
        categories_total: results.len(),
        aggregate_latency_ms: summarize(all_timings),
        physical_memory_before_mib: memory_before.map(to_mib),
        physical_memory_after_mib: memory_after.map(to_mib),
        physical_memory_delta_mib: memory_before
            .zip(memory_after)
            .map(|(before, after)| to_mib(after.saturating_sub(before))),
        results,
    }
}

fn summarize(mut values: Vec<f64>) -> Stats {
    values.sort_by(f64::total_cmp);
    let last = values.len() - 1;
    Stats {
        min: values[0],
        median: values[last.div_ceil(2)],
        p95: values[last.saturating_mul(95).div_ceil(100)],
        max: values[last],
    }
}

fn to_mib(bytes: usize) -> f64 {
    let kib = u32::try_from(bytes / 1_024).unwrap_or(u32::MAX);
    f64::from(kib) / 1_024.0
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let corpus_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generated");
    let fixtures = corpus();
    write_corpus(&fixtures, &corpus_path)?;

    let quircs = benchmark_engine("quircs 0.10.3", &fixtures, decode_quircs);
    let reader = BarcodeReader::new()
        .formats(BarcodeFormat::QRCode)
        .try_harder(true)
        .try_invert(true)
        .try_rotate(true)
        .try_downscale(true);
    let zxing_cpp = benchmark_engine("ZXing-C++ 3.x via zxing-cpp 0.5.2", &fixtures, |image| {
        reader
            .from(image)
            .unwrap_or_default()
            .into_iter()
            .filter(zxingcpp::Barcode::is_valid)
            .map(|barcode| barcode.bytes())
            .collect()
    });

    let report = Report {
        spike: "real-world-decoder-comparison",
        corpus_path: "spikes/decoder-comparison/fixtures/generated".to_owned(),
        fixtures: fixtures.len(),
        iterations_per_fixture: ITERATIONS,
        identical_inputs: true,
        qr_family_only: true,
        accepted_zxing_failures: vec!["perspective.png"],
        quircs,
        zxing_cpp,
        zxing_build: ZxingBuildCost {
            license: "Apache-2.0",
            linkage: "bundled static C++ core through Rust C-API wrapper",
            compiler_requirement: "C++20-capable compiler; MSVC Build Tools on Windows",
            ffi_boundary: "unsafe implementation is contained in the third-party zxing-cpp wrapper",
        },
    };
    let serialized = serde_json::to_string_pretty(&report)?;
    if let Ok(report_path) = std::env::var("QRFORGE_REPORT_PATH") {
        let report_path = PathBuf::from(report_path);
        if let Some(parent) = report_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(report_path, &serialized)?;
    }
    println!("{serialized}");

    let accepted = report
        .accepted_zxing_failures
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let actual = report
        .zxing_cpp
        .results
        .iter()
        .filter(|result| result.failure.is_some())
        .map(|result| result.fixture.as_str())
        .collect::<BTreeSet<_>>();
    if actual != accepted {
        return Err(format!(
            "ZXing fixture failures changed: accepted {accepted:?}, actual {actual:?}"
        )
        .into());
    }
    Ok(())
}
