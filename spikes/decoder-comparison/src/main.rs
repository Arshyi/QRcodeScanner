use image::{GrayImage, Luma, imageops};
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
    image: GrayImage,
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
    quircs: EngineReport,
    zxing_cpp: EngineReport,
    zxing_build: ZxingBuildCost,
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
    let unicode_payload = "QRForge: こんにちは • Привет • مرحبا".as_bytes();
    let binary_payload: &[u8] = &[0x00, 0x01, 0x7f, 0x80, 0xfe, 0xff, b'Q', b'R'];
    let unusual_url = b"javascript:alert('qrforge-test')";

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

    let mut binary = canvas(1000, 800, 255);
    centered(&mut binary, &qr(binary_payload, 9));

    let mut unusual = canvas(1100, 800, 250);
    centered(&mut unusual, &qr(unusual_url, 8));

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
        Fixture {
            name: "normal-screen",
            category: "normal_screen",
            image: normal,
            expected: expected(&[normal_payload]),
        },
        Fixture {
            name: "multiple",
            category: "multiple",
            image: multiple,
            expected: expected(&multi_payloads),
        },
        Fixture {
            name: "rotated",
            category: "rotated",
            image: rotated,
            expected: expected(&[b"rotated-90-degrees"]),
        },
        Fixture {
            name: "inverted",
            category: "inverted",
            image: inverted,
            expected: expected(&[b"inverted-code"]),
        },
        Fixture {
            name: "low-contrast",
            category: "low_contrast",
            image: low_contrast,
            expected: expected(&[b"low-contrast-code"]),
        },
        Fixture {
            name: "blurred",
            category: "blurred",
            image: blurred,
            expected: expected(&[b"blurred-code"]),
        },
        Fixture {
            name: "damaged",
            category: "partially_damaged",
            image: damaged,
            expected: expected(&[b"partially-damaged-code"]),
        },
        Fixture {
            name: "perspective",
            category: "perspective_distorted",
            image: perspective,
            expected: expected(&[b"perspective-distorted"]),
        },
        Fixture {
            name: "small-high-res",
            category: "small_high_resolution",
            image: small_high_res,
            expected: expected(&[b"small-in-high-resolution"]),
        },
        Fixture {
            name: "unicode",
            category: "unicode",
            image: unicode,
            expected: expected(&[unicode_payload]),
        },
        Fixture {
            name: "binary",
            category: "binary",
            image: binary,
            expected: expected(&[binary_payload]),
        },
        Fixture {
            name: "unusual-url",
            category: "malicious_unusual_url",
            image: unusual,
            expected: expected(&[unusual_url]),
        },
        Fixture {
            name: "false-positive",
            category: "false_positive_background",
            image: false_positive,
            expected: BTreeSet::new(),
        },
    ]
}

fn write_corpus(fixtures: &[Fixture], output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(output)?;
    for fixture in fixtures {
        fixture
            .image
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
        let _ = decode(&fixture.image);
        let mut timings = Vec::with_capacity(ITERATIONS);
        let mut latest = BTreeSet::new();
        let mut correct = 0;
        for _ in 0..ITERATIONS {
            let started = Instant::now();
            latest = decode(&fixture.image);
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
        corpus_path: corpus_path.display().to_string(),
        fixtures: fixtures.len(),
        iterations_per_fixture: ITERATIONS,
        identical_inputs: true,
        qr_family_only: true,
        quircs,
        zxing_cpp,
        zxing_build: ZxingBuildCost {
            license: "Apache-2.0",
            linkage: "bundled static C++ core through Rust C-API wrapper",
            compiler_requirement: "C++20-capable compiler; MSVC Build Tools on Windows",
            ffi_boundary: "unsafe implementation is contained in the third-party zxing-cpp wrapper",
        },
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
