# ADR 0003: Offline QR decoder behind a replaceable adapter

- Status: Accepted: ZXing-C++ primary decoder
- Date: 2026-07-12

## Context

The decoder must find multiple QR codes offline, preserve payload bytes, and remain fast on full-screen frames. Native FFI increases build, packaging, and memory-safety complexity.

## Decision

Keep the production decoder behind a borrowed-luminance adapter and use ZXing-C++ as the initial primary engine. On the identical 13-category corpus, ZXing-C++ passed 12 categories versus 11 for `quircs`, recovered inverted QR codes, and had lower aggregate median and p95 latency. Both engines failed the current perspective-stress fixture, so that fixture remains a regression target rather than being weakened.

Use the bundled static `zxing-cpp` Rust wrapper. Pin its version, restrict formats to the QR family, and isolate the FFI dependency in the decoder adapter. Do not ship both decoders initially: a staged strategy adds binary and maintenance cost without covering the shared perspective failure.

## Acceptance criteria

- Exact expected result set on every generated multi-code iteration.
- Median below 40 ms and p95 below 100 ms for a 1920x1080 luminance fixture.
- No network access or image persistence.
- Real-world follow-up corpus before final acceptance.

## Costs accepted

- Apache-2.0 dependency and license notice.
- CMake plus a C++20-capable compiler on every release platform.
- Approximately 6.5 MB comparison executable versus 0.3 MB for the pure-Rust baseline; production size must be measured after adapter integration and release optimization.
- Unsafe code exists inside the third-party wrapper, not in QRForge domain/application crates.
