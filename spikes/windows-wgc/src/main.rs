use cpu_time::ProcessTime;
use memory_stats::memory_stats;
use serde::Serialize;
use std::{hint::black_box, thread, time::Duration, time::Instant};
use windows_capture::{
    capture::{Context, GraphicsCaptureApiHandler},
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    },
};

const FRAME_COUNT: usize = 100;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum Mode {
    Full,
    CenterRegion,
}

#[derive(Clone, Copy)]
struct Flags {
    mode: Mode,
    requested_at: Instant,
    monitor_width: u32,
    monitor_height: u32,
}

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
    backend: &'static str,
    mode: Mode,
    monitor_width: u32,
    monitor_height: u32,
    sampled_width: u32,
    sampled_height: u32,
    startup_ms: f64,
    first_frame_ms: f64,
    frames: usize,
    failures: usize,
    frame_interval_ms: Stats,
    gpu_to_cpu_readback_ms: Stats,
    useful_samples_per_second: f64,
    normalized_cpu_percent: f64,
    physical_memory_start_mib: Option<f64>,
    physical_memory_end_mib: Option<f64>,
    checksum: u64,
    staging_texture_reused: bool,
}

struct Capture {
    flags: Flags,
    session_started: Instant,
    process_cpu_started: ProcessTime,
    memory_start: Option<usize>,
    first_frame_ms: Option<f64>,
    previous_frame: Option<Instant>,
    intervals: Vec<f64>,
    readbacks: Vec<f64>,
    failures: usize,
    sampled_size: (u32, u32),
    checksum: u64,
}

impl GraphicsCaptureApiHandler for Capture {
    type Flags = Flags;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self {
            flags: ctx.flags,
            session_started: Instant::now(),
            process_cpu_started: ProcessTime::now(),
            memory_start: memory_stats().map(|stats| stats.physical_mem),
            first_frame_ms: None,
            previous_frame: None,
            intervals: Vec::with_capacity(FRAME_COUNT - 1),
            readbacks: Vec::with_capacity(FRAME_COUNT),
            failures: 0,
            sampled_size: (0, 0),
            checksum: 0,
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        let arrived = Instant::now();
        self.first_frame_ms
            .get_or_insert_with(|| self.flags.requested_at.elapsed().as_secs_f64() * 1_000.0);
        if let Some(previous) = self.previous_frame.replace(arrived) {
            self.intervals
                .push(arrived.duration_since(previous).as_secs_f64() * 1_000.0);
        }

        let readback_started = Instant::now();
        let read_result = match self.flags.mode {
            Mode::Full => frame.buffer(),
            Mode::CenterRegion => {
                let width = frame.width();
                let height = frame.height();
                let region_width = width / 3;
                let region_height = height / 3;
                let left = (width - region_width) / 2;
                let top = (height - region_height) / 2;
                frame.buffer_crop(left, top, left + region_width, top + region_height)
            }
        };

        match read_result {
            Ok(mut buffer) => {
                self.readbacks
                    .push(readback_started.elapsed().as_secs_f64() * 1_000.0);
                self.sampled_size = (buffer.width(), buffer.height());
                let stride = usize::try_from(buffer.row_pitch()).unwrap_or(1).max(1);
                self.checksum = self.checksum.wrapping_add(
                    buffer
                        .as_raw_buffer()
                        .iter()
                        .step_by(stride)
                        .map(|byte| u64::from(*byte))
                        .sum::<u64>(),
                );
                black_box(self.checksum);
            }
            Err(_) => self.failures += 1,
        }

        if self.readbacks.len() + self.failures >= FRAME_COUNT {
            let wall = self.session_started.elapsed();
            let cpu = self.process_cpu_started.elapsed();
            let processors = thread::available_parallelism()
                .map_or(1, usize::from)
                .try_into()
                .unwrap_or(u32::MAX);
            let sample_count = u32::try_from(self.readbacks.len()).unwrap_or(u32::MAX);
            let normalized_cpu_percent =
                cpu.as_secs_f64() / wall.as_secs_f64() / f64::from(processors) * 100.0;
            let report = Report {
                spike: "persistent-windows-graphics-capture",
                backend: "windows-capture 2.0 / Windows Graphics Capture",
                mode: self.flags.mode,
                monitor_width: self.flags.monitor_width,
                monitor_height: self.flags.monitor_height,
                sampled_width: self.sampled_size.0,
                sampled_height: self.sampled_size.1,
                startup_ms: self
                    .session_started
                    .duration_since(self.flags.requested_at)
                    .as_secs_f64()
                    * 1_000.0,
                first_frame_ms: self.first_frame_ms.unwrap_or_default(),
                frames: self.readbacks.len(),
                failures: self.failures,
                frame_interval_ms: summarize(self.intervals.clone()),
                gpu_to_cpu_readback_ms: summarize(self.readbacks.clone()),
                useful_samples_per_second: f64::from(sample_count) / wall.as_secs_f64(),
                normalized_cpu_percent,
                physical_memory_start_mib: self.memory_start.map(to_mib),
                physical_memory_end_mib: memory_stats().map(|stats| to_mib(stats.physical_mem)),
                checksum: self.checksum,
                staging_texture_reused: false,
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
            control.stop();
        }
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        Ok(())
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

fn run(mode: Mode) -> Result<(), Box<dyn std::error::Error>> {
    let monitor = Monitor::primary()?;
    let width = monitor.width()?;
    let height = monitor.height()?;
    let flags = Flags {
        mode,
        requested_at: Instant::now(),
        monitor_width: width,
        monitor_height: height,
    };
    let settings = Settings::new(
        monitor,
        CursorCaptureSettings::WithoutCursor,
        DrawBorderSettings::WithoutBorder,
        SecondaryWindowSettings::Exclude,
        MinimumUpdateIntervalSettings::Custom(Duration::from_millis(16)),
        DirtyRegionSettings::Default,
        ColorFormat::Bgra8,
        flags,
    );
    Capture::start(settings)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(Mode::Full)?;
    thread::sleep(Duration::from_millis(500));
    run(Mode::CenterRegion)?;
    Ok(())
}
