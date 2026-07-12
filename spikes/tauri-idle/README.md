# Tauri idle-resource benchmark

This minimal Tauri 2 host has no plugins, timers, commands, or production services. It creates no webview by default, modeling tray-idle lifecycle. `--with-window` creates one hidden webview for comparison.

```powershell
cargo build --release --manifest-path spikes/tauri-idle/src-tauri/Cargo.toml
powershell -ExecutionPolicy Bypass -File spikes/tauri-idle/measure.ps1
powershell -ExecutionPolicy Bypass -File spikes/tauri-idle/measure.ps1 -Arguments "--with-window"
```

The sampler includes the root process and descendants, normalizes CPU against logical processor count, and sums private working set.

