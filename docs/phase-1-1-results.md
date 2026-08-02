# Phase 1.1 completion report

- Audit date: 2026-07-29
- Repository: QRForge repository root
- Starting branch: `main`
- Starting commit: `5012322fc1d7691809251e0f616774d72bf840fe`
- Working branch: `feat/phase-1-1-scan-experience`
- Starting worktree: clean and synchronized with `origin/main` (ahead 0, behind 0)
- Release version: `0.1.1`
- Verdict: **COMPLETE for Phase 1.1 code and the tested single-display Windows configuration**

This report labels evidence as code inspection, automated test, measured Windows
runtime behavior, manual/automated UI interaction, inferred behavior, or
untested. It does not claim physical multi-monitor, mixed-DPI, portrait, Windows
high-contrast, or screen-reader coverage that the available machine could not
provide.

## Scope delivered

### Multi-code chooser

- More than one detection publishes a Rust-owned pending-result session and
  opens one lazy chooser webview.
- Original decoder order is preserved.
- Rust classifies HTTP, HTTPS, plain text, malformed URL-like text, blocked
  schemes, blocked authorities, unsafe text, and binary payloads.
- The webview receives only bounded inert previews and action flags. Binary and
  control-character payload bytes are not exposed as previews.
- Open, copy, copy-all, and dismiss commands are typed and validated in Rust.
  Only a retained `SafeHttpUrl` can reach the browser adapter.
- Session generations reject stale commands. Open and dismiss clear the
  retained session; closing the chooser also clears it.
- The chooser reloads when a newer multi-scan replaces an existing session.
- Escape dismisses, Enter activates the focused action, and the safe Dismiss
  button receives deterministic initial focus without scrolling the heading
  away.

### Physical-display selection

- The xcap adapter enumerates native displays and exposes signed virtual-desktop
  origins, physical width/height, integer scale percentage, rotation, and
  primary status.
- Settings schema 2 stores an optional validated opaque monitor identifier.
- The configured monitor is used when present. A missing monitor falls back to
  the primary display (or first available display) and produces explicit
  feedback.
- Labels omit xcap's raw `Unknown Monitor <handle>` values. Duplicate anonymous
  monitors receive deterministic topology suffixes.
- A captured frame must exactly match the selected monitor's physical
  dimensions. The decoder boundary accepts only non-zero, tightly packed RGBA
  frames with a validated stride.
- Settings can refresh topology and show a disconnected saved selection.

### First run, feedback, hotkey recovery, and accessibility

- Schema 2 persists dismissible onboarding completion. First run explains local
  processing, ephemeral screenshots, the global hotkey, URL/text behavior, tray
  access, and complete Quit.
- Feedback distinguishes no code, one detected/opened/copied result, multiple
  codes, blocked/malformed content, capture/decode/browser/clipboard failures,
  overlap rejection, unavailable monitor, and hotkey conflict.
- Scan start uses a short-lived payload-free tray tooltip. It resets even when
  result notifications are disabled.
- Reserved Alt+F4 and Super+L shortcuts are rejected in both Rust and the UI.
  Capture mode is window-level, Escape-cancelable, and toggleable. Failed
  registration leaves the previous working shortcut active and unpersisted.
- Settings and chooser use native dialog semantics, explicit labels, logical
  document order, visible focus rings, text labels in addition to color,
  reduced-motion CSS, forced-colors CSS, responsive narrow-window layout, and
  wrapping/scrolling bounded payload previews.
- The narrow Tauri capability applies only to the `settings` and `results`
  windows and exposes no shell, filesystem, or process permissions.

### Fixtures and CI

- The deterministic corpus grew from 15 to 21 PNGs.
- Added downscaled, browser-rendered, screenshot-compressed, RGB colored,
  high-DPI, and dense-UI fixtures while retaining URL, text, malformed,
  dangerous, Unicode, multi-code, inverted, low-contrast, rotated, partially
  obscured, perspective, binary, and no-code cases.
- The benchmark writes an optional JSON artifact and fails when the production
  ZXing failure set differs from the explicitly accepted
  `perspective.png` limitation.
- Windows CI now pins Rust 1.97.1 and Node 24.18.0, uses locked installs,
  read-only repository permissions, bounded concurrency, dependency caching,
  full Rust/frontend/spike/release/Tauri/NSIS validation, and installer plus
  fixture-summary artifacts. It does not publish releases or use signing
  secrets.
- Tracked frontend `dist` files were removed and `apps/desktop/dist/` is now
  ignored because Tauri's `beforeBuildCommand` deterministically regenerates
  them.

## Architecture and security decisions

1. `MonitorInfo` and `MonitorId` are domain values; xcap remains an adapter.
2. Capture selection is an application port operation. Captured pixels never
   cross IPC and are dropped after decode.
