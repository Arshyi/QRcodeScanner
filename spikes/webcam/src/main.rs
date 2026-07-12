use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    query,
    utils::{ApiBackend, CameraIndex, RequestedFormat, RequestedFormatType},
};
use serde::Serialize;
use std::{process::ExitCode, time::Instant};

#[derive(Serialize)]
struct Stats {
    min: f64,
    median: f64,
    p95: f64,
    max: f64,
}

#[derive(Serialize)]
struct CameraInfo {
    index: String,
    name: String,
    description: String,
}

#[derive(Serialize)]
struct Report {
    spike: &'static str,
    backend: &'static str,
    enumeration_ms: f64,
    camera_count: usize,
    cameras: Vec<CameraInfo>,
    physical_stream_tested: bool,
    stream_open_ms: Option<f64>,
    first_frame_ms: Option<f64>,
    frame_count: usize,
    frame_width: Option<u32>,
    frame_height: Option<u32>,
    source_format: Option<String>,
    frame_interval_ms: Option<Stats>,
    stream_error: Option<String>,
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
    let started = Instant::now();
    let devices = query(ApiBackend::MediaFoundation)?;
    let enumeration_ms = started.elapsed().as_secs_f64() * 1_000.0;
    let cameras = devices
        .into_iter()
        .map(|device| CameraInfo {
            index: device.index().to_string(),
            name: device.human_name().clone(),
            description: device.description().to_owned(),
        })
        .collect::<Vec<_>>();

    let mut report = Report {
        spike: "webcam",
        backend: "Windows Media Foundation via nokhwa 0.10.11",
        enumeration_ms,
        camera_count: cameras.len(),
        cameras,
        physical_stream_tested: false,
        stream_open_ms: None,
        first_frame_ms: None,
        frame_count: 0,
        frame_width: None,
        frame_height: None,
        source_format: None,
        frame_interval_ms: None,
        stream_error: None,
    };

    if report.camera_count == 0 {
        return Ok(report);
    }

    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let mut camera = match Camera::new(CameraIndex::Index(0), requested) {
        Ok(camera) => camera,
        Err(error) => {
            report.stream_error = Some(error.to_string());
            return Ok(report);
        }
    };

    let open_started = Instant::now();
    if let Err(error) = camera.open_stream() {
        report.stream_error = Some(error.to_string());
        return Ok(report);
    }
    report.stream_open_ms = Some(open_started.elapsed().as_secs_f64() * 1_000.0);

    let mut frame_intervals = Vec::new();
    for index in 0..30 {
        let frame_started = Instant::now();
        match camera.frame() {
            Ok(frame) => {
                let elapsed = frame_started.elapsed().as_secs_f64() * 1_000.0;
                if index == 0 {
                    report.first_frame_ms = Some(elapsed);
                } else {
                    frame_intervals.push(elapsed);
                }
                report.frame_count += 1;
                report.frame_width = Some(frame.resolution().width());
                report.frame_height = Some(frame.resolution().height());
                report.source_format = Some(format!("{:?}", frame.source_frame_format()));
                std::hint::black_box(frame.buffer());
            }
            Err(error) => {
                report.stream_error = Some(error.to_string());
                break;
            }
        }
    }

    report.physical_stream_tested = report.frame_count > 0;
    if !frame_intervals.is_empty() {
        report.frame_interval_ms = Some(summarize(frame_intervals));
    }
    Ok(report)
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
            eprintln!("webcam spike failed: {error}");
            ExitCode::FAILURE
        }
    }
}
