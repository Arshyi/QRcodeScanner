# Windows capture spike

Captures the primary monitor through `xcap`, validates the returned dimensions/buffer, and reports latency. Frames are neither encoded nor written.

```powershell
cargo run --release --manifest-path spikes/windows-capture/Cargo.toml -- --iterations 100 --warmup 10
```

This validates a one-shot cross-platform adapter candidate. A persistent Windows Graphics Capture session remains a separate production-path comparison if latency or allocation misses budget.