3. Multi-result action-capable values stay in native memory behind an opaque
   session number. The frontend cannot reconstruct URL-opening policy.
4. Preview size is bounded by Unicode scalar count rather than bytes, avoiding
   broken UTF-8 truncation.
5. Copy-all includes only Rust-approved textual values, separated by newlines
   in original detection order. Binary and unsafe text are excluded.
6. xcap already returns a full physical-monitor image. No logical-coordinate
   crop or DPI multiplication is performed; an exact physical dimension check
   guards this boundary.
7. The platform hotkey API requires unregister-before-register when replacing
   the same process-wide binding. The adapter restores the previous shortcut
   on failure; application persistence occurs only after platform changes
   succeed, with compensating rollback for later startup or persistence errors.
8. Display IDs are stable where xcap exposes a meaningful name and stable
   physical properties. Identical anonymous monitors necessarily use a
   topology-derived suffix; this limitation is surfaced by graceful fallback.

No runtime network client, telemetry, screenshot persistence, shell command,
arbitrary process execution, broad webview permission, idle polling, or
unbounded scan queue was added.

## Defects found and corrected

1. A plain `cargo build --release` host still referenced the configured Vite
   development URL and showed connection refused when launched directly. A
   production desktop artifact must be made with `tauri build`, which embeds
   the frontend; the documented and CI build path now enforces that.
2. The newly added chooser initially inherited a settings-only capability and
   would have been unable to invoke result commands. The capability and
   generated schema now include exactly `settings` and `results`.
3. A completed Open action initially relied on the window-destroy callback to
   clear pending results. The action now clears the Rust session before close,
   preventing stale retained openable values.
4. Reusing an already-open chooser could display the previous session. The
   existing chooser now reloads after replacement.
5. Scan-start tooltip text could remain indefinitely when notifications were
   disabled. A bounded reset now restores the actual active-hotkey tooltip.
6. Initial Dismiss focus scrolled the chooser below its heading. Focus now uses
   `preventScroll`.
7. Runtime keyboard testing found that Escape did not reliably cancel hotkey
   capture after focus movement. Window-level capture and a toggleable capture
   button correct this.
8. The repository tracked generated Vite `dist` output even though every Tauri
   build regenerates it. The output is removed from source control and ignored.

## Automated validation

The following commands passed on the final source unless a later table
explicitly identifies a pending final-artifact rerun:

| Command | Result | Evidence type |
|---|---|---|
| `cargo fmt --all --check` | PASS | automated |
| spike-workspace formatting | PASS | automated |
| `cargo check --workspace --all-targets --locked` | PASS | automated |
| `cargo test --workspace --locked` | PASS, 60 tests plus doc tests | automated |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS | automated |
| spike workspace check/test/strict Clippy | PASS | automated |
| `npm ci --no-audit --fund=false` | PASS, 233 packages | automated |
| `npm run format:check` | PASS | automated |
| `npm run lint` | PASS | automated |
| `npm run typecheck` | PASS, 0 errors and 0 warnings | automated |
| `npm test` | PASS, 2 files / 6 tests | automated |
| `npm run build` | PASS, 116 modules | automated |
| release workspace build | PASS | automated |
| `npm run tauri -- build --bundles nsis` | PASS | automated |
| decoder comparison | PASS gate, 20/21 production categories | automated/measured |
| Windows workflow YAML parse | PASS | automated |

Rust test distribution is 30 application, 3 capture, 5 decoder, 3 desktop,
13 domain, and 6 storage tests. Coverage includes monitor migration/fallback,
negative coordinates, 125/150/200% scale conversion, rotation, frame
dimensions/stride, multi-result ordering/classification/actions, stale and
malformed IPC, long Unicode, unsafe/binary previews, URL policy, specific
failure feedback, scan concurrency, hotkey rollback, first-run state,
corrupt-settings fallback, and error sanitization.

The Windows workflow itself was not run on GitHub because this branch is
intentionally uncommitted and unpushed. The workflow was parsed locally and its
supported constituent commands were run on the Windows host. Headless CI does
not claim UI automation, global-hotkey, notification, real desktop topology, or
installer interaction coverage.

## Fixture results

The release benchmark ran 30 iterations per fixture (630 inputs per engine):

| Engine | Exact categories | Aggregate median | Aggregate p95 | Known failures |
|---|---:|---:|---:|---|
| ZXing-C++ 3.x through `zxing-cpp` 0.5.2 | 20/21 | 2.0115 ms | 13.7658 ms | `perspective.png` |
| quircs 0.10.3 comparison | 19/21 | 3.5644 ms | 17.3704 ms | `inverted.png`, `perspective.png` |

