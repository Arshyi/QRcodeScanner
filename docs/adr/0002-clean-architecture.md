# ADR 0002: Clean architecture with platform ports

- Status: Accepted
- Date: 2026-07-12

## Decision

Use four dependency layers:

1. Domain types and policies with no framework dependencies.
2. Application use cases depending on abstract ports.
3. Infrastructure/platform adapters implementing capture, decoder, persistence, browser, clipboard, hotkey, and camera ports.
4. Tauri composition and UI exposing a narrow typed IPC boundary.

Crates represent genuine ownership, platform, or compilation boundaries—not individual classes. Native image buffers never cross into the webview.

## Consequences

- Safety and business policies are deterministic and hardware-independent in tests.
- Platform adapters remain replaceable.
- Interfaces require deliberate buffer ownership, cancellation, and error semantics.
- Tauri, SQLite, and decoder-specific types cannot leak into the domain.

