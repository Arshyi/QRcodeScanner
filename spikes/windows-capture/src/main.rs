use serde::Serialize;
use std::{env, process::ExitCode, time::Instant};
use xcap::Monitor;

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
    monitor_name: String,
    width: u32,
    height: u32,
    scale_factor: f32,
    iterations: usize,
    failures: usize,
    latency_ms: Stats,
    checksum: u64,
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

fn run() -> Result<Report, Box<dyn std::error::Error>> {
    let iterations = argument("--iterations", 100);
    let warmup = argument("--warmup", 10);
    if iterations == 0 {
        return Err("iterations must be positive".into());
    }

    let monitors = Monitor::all()?;
    let monitor = monitors
        .iter()
        .find(|candidate| candidate.is_primary().unwrap_or(false))
        .or_else(|| monitors.first())
        .ok_or("no monitor was enumerated")?;

    for _ in 0..warmup {
        drop(monitor.capture_image()?);
    }

    let mut samples = Vec::with_capacity(iterations);
    let mut failures = 0;
    let mut checksum = 0_u64;
    let mut dimensions = (0, 0);

    for _ in 0..iterations {
        let started = Instant::now();
        match monitor.capture_image() {
            Ok(frame) => {
                samples.push(started.elapsed().as_secs_f64() * 1_000.0);
                dimensions = (frame.width(), frame.height());
                let stride = (frame.width() as usize).saturating_mul(4).max(1);
                checksum = checksum.wrapping_add(
                    frame
                        .as_raw()
                        .iter()
                        .step_by(stride)
                        .map(|byte| u64::from(*byte))
                        .sum::<u64>(),
                );
            }
            Err(_) => failures += 1,
        }
    }

    if samples.is_empty() {
        return Err("all captures failed".into());
    }

    Ok(Report {
        spike: "windows-capture",
        engine: "xcap 0.7.1",
        monitor_name: monitor.name().unwrap_or_else(|_| "unknown".into()),
        width: dimensions.0,
        height: dimensions.1,
        scale_factor: monitor.scale_factor().unwrap_or(1.0),
        iterations,
        failures,
        latency_ms: summarize(samples),
        checksum,
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
            eprintln!("capture spike failed: {error}");
            ExitCode::FAILURE
        }
    }
}