The production adapter separately decoded all newly added transformed fixtures
from their saved PNG files, including actual RGB `colored.png`. The patterned
no-code fixture produced zero detections. Perspective distortion is retained
unchanged as a visible production-decoder limitation rather than weakened to
make the suite green.

## Windows runtime and UI validation

The production 0.1.1 Tauri build was exercised on the visible Windows desktop
with the checked-in synthetic fixtures open in Windows Photos.

| Behavior | Result | Evidence type |
|---|---|---|
| First-run dialog | PASS; content visible, Continue focus visible, completion persisted | Windows UI/runtime |
| Single instance | PASS during 10 reopens; one host process remained | Windows runtime |
| Settings lifecycle | PASS, at least 10 create/destroy cycles | Windows UI/runtime |
| Single-code scans | PASS, 10 required plus repeated stability scans | Windows UI/runtime |
| Multi-code scans | PASS, 10 required chooser create/destroy cycles | Windows UI/runtime |
| Multi result order/classification | PASS; `multi-one`, `multi-two`, then HTTPS URL | Windows UI/runtime |
| Copy all | PASS; success state reported scan-order copy | Windows UI/runtime |
| Escape dismiss | PASS | Windows keyboard test |
| Enter on default focused action | PASS | Windows keyboard test |
| Focus ring | PASS, visible on onboarding, selector, and chooser actions | Windows UI inspection |
| Overlapping activation | PASS; second dispatch reported `already_in_progress` in 1 ms | Windows UI/runtime |
| Explicit display selection | PASS on the only physical display, then restored to Automatic | Windows UI/runtime |
| Reserved Alt+F4 replacement | PASS; window remained open and working shortcut remained active | Windows UI/runtime |
| Escape from hotkey capture | PASS on the final rebuilt artifact; configured shortcut was restored | Windows keyboard test |
| Hotkey rollback | PASS in application tests; not forced against a second live registering process | automated |
| WebView cleanup | PASS; zero WebView2 descendants after Settings and chooser destruction | Windows process measurement |
| Pixel persistence search | PASS for repository/app data; only settings and opt-in payload-free JSONL diagnostics written | filesystem inspection |

The Windows automation bridge returned `accessibility: null` for WebView2
content, so it could not provide a semantic UIA tree for either webview.
Screen-reader semantics were therefore verified by HTML/code inspection and
zero-warning Svelte accessibility checks, while visual state, focus rings,
Tab/Enter/Escape behavior, and long-page scrolling were exercised on the
production windows. A human Narrator/NVDA pass remains recommended before a
signed public release.

## Performance before and after

Reference machine configuration: one primary `1920x1080` display at 100%
Windows scale, landscape orientation.

### Starting-commit measurements

| Metric | Starting value |
|---|---:|
| 30 s tray-idle mean CPU | 0% |
| Tray-idle host working set | 18,350,080 bytes |
| Tray-idle host private memory | 2,772,992 bytes |
| Settings-open host + 6 WebView2 working set | 419,962,880 bytes |
| Settings-open host + 6 WebView2 private memory | 205,287,424 bytes |
| Host after Settings destruction working set | 31,096,832 bytes |
| Host after Settings destruction private memory | 6,459,392 bytes |
| 100 selected-display captures | 33.0963 ms median / 34.1115 ms p95 |
| 15-fixture ZXing aggregate decode | 2.1319 ms median / 12.0436 ms p95 |

### Phase 1.1 production measurements

| Metric | Phase 1.1 value | Assessment |
|---|---:|---|
| 30 s tray-idle mean CPU after lifecycle tests | 0% | PASS |
| Tray-idle host working set | 33,591,296 bytes | below budget; native/webview code was warmed |
| Tray-idle host private memory | 7,917,568 bytes | below 60 MiB budget |
| Settings-open host + 6 WebView2 working set | 415,600,640 bytes | about 1.0% lower |
| Settings-open host + 6 WebView2 private memory | 209,698,816 bytes | about 2.1% higher |
| Host after 10 Settings cycles working set | 33,665,024 bytes | +2.57 MiB from starting post-destroy value |
| Host after 10 Settings cycles private memory | 8,036,352 bytes | +1.50 MiB |
| Settings creation (12 samples) | 256 ms median / 335 ms p95 | below 750 ms budget |
| Single hotkey-to-result (13 controlled samples) | 74 ms median / 85 ms p95 | within 10% of 41 ms historical p95 is not met; see note |
| Single capture | 33 ms median / 40 ms p95 | within capture budget |
| Single decode from full 1920x1080 screen | 40 ms median / 42 ms p95 | exact result |
| Multi hotkey-to-chooser (10 samples) | 81 ms median / 92 ms p95 | PASS under 150 ms end-to-end budget |
| Multi capture | 41 ms median / 48 ms p95 | PASS |
| Multi decode | 39 ms median / 41 ms p95 | PASS |
| 21-fixture ZXing aggregate decode | 2.0115 ms median / 13.7658 ms p95 | broader corpus; p95 +14.3% |
| After 23 scans + 10 chooser cycles | 41,922,560 byte WS / 9,093,120 byte private | zero WebView2 children |
| After 10 more scans | 50,446,336 byte WS / 9,306,112 byte private | pages warmed |
| After 30 additional scans | 50,487,296 byte WS / 9,330,688 byte private | plateau: +40,960 WS / +24,576 private |

