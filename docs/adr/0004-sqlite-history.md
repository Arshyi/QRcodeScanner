# ADR 0004: SQLite for local history

- Status: Accepted for later production implementation
- Date: 2026-07-12

## Decision

Use SQLite through a Rust repository adapter for history, search, favorites, retention, and migrations. Store decoded results and minimal metadata only. Never store screenshots, camera frames, window titles, or source application names.

Do not enable WAL until concurrent access measurements justify its additional sidecar files. History must be disableable, clearable, and subject to configurable retention.

## Consequences

- Embedded transactional storage without a service or network dependency.
- Local storage is not inherently encrypted.
- Any future encryption feature requires a separate key-management decision.

