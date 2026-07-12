# Webcam feasibility spike

Enumerates cameras through Windows Media Foundation. If a camera is present, it opens the first device and retrieves 30 raw frames without persisting them.

```powershell
cargo run --release --manifest-path spikes/webcam/Cargo.toml
```

The happy path does not replace permission-denial, contention, unplug, suspend/resume, and reconnection testing.

