# Tauri idle-resource benchmark

This minimal Tauri 2 host has no plugins, timers, commands, or production services. It creates no webview by default, modeling tray-idle lifecycle. `--with-window` creates one hidden webview for comparison.

```powershell
cargo build --release --manifest-path spikes/tauri-idle/src-tauri/Cargo.toml
powershell -ExecutionPolicy Bypass -File spikes/tauri-idle/measure.ps1
powershell -ExecutionPolicy Bypass -File spikes/tauri-idle/measure.ps1 -Arguments "--with-window"
spikes\target\release\qrforge-tauri-idle-spike.exe --lifecycle
```

The sampler includes the root process and descendants, normalizes CPU against logical processor count, and sums private working set.

`--lifecycle` creates a real tray icon, keeps a host heartbeat service alive, and performs ten visible settings-webview create/destroy cycles. It reports creation/destruction latency, process-tree memory/CPU, process counts, memory growth, tray continuity, and orderly shutdown intent.

The raw lifecycle report is written to `results/lifecycle.json` because release Tauri binaries use the Windows GUI subsystem.
