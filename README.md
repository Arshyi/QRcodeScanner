# QRForge

A lightweight desktop QR code scanner that instantly detects QR codes from your screen and opens them in your browser—no screenshots or uploads required. Works globally with a hotkey, decodes offline locally, and operates in the system tray with minimal resource usage.

## Status: Phase 1 MVP Complete ✓

QRForge is now a **usable Windows MVP** with:
- Tray-first application lifecycle
- Global hotkey scan (Ctrl+Shift+Q by default, customizable)
- One-shot primary screen capture
- Local offline QR decoding via ZXing-C++
- Safe URL opening (http/https only)
- Plain-text and blocked-scheme handling
- Native notifications
- Persistent settings
- Comprehensive test coverage (37 tests passing)

See [Phase 1 Results](docs/phase-1-results.md) for detailed measurements and architecture.
