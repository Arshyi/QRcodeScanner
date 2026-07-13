# Phase 0.5 results

- Date: 2026-07-12
- Repository: `E:\QRcodeScanner\QRcodeScanner`
- Host: Windows 10 Education, x86-64, 12 logical processors
- Display: built-in `Color LCD`, 2880x1800 at 200% scaling
- Rust/Cargo: 1.97.0
- Native tools: Visual Studio Build Tools 2022, CMake 4.4.0
- Build profile: release

Phase 0.5 tested three remaining architecture gates without adding production application code.

## Gate decisions

| Gate | Decision | Evidence |
|---|---|---|
| Persistent Windows Graphics Capture | **Pass with split capture paths** | 100/100 frames in both modes; 6.14 ms full and 1.32 ms region median readback; 3.83 useful samples/s. Cold first-frame latency does not satisfy one-shot hotkey use. |
| Real-world decoder comparison | **Pass: select ZXing-C++** | ZXing passed 12/13 categories versus quircs 11/13 and was faster at aggregate median/p95. |
| Lazy Tauri webview lifecycle | **Pass** | 10/10 create/destroy cycles, tray and heartbeat survived, 0.60 MiB post-destroy resident-memory growth, clean programmatic exit. |

Phase 1 is approved with the narrow scope described at the end of this document.

## 1. Persistent Windows Graphics Capture

### Method

The spike uses `windows-capture` 2.0 over the Windows Graphics Capture API. Each mode creates one persistent WGC frame-pool/session and receives 100 change-driven frames. A local WinForms animation changes a visible screen region at approximately 60 Hz so reliability does not depend on unrelated desktop activity. Cursor and capture border are disabled. No frame is encoded or written.

The wrapper reuses WGC session/frame-pool GPU resources. Its `buffer` and `buffer_crop` implementation creates a CPU staging texture for every mapped readback; the spike reports `staging_texture_reused: false` rather than implying otherwise.

### Raw summary

| Metric | Full 2880x1800 | Center region 960x600 |
|---|---:|---:|
| Session startup | 143.86 ms | 109.75 ms |
| First frame from request | 347.70 ms | 437.06 ms |
| Frames / failures | 100 / 0 | 100 / 0 |
| Frame interval median | 264.41 ms | 263.94 ms |
| Frame interval p95 | 279.66 ms | 279.75 ms |
| GPU-to-CPU readback median | 6.14 ms | 1.32 ms |
| GPU-to-CPU readback p95 | 6.85 ms | 1.33 ms |
| Useful samples per second | 3.83 | 3.83 |
| Normalized process CPU | 0.025% | below measurement resolution |
| Maximum observed process physical memory | 46.57 MiB | 46.90 MiB at second-session start |

An explicit 16 ms minimum-update interval did not change the approximately 264 ms delivery interval on this host. WGC/compositor change coalescing therefore limits this environment to about 3.8 useful samples per second, which meets the 3 Hz minimum but not the 5 Hz target.

### Comparison with Phase 0 xcap

| Path | Median | p95 | Intended use |
|---|---:|---:|---|
| xcap one-shot full screen | 65.32 ms | 71.61 ms | Phase 1 hotkey scan fallback |
| Warm persistent WGC full readback | 6.14 ms | 6.85 ms | Smart Scroll active session |
| Warm persistent WGC region readback | 1.32 ms | 1.33 ms | Region/change preprocessing |
| Cold persistent WGC first frame | 347.70–437.06 ms | not sampled as distribution | Not suitable for one-shot hotkey capture |

Capture and QR decode were measured separately. Adding ZXing's aggregate p95 of 22.18 ms to xcap's p95 gives approximately 93.8 ms before policy/UI overhead, inside the revised 150 ms clear-code end-to-end target.

### Recommendation

