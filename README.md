# QRForge

QRForge is a local-first Windows tray utility that scans QR codes from a
selected display. Capture and decoding stay in native memory; the application
does not upload or persist screenshots.

## Phase 1.2 status

Windows release hardening is being validated on the Phase 1.1 screen-scanning
baseline. See the [Phase 1.2 report](docs/phase-1-2-results.md) for exact
automated, inspected, runtime, installer, and untested evidence. The 51-case
Phase 1.1 manual RC matrix remains `Not run` unless physically exercised.

The current scope is intentionally narrow:

- tray-first startup with explicit Quit
- one running host enforced by Tauri's platform single-instance plugin
- configurable global scan hotkey
- one-shot selected-monitor capture with primary fallback through xcap
- local QR-family decoding through bundled ZXing-C++
- Rust-owned HTTP/HTTPS, clipboard, and multi-result action policy
- a lazy, keyboard-operable multi-code result chooser
- first-run local-processing guidance and specific scan feedback
- lazy creation and destruction of Settings and result webviews
- atomic, versioned local settings
- explicit privacy-safe Copy Diagnostics and bounded opt-in local logs

Smart Scroll, webcam capture, history, updating, and broad UI redesign remain
deferred.

## Documentation

- [Development and builds](docs/development.md)
- [Windows release procedure](docs/release-procedure.md)
- [Installer behavior](docs/installer-behavior.md)
- [Diagnostics and logging](docs/diagnostics.md)
- [Dependency security and licensing](docs/dependency-security.md)
- [Signing readiness](docs/signing-readiness.md)
- [Phase 1.2 hardening report](docs/phase-1-2-results.md)
- [Threat model](docs/threat-model.md)
- [Performance budgets](docs/performance-budgets.md)
- [Phase 1.1 completion report](docs/phase-1-1-results.md)
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
