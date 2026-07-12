# Phase 0 validation plan

Phase 0 proves or rejects the highest-risk technical choices before production code begins.

## In scope

- Architectural decision records.
- Threat model and measurable performance budgets.
- Windows full-monitor capture latency and correctness spike.
- Offline multi-QR correctness and latency benchmark.
- Tauri host-only and hidden-webview idle-resource measurements.
- Windows webcam enumeration, open, and frame-delivery feasibility spike.
- CI skeleton for formatting, linting, tests, and release builds.

## Out of scope

- Production UI, hotkeys, tray, settings, history, updater, or persistence.
- Stable public APIs.
- Installers, signing, publishing, commits, or pushes.

## Measurement rules

1. Spikes use release builds.
2. Latency tests warm up before collecting at least 100 samples.
3. Idle tests stabilize for 10 seconds and sample for 30 seconds.
4. Reports include minimum, median, p95, maximum, and failures.
5. Captured pixels and camera frames remain memory-only.
6. Hardware/session details and untested cases are recorded with results.
7. A fast incorrect decoder is a failure.

