# Real-world QR decoder comparison

Generates a deterministic, legally distributable PNG corpus and decodes each identical image with `quircs` and bundled ZXing-C++ through `zxing-cpp` 0.5.2.

```powershell
cargo run --release --manifest-path spikes/decoder-comparison/Cargo.toml
```

Categories include normal screen content, multiple, rotated, inverted, low-contrast, blurred, damaged, perspective-like distortion, small-in-high-resolution, Unicode, binary, unusual URL, and false-positive backgrounds. Generated corpus files are under `fixtures/generated/`.

ZXing-C++ is Apache-2.0 licensed. The Rust wrapper compiles and statically links the bundled C++ core, requiring a C++20-capable compiler and increasing build/packaging complexity relative to pure Rust.
