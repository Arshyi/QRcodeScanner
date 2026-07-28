# QRForge

QRForge is a local-first Windows tray utility that scans QR codes from the
primary screen. Capture and decoding stay in native memory; the application
does not upload or persist screenshots.

## Phase 1 status

The narrow Windows MVP is complete. The current source passed clean frontend
and native gates, a full Release and NSIS build, installer install/launch/
uninstall checks, and current Windows lifecycle, hotkey, fixture, privacy, and
performance validation. See the
[Phase 1 completion report](docs/phase-1-results.md) for exact evidence,
artifact hashes, and remaining limitations.

The current scope is intentionally narrow:

- tray-first startup with explicit Quit
- one running host enforced by Tauri's platform single-instance plugin
- configurable global scan hotkey
- one-shot primary-monitor capture through xcap
- local QR-family decoding through bundled ZXing-C++
- Rust-owned HTTP/HTTPS, clipboard, and multi-result safety policy
- lazy creation and destruction of the Svelte settings webview
- atomic, versioned local settings
- opt-in payload-free performance diagnostics

Smart Scroll, webcam capture, history, updating, and broad UI redesign remain
deferred.

## Documentation

- [Development and builds](docs/development.md)
- [Threat model](docs/threat-model.md)
- [Performance budgets](docs/performance-budgets.md)
- [Phase 1 completion report](docs/phase-1-results.md)
- [Architecture decisions](docs/adr/)

## Quick validation

From the repository root:

```powershell
cargo fmt --all --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --workspace

Set-Location apps\desktop
npm ci
npm run format:check
npm run lint
npm run typecheck
npm run test
npm run build
npm run tauri -- build --bundles nsis
```

See the development guide for the required Windows C++/CMake toolchain and the
repository-owned MSVC runtime configuration used by ZXing-C++.
