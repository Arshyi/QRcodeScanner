# Phase 0 technical spikes

These programs validate architecture choices and are not production modules.

- `windows-capture`: primary-monitor capture correctness and latency.
- `decoder`: deterministic in-memory multi-QR correctness and latency.
- `webcam`: Windows Media Foundation enumeration and physical frame delivery.
- `tauri-idle`: host-only versus hidden-webview process-tree resources.

All benchmarks print JSON. They never persist captured images or camera frames.

