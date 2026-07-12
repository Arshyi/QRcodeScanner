# Multi-QR decoder benchmark

Generates five deterministic QR symbols in memory, composites them into a 1920x1080 luminance frame, and repeatedly decodes the frame using `quircs`.

```powershell
cargo run --release --manifest-path spikes/decoder/Cargo.toml -- --iterations 100 --warmup 10
```

This is a pure-Rust performance baseline. It does not finalize the production decoder; the same interface and corpus must be used for a ZXing-C++ resilience comparison.

