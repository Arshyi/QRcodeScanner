# Development and release builds

## Supported host

Phase 1 targets 64-bit Windows 10 or newer. Development requires:

- current stable Rust installed through Rustup; the workspace minimum is Rust
  1.85
- Visual Studio 2022 Build Tools with the **Desktop development with C++**
  workload, including the x64 MSVC toolset, a Windows 10/11 SDK, and CMake
- Node.js and npm
- the Microsoft Edge WebView2 Runtime, normally present on supported Windows
  installations

VS Code is an editor and does not provide the MSVC linker or Windows SDK.

## Repository setup

```powershell
Set-Location QRcodeScanner

rustc --version
cargo --version
cmake --version

Set-Location apps\desktop
npm ci
Set-Location ..\..
```

The explicit optional dependency on
`@tauri-apps/cli-win32-x64-msvc` is intentional. It pins the native Windows
CLI binary that a clean npm install needs for `tauri build`; do not remove it
merely because `@tauri-apps/cli` also declares platform packages as optional.

The application has no runtime network client or telemetry. Cargo and npm need
network access only to obtain locked build dependencies.

## ZXing-C++ and the MSVC runtime

`qrforge-decoder` pins `zxing-cpp` 0.5.2 and enables its `bundled` feature, so
Cargo builds ZXing-C++ locally with CMake. Rust's MSVC targets link the dynamic
release CRT in both Debug and Release Cargo profiles. A default CMake Debug
build instead selects the debug CRT, which causes incompatible-library or
Debug-CRT linkage failures.

The checked-in `.cargo/config.toml` sets `CMAKE_TOOLCHAIN_FILE` to
`cmake/msvc-runtime.cmake`. The Rust `cmake` crate passes that supported
setting to the bundled dependency before CMake enables the MSVC languages.
The toolchain file selects `MultiThreadedDLL` (`/MD`) in every profile. This
is build configuration, not application runtime behavior, and should not be
replaced with a warning suppression.

A clean-enough verification can use a separate target directory:

```powershell
$env:CARGO_TARGET_DIR = 'target\phase1-validation'
cargo test --workspace
cargo build --release --workspace
```

Inspect the resulting ZXing CMake cache and confirm:

```text
CMAKE_MSVC_RUNTIME_LIBRARY:STRING=MultiThreadedDLL
```

Remove `CARGO_TARGET_DIR` from the current shell after the isolated run if the
default `target` directory should be used again.

## Required validation

Run these commands from the repository root:

```powershell
cargo fmt --all --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --workspace
```

Run the frontend gates from `apps\desktop`:

```powershell
npm ci
npm run format:check
npm run lint
npm run typecheck
npm run test
npm run build
```

## Development and production launch

From `apps\desktop`:

```powershell
npm run tauri -- dev
npm run tauri -- build --bundles nsis
```

The NSIS installer is emitted under:

```text
target\release\bundle\nsis\QRForge_<version>_x64-setup.exe
```

Release signing is not configured in Phase 1, so a successful local bundle is
an unsigned test installer.

Current measured Release timings, process counts, memory, artifact hashes, and
installer validation are recorded in `docs/phase-1-results.md`.

## Manual Windows release checklist

Use a Release build and record actual evidence in `docs/phase-1-results.md`.

1. Launch QRForge and confirm no settings webview exists at idle.
2. Launch a second copy and confirm it exits while the first copy opens or
   focuses Settings.
3. Exercise tray **Scan Now**, **Open Settings**, and **Quit**.
4. Close Settings and confirm its WebView2 children exit while the tray host
   and registered hotkey remain.
5. Reopen Settings and confirm the saved hotkey and options are correct.
6. Display the checked-in fixtures and exercise normal, inverted, Unicode,
   multiple-code, malformed URL-like, no-code, and dangerous-scheme cases.
7. Trigger the hotkey repeatedly during a scan and confirm only one capture
   worker runs and the duplicate request receives feedback.
8. Repeat at least ten settings create/destroy cycles and ten scans; compare
   process counts and private working set before and after.
9. Search the application data directory and repository for newly created
   image/frame files. Only settings and explicitly enabled diagnostics should
   be written.

## Local application data

Settings live in Tauri's per-user application-data directory as
`settings.json`. Writes use a same-directory temporary file and atomic
replacement. Malformed JSON falls back to validated defaults without deleting
the corrupt source file; the next successful settings save replaces it.

When `QRFORGE_DIAGNOSTICS=1` is present at launch, QRForge writes local JSONL
timings and lifecycle events. Diagnostics contain result categories, timing,
frame dimensions, and monitor/scaling metadata, but never pixels or decoded
payload bytes. Diagnostics are disabled by default.