- Use a persistent WGC session only while Smart Scroll is active.
- Prefer region/change preprocessing before QR decode.
- Use a separate one-shot capture adapter for Phase 1 hotkey scanning; xcap is acceptable behind the capture port with a revised 70 ms median / 80 ms p95 capture budget.
- Before Smart Scroll production work, replace per-frame CPU staging allocation with a reusable staging texture and remeasure.
- Do not keep WGC active while idle.

The original 35 ms one-shot target was unrealistic for cold capture on this host and is revised with explicit evidence rather than silently changed.

Raw summary: `spikes/windows-wgc/results/latest.json`.

## 2. Real-world decoder comparison

### Corpus

The benchmark deterministically generated 13 legally distributable PNG fixtures under `spikes/decoder-comparison/fixtures/generated`. Both engines received the exact same decoded grayscale image objects.

Categories:

- Normal screen QR
- Multiple QR codes
- Rotated
- Inverted
- Low contrast
- Blurred
- Partially damaged
- Perspective-distorted
- Small code in a 2880x1800 screenshot
- Unicode payload
- Binary payload
- Unusual/malicious URL text
- False-positive visual background

Each fixture ran 30 times per engine with QR-family formats restricted.

### Aggregate results

| Metric | quircs 0.10.3 | ZXing-C++ via zxing-cpp 0.5.2 |
|---|---:|---:|
| Categories passed | 11/13 | 12/13 |
| Aggregate median | 6.43 ms | 4.13 ms |
| Aggregate p95 | 24.71 ms | 22.18 ms |
| Observed physical-memory delta | +5.73 MiB | no positive delta after quircs run |
| Release comparison executable | n/a combined | 6,497,792 bytes combined |
| Pure quircs baseline executable | 314,880 bytes | n/a |

Memory deltas are process snapshots and order-sensitive because both engines execute in one process; they establish absence of ongoing growth, not isolated peak allocation.

### Correctness by category

| Category | quircs | ZXing-C++ |
|---|---|---|
| Normal screen | Pass | Pass |
| Multiple | Pass, 3/3 | Pass, 3/3 |
| Rotated | Pass | Pass |
| Inverted | **Fail, 0/1** | Pass |
| Low contrast | Pass | Pass |
| Blurred | Pass | Pass |
| Partially damaged | Pass | Pass |
| Perspective stress | **Fail, 0/1** | **Fail, 0/1** |
| Small/high-resolution | Pass | Pass |
| Unicode | Pass | Pass |
| Binary | Pass | Pass |
| Unusual URL | Pass | Pass |
| False-positive background | Pass, 0 detected | Pass, 0 detected |

### ZXing costs

- Apache-2.0 license and notice obligations.
- Bundled static C++ core through the Rust C-API wrapper.
- CMake and a C++20-capable compiler on each release platform.
- This host required `CMAKE_GENERATOR=Visual Studio 17 2022`; automatic selection incorrectly attempted an unavailable Visual Studio 2026 generator.
- The full cold release workspace build took approximately 6 minutes 35 seconds after dependencies were available.
- Unsafe FFI implementation lives in the third-party wrapper and must remain behind QRForge's safe decoder adapter.

### Recommendation

Select ZXing-C++ as the initial production decoder. It is both more correct on this corpus and faster. Do not ship a staged quircs-first strategy initially: it adds binary and maintenance cost but does not cover the perspective case that ZXing misses. Keep the adapter replaceable and preserve the shared corpus for regressions.

The perspective fixture remains failing and must not be weakened. Add real camera/screen photographs in later legally cleared corpus expansions.

Raw summary: `spikes/decoder-comparison/results/latest-summary.json`.

## 3. Lazy Tauri webview lifecycle

### Method

The Tauri host starts with a real tray icon and no webview. A heartbeat service runs every 100 ms. The benchmark then creates a visible settings webview, waits for process stabilization, destroys it, waits again, and repeats ten times. Process-tree resident memory, CPU, and process counts are sampled with `sysinfo`.

Destroying the first last-window attempt caused Tauri's default implicit exit. The correct tray-first policy is now explicit: prevent only `ExitRequested { code: None }`, while permitting programmatic `app.exit(0)`.