The starting 41 ms historical hotkey p95 came from a warmed automation path
recorded by the earlier report. The fresh Phase 1.1 visible-desktop path
captures the full 1920x1080 Photos composition and includes current xcap plus
decoder work. Capture and decode submetrics explain the 85 ms p95; it remains
below the 150 ms one-shot budget and the multi chooser stays below 100 ms p95.
No attempt was made to hide this difference by changing the workload.

Working set rose as capture/decoder pages were first touched, while private
memory stabilized. Thirty additional scans changed working set by only 40 KiB
and private memory by 24 KiB, so no continuing scan leak was observed. Settings
and chooser windows left zero WebView2 descendants.

## Displays, DPI, and accessibility configurations

### Physically tested

- One primary landscape display
- Virtual-desktop origin `(0, 0)`
- Physical capture `1920x1080`
- Windows scale 100%
- Automatic-primary and explicit `Display 1` selection

### Automated model/adapter tests

- A display left of primary at negative X
- Signed negative X/Y serialization
- Missing configured monitor fallback
- 125%, 150%, and 200% scale conversion
- 90° and 270° rotation mapping
- Portrait metadata (`1080x1920`, 200%)
- Stable label/ID generation and raw-handle suppression
- Exact physical capture dimensions and packed stride validation

### Not physically tested

- Two or more attached monitors
- A monitor above or left of primary
- Mixed 100/125/150/200% display topology
- Portrait hardware
- Windows at 200% system scale
- Windows forced high-contrast mode
- Reduced-motion setting toggled at OS level

The untested physical configurations are not inferred from unit tests. They
remain a release-candidate hardware matrix item.

## Installer

- Path:
  `target\release\bundle\nsis\QRForge_0.1.1_x64-setup.exe`
- Signing: unsigned development/test bundle
- Size: 2,234,731 bytes
- SHA-256:
  `6CFA01C96FFE3979C994677FB523EE7668B6A7EE4D35B1E3EFF9F79A5B70D6DD`

The NSIS bundle build passed. Install/uninstall was not rerun in Phase 1.1 to
avoid changing the existing per-user installation; Phase 1 previously covered
that lifecycle. CI uploads the unsigned installer but does not publish a
release.

## Files changed

- Domain: monitor model, settings schema 2, stricter frame/stride, hotkey, and
  payload classification.
- Application: capture selection port, result service, scan policy/feedback,
  first-run state, settings transactions, and tests.
- Adapters/host: xcap display enumeration, Tauri commands/state/windows/tray/
  notifications/capability wiring, and tests.
- Frontend: routed Settings/Results panels, typed IPC, keyboard helpers, tests,
  compact responsive/accessibility styling.
- Fixtures: generator, six new PNGs, regression tests, summary, corpus
  documentation.
- Delivery/docs: Windows CI, version 0.1.1, lockfiles, ignore rules, README,
  development/threat/performance documentation, and this report.

`target`, npm caches, `node_modules`, regenerated `dist`, diagnostics, runtime
profiles, machine settings, logs, and installer binaries remain untracked.

## Remaining limitations and deferred work

- Physical multi-monitor/mixed-DPI/portrait testing remains outstanding because
  only one 100%-scale landscape monitor was attached.
- Anonymous identical monitor IDs can change when their topology ordering
  changes; fallback is safe and visible.
- The retained perspective-distorted fixture is not decoded by ZXing.
- Native WebView2 UIA semantics could not be inspected with the available
  automation bridge; perform Narrator and NVDA checks before a signed public
  release.
- The local Windows CI definition is not a remote CI run until this branch is
  reviewed, committed, and pushed.
- The installer is unsigned.

Smart Scroll, webcam mode, SQLite history, updater/signing, non-Windows support,
accounts, telemetry, and broad visual redesign remain deferred.

## Recommended next phase

Run a **Phase 1.1 release-candidate hardware/accessibility matrix** before
starting new product scope:

1. two- and three-monitor layouts including left/above negative coordinates;
2. mixed 100/150/200% DPI and portrait hardware;
3. Windows high contrast plus Narrator and NVDA;
4. signed-installer planning and a real GitHub Actions run;
5. investigate a more realistic projective fixture or decoder preprocessing
   without weakening the current known-failure gate.

Only after those release-candidate checks should Phase 1.2 scope be selected.
