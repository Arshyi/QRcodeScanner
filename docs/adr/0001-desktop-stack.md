# ADR 0001: Rust, Tauri, and Svelte desktop stack

- Status: Proposed pending Phase 0 resource measurements
- Date: 2026-07-12

## Context

QRForge requires native screen, webcam, hotkey, tray, clipboard, startup, and permission integration while remaining effectively idle between scans. The UI is settings- and history-oriented rather than graphics-heavy.

## Decision

Use Rust for the trusted core and platform adapters, Tauri 2 for desktop lifecycle and webview hosting, and Svelte 5 with TypeScript for the eventual UI. Captured images and QR decoding remain native. Only small typed results cross IPC. The tray-idle state should avoid retaining a webview if measurements show material savings.

## Consequences

- Rust provides memory safety, strong typing, and direct native API access.
- Tauri uses the OS webview instead of bundling Chromium.
- WebView2 may still have substantial process-tree memory cost.
- Contributor setup on Windows requires Rust and MSVC tooling.
- The decision remains conditional until host-only and webview idle measurements are recorded.

## Rejected alternatives

- Electron: mature but conflicts with the low-footprint mission.
- Qt/QML: capable native stack with a larger C++ and distribution surface.
- Separate native UIs: strongest platform fit but duplicates cross-platform product work.

