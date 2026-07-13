use serde::Serialize;
use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
use sysinfo::{Pid, ProcessesToUpdate, System};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

const CYCLES: usize = 10;

#[derive(Serialize)]
struct ResourceSample {
    resident_memory_mib: f64,
    normalized_cpu_percent: f32,
    process_count: usize,
}

#[derive(Serialize)]
struct CycleReport {
    cycle: usize,
    create_ms: f64,
    visible: ResourceSample,
    destroy_ms: f64,
    post_destroy: ResourceSample,
    tray_present: bool,
}

#[derive(Serialize)]
struct Stats {
    min: f64,
    median: f64,
    p95: f64,
    max: f64,
}

#[derive(Serialize)]
struct LifecycleReport {
    spike: &'static str,
    cycles: usize,
    host_only: ResourceSample,
    create_latency_ms: Stats,
    visible_memory_mib: Stats,
    post_destroy_memory_mib: Stats,
    post_destroy_memory_growth_mib: f64,
    maximum_process_count: usize,
    heartbeat_ticks: u64,
    tray_present_all_cycles: bool,
    host_service_functional: bool,
    shutdown_requested: bool,
    cycle_results: Vec<CycleReport>,
}

pub fn start_heartbeat(counter: Arc<AtomicU64>) {
    thread::spawn(move || {
        loop {
            counter.fetch_add(1, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(100));
        }
    });
}

pub fn start(app: AppHandle, heartbeat: Arc<AtomicU64>) {
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        let output = match run(&app, &heartbeat) {
            Ok(report) => {
                serde_json::to_string_pretty(&report).expect("lifecycle report must serialize")
            }
            Err(error) => serde_json::to_string_pretty(&serde_json::json!({
                "spike": "lazy-tauri-webview-lifecycle",
                "error": error.to_string()
            }))
            .expect("lifecycle error must serialize"),
        };
        let results = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../results");
        if fs::create_dir_all(&results).is_ok() {
            let _ = fs::write(results.join("lifecycle.json"), output);
        }
        app.exit(0);
    });
}

fn run(
    app: &AppHandle,
    heartbeat: &AtomicU64,
) -> Result<LifecycleReport, Box<dyn std::error::Error + Send + Sync>> {
    let mut system = System::new_all();
    let host_only = sample_process_tree(&mut system);
    let heartbeat_start = heartbeat.load(Ordering::Relaxed);
    let mut cycle_results = Vec::with_capacity(CYCLES);

    for cycle in 1..=CYCLES {
        let create_started = Instant::now();
        run_on_main(app, {
            let app = app.clone();
            move || {
                WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("index.html".into()))
                    .title("QRForge Settings Lifecycle Spike")
                    .visible(true)
                    .build()
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            }
        })?;
        let create_ms = create_started.elapsed().as_secs_f64() * 1_000.0;
        thread::sleep(Duration::from_secs(2));
        let visible = sample_process_tree(&mut system);

        let destroy_started = Instant::now();
        run_on_main(app, {
            let app = app.clone();
            move || {
                let window = app
                    .get_webview_window("settings")
                    .ok_or_else(|| "settings window not found".to_owned())?;
                window.destroy().map_err(|error| error.to_string())
            }
        })?;
        let destroy_ms = destroy_started.elapsed().as_secs_f64() * 1_000.0;
        thread::sleep(Duration::from_secs(2));
        let post_destroy = sample_process_tree(&mut system);
        cycle_results.push(CycleReport {
            cycle,
            create_ms,
            visible,
            destroy_ms,
            post_destroy,
            tray_present: app.tray_by_id("main-tray").is_some(),
        });
    }

    let create_latencies = cycle_results.iter().map(|cycle| cycle.create_ms).collect();
    let visible_memory = cycle_results
        .iter()
        .map(|cycle| cycle.visible.resident_memory_mib)
        .collect();
    let post_memory = cycle_results
        .iter()
        .map(|cycle| cycle.post_destroy.resident_memory_mib)
        .collect::<Vec<_>>();
    let post_destroy_memory_growth_mib = post_memory.last().copied().unwrap_or_default()
        - post_memory.first().copied().unwrap_or_default();
    let maximum_process_count = cycle_results
        .iter()
        .flat_map(|cycle| {
            [
                cycle.visible.process_count,
                cycle.post_destroy.process_count,
            ]
        })
        .max()
        .unwrap_or(host_only.process_count);
    let heartbeat_ticks = heartbeat.load(Ordering::Relaxed) - heartbeat_start;
    let tray_present_all_cycles = cycle_results.iter().all(|cycle| cycle.tray_present);

    Ok(LifecycleReport {
        spike: "lazy-tauri-webview-lifecycle",
        cycles: CYCLES,
        host_only,
        create_latency_ms: summarize(create_latencies),
        visible_memory_mib: summarize(visible_memory),
        post_destroy_memory_mib: summarize(post_memory),
        post_destroy_memory_growth_mib,
        maximum_process_count,
        heartbeat_ticks,
        tray_present_all_cycles,
        host_service_functional: heartbeat_ticks >= 300,
        shutdown_requested: true,
        cycle_results,
    })
}

fn run_on_main<F>(
    app: &AppHandle,
    operation: F,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: FnOnce() -> Result<(), String> + Send + 'static,
{
    let (sender, receiver) = mpsc::sync_channel(1);
    app.run_on_main_thread(move || {
        let _ = sender.send(operation());
    })?;
    receiver
        .recv_timeout(Duration::from_secs(15))
        .map_err(|error| error.to_string())?
        .map_err(Into::into)
}

fn sample_process_tree(system: &mut System) -> ResourceSample {
    system.refresh_processes(ProcessesToUpdate::All, true);
    thread::sleep(Duration::from_millis(500));
    system.refresh_processes(ProcessesToUpdate::All, true);

    let root = Pid::from_u32(std::process::id());
    let mut members = HashSet::from([root]);
    loop {
        let before = members.len();
        for (pid, process) in system.processes() {
            if process
                .parent()
                .is_some_and(|parent| members.contains(&parent))
            {
                members.insert(*pid);
            }
        }
        if members.len() == before {
            break;
        }
    }

    let processors = u16::try_from(System::physical_core_count().unwrap_or(1)).unwrap_or(u16::MAX);
    let mut memory = 0_u64;
    let mut cpu = 0_f32;
    for pid in &members {
        if let Some(process) = system.process(*pid) {
            memory = memory.saturating_add(process.memory());
            cpu += process.cpu_usage();
        }
    }
    ResourceSample {
        resident_memory_mib: f64::from(u32::try_from(memory / 1_024).unwrap_or(u32::MAX)) / 1_024.0,
        normalized_cpu_percent: cpu / f32::from(processors),
        process_count: members.len(),
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
