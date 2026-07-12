# ADR 0003: Offline QR decoder behind a replaceable adapter

- Status: Proposed pending comparative corpus benchmark
- Date: 2026-07-12

## Context

The decoder must find multiple QR codes offline, preserve payload bytes, and remain fast on full-screen frames. Native FFI increases build, packaging, and memory-safety complexity.

## Decision

Establish a pure-Rust `quircs` QR-only performance baseline in Phase 0. Keep the production decoder behind a borrowed-luminance adapter. Compare the baseline with ZXing-C++ using the same real-world corpus before choosing a production engine.

ZXing-C++ is accepted only if its resilience to rotation, inversion, damage, perspective, and difficult screen content materially justifies the C++ toolchain and FFI boundary.

## Acceptance criteria

- Exact expected result set on every generated multi-code iteration.
- Median below 40 ms and p95 below 100 ms for a 1920x1080 luminance fixture.
- No network access or image persistence.
- Real-world follow-up corpus before final acceptance.

