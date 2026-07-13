# ADR 0001: Rust, Tauri, and Svelte desktop stack

- Status: Accepted with lazy-webview lifecycle constraint
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
- A real tray host survived ten create/destroy cycles with 0.38 MiB post-destroy resident-memory growth.
- Tauri must explicitly prevent implicit last-window exit while allowing programmatic shutdown.
- A webview is created only while the user-facing window is open; hiding a permanent webview is prohibited.

## Rejected alternatives

- Electron: mature but conflicts with the low-footprint mission.
- Qt/QML: capable native stack with a larger C++ and distribution surface.
- Separate native UIs: strongest platform fit but duplicates cross-platform product work.
