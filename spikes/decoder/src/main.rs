use image::{ImageBuffer, Luma};
use qrcode::QrCode;
use serde::Serialize;
use std::{collections::BTreeSet, env, process::ExitCode, time::Instant};

const EXPECTED: [&str; 5] = [
    "https://example.com/qrforge/a",
    "QRForge offline text payload",
    "https://example.org/docs?source=screen",
    "WIFI:T:WPA;S:LocalOnly;P:not-a-real-password;;",
    "mailto:security@example.net",
];

#[derive(Serialize)]
struct Stats {
    min: f64,
    median: f64,
    p95: f64,
    max: f64,
}

#[derive(Serialize)]
struct Report {
    spike: &'static str,
    engine: &'static str,
    frame: &'static str,
    iterations: usize,
    expected_codes: usize,
    decoded_codes: Vec<String>,
    correct_iterations: usize,
    latency_ms: Stats,
}

fn argument(name: &str, default: usize) -> usize {
    env::args()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|pair| pair[0] == name)
        .and_then(|pair| pair[1].parse().ok())
        .unwrap_or(default)
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

fn fixture() -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let mut canvas = ImageBuffer::from_pixel(1920, 1080, Luma([255]));
    let placements = [
        (70, 60, 7),
        (680, 80, 6),
        (1260, 90, 5),
        (250, 570, 6),
        (1100, 580, 7),
    ];

    for (payload, (x, y, scale)) in EXPECTED.iter().zip(placements) {
        let code = QrCode::new(payload.as_bytes()).expect("fixture payload must encode");
        let symbol = code
            .render::<Luma<u8>>()
            .quiet_zone(true)
            .module_dimensions(scale, scale)
            .build();
        image::imageops::overlay(&mut canvas, &symbol, x, y);
    }
    canvas
}

fn decode(frame: &ImageBuffer<Luma<u8>, Vec<u8>>) -> BTreeSet<String> {
    let mut decoder = quircs::Quirc::default();
    decoder.resize(frame.width() as usize, frame.height() as usize);
    decoder
        .identify(
            frame.width() as usize,
            frame.height() as usize,
            frame.as_raw(),
        )
        .filter_map(Result::ok)
        .filter_map(|code| code.decode().ok())
        .filter_map(|decoded| String::from_utf8(decoded.payload).ok())
        .collect()
}

fn run() -> Result<Report, Box<dyn std::error::Error>> {
    let iterations = argument("--iterations", 100);
    let warmup = argument("--warmup", 10);
    if iterations == 0 {
        return Err("iterations must be positive".into());
    }

    let frame = fixture();
    for _ in 0..warmup {
        let _ = decode(&frame);
    }

    let expected = EXPECTED
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    let mut samples = Vec::with_capacity(iterations);
    let mut latest = BTreeSet::new();
    let mut correct_iterations = 0;

    for _ in 0..iterations {
        let started = Instant::now();
        latest = decode(&frame);
        samples.push(started.elapsed().as_secs_f64() * 1_000.0);
        if latest == expected {
            correct_iterations += 1;
        }
    }

    Ok(Report {
        spike: "decoder",
        engine: "quircs 0.10.3",
        frame: "1920x1080 L8",
        iterations,
        expected_codes: expected.len(),
        decoded_codes: latest.into_iter().collect(),
        correct_iterations,
        latency_ms: summarize(samples),
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(report) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("report must serialize")
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("decoder spike failed: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn generated_fixture_decodes_every_payload() {
        let frame = super::fixture();
        assert_eq!(super::decode(&frame).len(), super::EXPECTED.len());
    }
}
