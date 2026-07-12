# Phase 0 results

- Date: 2026-07-12
- Repository: `E:\QRcodeScanner\QRcodeScanner`
- Host: Windows 10 Education, x86-64, 12 logical processors
- Display: `Color LCD`, 2880x1800, scale factor 2.0
- Toolchain: Rust/Cargo 1.97.0, release profile

These are measurements from one interactive Windows host, not universal product claims.

## Summary

| Spike | Measured result | Budget |
|---|---|---|
| Tauri host-only idle | 0% mean/p95 CPU; 3.586 MiB mean, 3.629 MiB p95 private memory | Pass |
| Tauri hidden-webview idle | 0.0112% mean, 0% p95 CPU; 162.19 MiB mean, 162.85 MiB p95 private memory | CPU pass; memory recorded separately |
| Primary-monitor capture | 0/100 failures; 65.324 ms median, 71.615 ms p95 | Fail |
| Multi-QR decode | 5/5 codes in 100/100 iterations; 13.699 ms median, 16.472 ms p95 | Pass |
| Webcam | One device; 30/30 frames; 825.407 ms first frame; 33.162 ms median interval | Feasible |

## Tauri idle resources

Tauri 2.11.5 was built without plugins, timers, IPC commands, or production services. The same binary supports two lifecycles.

### Host-only tray-idle model

```json
{
  "warmup_seconds": 10,
  "sample_seconds": 30,
  "logical_processors": 12,
  "cpu_percent": {
    "min": 0.0,
    "median": 0.0,
    "p95": 0.0,
    "max": 0.0,
    "mean": 0.0
  },
  "private_mib": {
    "min": 3.515625,
    "median": 3.62890625,
    "p95": 3.62890625,
    "max": 3.62890625,
    "mean": 3.5859375
  },
  "process_count_max": 2
}
```

This comfortably passes the tray-idle resource budget. The production lifecycle should create the webview lazily and destroy it when the user closes the UI to tray.

### Hidden webview

```json
{
  "warmup_seconds": 10,
  "sample_seconds": 30,
  "logical_processors": 12,
  "cpu_percent": {
    "min": 0.0,
    "median": 0.0,
    "p95": 0.0,
    "max": 0.3256131,
    "mean": 0.0112280
  },
  "private_mib": {
    "min": 161.7578125,
    "median": 161.80078125,
    "p95": 162.8515625,
    "max": 162.90234375,
    "mean": 162.19140625
  },
  "process_count_max": 8
}
```

CPU remains excellent, but retaining a hidden WebView2 process tree costs approximately 158.6 MiB more private memory than host-only idle. A permanently hidden window is rejected.

## Windows screen capture

The `xcap` 0.7.1 one-shot API captured the full 2880x1800 primary monitor. Each row was touched in memory and no frame was encoded or persisted.

```json
{
  "iterations": 100,
  "failures": 0,
  "latency_ms": {
    "min": 47.6856,
    "median": 65.3235,
    "p95": 71.6148,
    "max": 75.0418
  }
}
```

Correctness passed, but the 35 ms median and 60 ms p95 latency budgets were missed. The owned-image API likely includes allocation/copy/readback costs unsuitable for continuous Smart Scroll. Production should use a persistent Windows Graphics Capture session with reusable textures/buffers and an early crop/downscale path. `xcap` may remain a one-shot fallback only after profiling its allocation behavior.

## Multi-QR decoding

The pure-Rust `quircs` 0.10.3 baseline decoded a generated 1920x1080 luminance image containing five QR codes.

```json
{
  "iterations": 100,
  "expected_codes": 5,
  "correct_iterations": 100,
  "latency_ms": {
    "min": 12.3401,
    "median": 13.6992,
    "p95": 16.4719,
    "max": 17.3701
  }
}
```

Every expected payload was returned on every iteration. This proves clear generated multi-code performance only. Rotation, inversion, damage, low contrast, perspective, binary/ECI payloads, and false positives still require a real-world corpus. ZXing-C++ should be compared against the same corpus; FFI complexity is not justified by current evidence alone.

## Webcam feasibility

Windows Media Foundation was exercised through `nokhwa` 0.10.11 using the built-in FaceTime HD Camera.

```json
{
  "enumeration_ms": 4.4288,
  "camera_count": 1,
  "physical_stream_tested": true,
  "stream_open_ms": 0.0073,
  "first_frame_ms": 825.4065,
  "frame_count": 30,
  "frame": "1280x720 YUYV",
  "frame_interval_ms": {
    "min": 26.2782,
    "median": 33.1616,
    "p95": 34.0180,
    "max": 34.1851
  },
  "stream_error": null
}
```

The happy path is feasible and stabilized near 30 FPS. `open_stream` returns before frame readiness, so application state must distinguish `Starting` from `Ready` and enforce a first-frame timeout. Permission denial, contention, unplug/replug, sleep/resume, and device switching remain physical-test requirements.

## Validation commands

The following repository-local validations passed:

- `cargo fmt --manifest-path spikes\Cargo.toml --all --check`
- `cargo check --manifest-path spikes\Cargo.toml --workspace --all-targets`
- `cargo test --manifest-path spikes\Cargo.toml --workspace`
- `cargo clippy --manifest-path spikes\Cargo.toml --workspace --all-targets -- -D warnings`
- `cargo build --release --manifest-path spikes\Cargo.toml --workspace`

The decoder fixture unit test passed. The first test invocation timed out during initial native compilation; the immediate incremental rerun completed successfully.

## Failed assumptions and architectural risks

1. A hidden Tauri webview is not a low-memory tray-idle state; it must be destroyed when not visible.
2. One-shot full-resolution `xcap` capture is slower than the capture budget and not appropriate as the primary Smart Scroll engine.
3. ZXing-C++ cannot be selected solely on reputation; the pure-Rust baseline is already fast and avoids FFI/build complexity.
4. Generated QR fixtures do not represent hostile or degraded screen content.
5. Webcam stream-open completion is not camera readiness; first-frame latency was approximately 825 ms.
6. Windows build/test cold starts are expensive because Tauri/WebView2 has a large native dependency graph.
7. Process-tree memory depends on the installed WebView2 runtime and must be tracked on reference hardware.

## Production-stack recommendation

- **Core:** Rust with clean domain/application boundaries and replaceable platform ports.
- **Desktop shell:** Tauri 2, accepted with a strict lazy-webview lifecycle. Host-only tray idle is the default state.
- **UI:** Svelte 5 and TypeScript, loaded only while the window exists.
- **Screen capture:** a direct persistent Windows Graphics Capture adapter with reusable GPU/CPU buffers. Do not use one-shot `xcap` as the continuous engine.
- **Decoder:** retain `quircs` as the current performance baseline. Final selection requires a real-world `quircs` versus ZXing-C++ correctness comparison.
- **Webcam:** Windows Media Foundation behind a replaceable port; `nokhwa` is feasible for an initial adapter subject to lifecycle/permission stress tests.
- **History:** SQLite behind a repository adapter, with no captured imagery and configurable retention.

Phase 0 validates Rust, the Tauri host-only lifecycle, pure-Rust multi-code performance, and Windows webcam feasibility. It rejects a permanently hidden webview and one-shot full-resolution capture as primary production designs.