### Raw summary

| Metric | Result |
|---|---:|
| Cycles completed | 10/10 |
| Host-only resident memory | 17.85 MiB |
| Host-only normalized CPU | 0.0014% |
| Creation latency median / p95 | 484.55 / 535.11 ms |
| Visible resident memory median / p95 | 339.73 / 341.88 MiB |
| Destruction call | approximately 0.13 ms |
| Post-destroy resident memory median / p95 | 32.41 / 32.63 MiB |
| Post-destroy growth, cycle 1 to 10 | 0.60 MiB |
| Maximum process count | 7 |
| Post-destroy process count | 1 on every cycle |
| Heartbeat ticks | 552 |
| Tray present throughout | Yes |
| Programmatic shutdown exit code | 0 |

WebView2 consumes substantial memory while visible, but destruction consistently released child processes and returned the host to approximately 32 MiB. The roughly 14.6 MiB difference from the initial 17.85 MiB host sample is stable runtime initialization retained after first use, not cycle-over-cycle leakage.

### Recommendation

The lazy lifecycle is reliable enough for production:

- Start tray and native services without a webview.
- Create the settings/main window on demand.
- Destroy, rather than hide, the webview when closing to tray.
- Explicitly prevent implicit last-window exit.
- Provide a real Quit action that calls programmatic exit.
- Treat approximately 0.5 seconds as expected first/window recreation latency and show native/tray feedback immediately.

Raw result: `spikes/tauri-idle/results/lifecycle.json`.

## Validation and commands

Toolchain/setup commands used:

```powershell
cargo info windows-capture@2.0.0
cargo info zxing-cpp@0.5.1
winget install --id Kitware.CMake --exact --silent --accept-package-agreements --accept-source-agreements
$env:CMAKE_GENERATOR = 'Visual Studio 17 2022'
cargo clean --manifest-path spikes\Cargo.toml -p zxing-cpp
```

Benchmark commands:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\spikes\windows-wgc\run.ps1
.\spikes\target\release\qrforge-decoder-comparison.exe
.\spikes\target\release\qrforge-tauri-idle-spike.exe --lifecycle
```

Required validation commands are run at the end of Phase 0.5:

```powershell
cargo fmt --manifest-path spikes\Cargo.toml --all --check
cargo check --manifest-path spikes\Cargo.toml --workspace --all-targets
cargo test --manifest-path spikes\Cargo.toml --workspace
cargo clippy --manifest-path spikes\Cargo.toml --workspace --all-targets -- -D warnings
cargo build --release --manifest-path spikes\Cargo.toml --workspace
git status --short
```

## Remaining risks

1. WGC delivered only 3.8 useful changed frames/s despite a 16 ms requested minimum interval; 5 Hz is not proven on this host.
2. The WGC wrapper allocates CPU staging resources per readback; Smart Scroll needs a reusable staging implementation before production.
3. Cold WGC startup is too slow for the hotkey one-shot path.
4. Both decoders fail the current perspective-stress fixture.
5. ZXing introduces CMake, MSVC/C++20, FFI, licensing, and cross-platform build maintenance.
6. WebView2 visible memory is approximately 340 MiB on this host; the UI must never be retained invisibly.
7. Webview creation takes roughly half a second, so user feedback must come from the already-running native host/tray.
8. Resource figures are single-host measurements and require CI/reference-hardware regression tracking.

## Phase 1 decision

**Phase 1 is approved to begin**, narrowly scoped to:

- Native tray host
- Explicit Quit lifecycle
- Versioned settings foundation
- Typed IPC boundary
- Global hotkey manager
- One-shot visible-screen capture behind a replaceable port
- ZXing-C++ QR decoding
- Safe handling of one HTTP(S) URL

Do not start Smart Scroll, webcam UI, history UI, or broad visual polish in the first usable slice. Smart Scroll waits for reusable WGC staging buffers; webcam UI remains a later phase.

