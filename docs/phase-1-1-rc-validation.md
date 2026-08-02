# QRForge Phase 1.1 release-candidate validation

This is a manual test matrix for a Windows release candidate. It is not evidence that any test below has been performed. Every row starts as **Not run** and must retain that status until a tester records the actual result and evidence from the named configuration.

## Test record conventions

- Use the exact RC version, installer filename, Windows build, hardware model, display connection type, scaling, and assistive-technology version in **Configuration**.
- Record screenshots, screen recordings, logs, checksums, or issue reproductions in **Evidence** without including sensitive QR payloads.
- Set **Pass/fail** to `Pass`, `Fail`, or `Blocked` only after execution. Link every failure or blocker to an issue.
- Confirm that diagnostics are enabled only when a test explicitly needs them, then remove any local diagnostic file after evidence is captured.

## Multi-monitor configurations

| Configuration | Steps | Expected result | Actual result | Pass/fail | Evidence | Tester | Date | Issue link |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Two monitors at 100% scaling | Select each monitor in turn; place a QR on each; scan from the hotkey and tray. | Only the selected physical monitor is captured; chooser/action matches its QR. | Not run | Not run | — | — | — | — |
| Primary 100% + secondary 125% | Select the secondary; scan QRs near all four edges and center. | Physical bounds are correct with no offset, clipping, or neighboring-display pixels. | Not run | Not run | — | — | — | — |
| Primary 100% + secondary 150% | Repeat edge and center scans on both displays. | Both selections decode correctly and preserve the selected monitor after Settings reopens. | Not run | Not run | — | — | — | — |
| Primary 100% + secondary 200% | Scan known QRs at each edge and center of the 200% display. | Capture covers the complete physical display and decodes without DPI offset. | Not run | Not run | — | — | — | — |
| Secondary left of primary | Arrange the secondary with negative X coordinates; select it and scan. | The left display is captured; negative virtual coordinates cause no fallback or crop. | Not run | Not run | — | — | — | — |
| Secondary above primary | Arrange the secondary with negative Y coordinates; select it and scan. | The upper display is captured; negative virtual coordinates cause no fallback or crop. | Not run | Not run | — | — | — | — |
| Portrait secondary | Rotate the secondary to portrait; select it; scan center and edge QRs. | Rotation and physical dimensions are correct; the full portrait frame is decoded. | Not run | Not run | — | — | — | — |
| Primary changed while QRForge runs | Select a monitor; change Windows primary display; scan without restarting. | Selection remains stable when available; primary fallback follows the new primary only when needed. | Not run | Not run | — | — | — | — |
| Selected monitor disconnected | Select the secondary; disconnect it; trigger one scan. | QRForge falls back once to the current primary and shows explicit non-sensitive feedback. | Not run | Not run | — | — | — | — |
| Monitor reconnected | Reconnect the same display after the fallback test; reopen Settings; scan it. | The display is enumerated again; the user can reselect it and capture succeeds. | Not run | Not run | — | — | — | — |
| Laptop display + external monitor | Test internal-only, extended, and external-only modes. | Enumeration, selection, fallback, and capture remain correct in each mode. | Not run | Not run | — | — | — | — |
| Display sleep and resume | Let both displays sleep; resume; trigger scans on the stored selection. | QRForge recovers without restart, stale handles, crash, or hidden retained chooser. | Not run | Not run | — | — | — | — |
| Remote Desktop, if feasible | Connect through RDP; enumerate displays; scan a visible test QR; disconnect and resume locally. | Behavior is documented as supported or gracefully unavailable; no crash or sensitive frame file is created. | Not run | Not run | — | — | — | — |

## Accessibility

| Configuration | Steps | Expected result | Actual result | Pass/fail | Evidence | Tester | Date | Issue link |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Windows Narrator | Enable Narrator; navigate Settings and a multi-result chooser; invoke every control. | Names, roles, values, grouping, status changes, and focus order are announced meaningfully. | Not run | Not run | — | — | — | — |
| NVDA, if available | Repeat the Narrator workflow with the current NVDA release. | All actionable controls and chooser result states are operable and announced. | Not run | Not run | — | — | — | — |
| Windows High Contrast | Enable each available contrast theme; inspect Settings, onboarding, feedback, and chooser. | Text, focus indicators, selected state, warnings, and buttons remain visible and distinguishable. | Not run | Not run | — | — | — | — |
| Keyboard-only Settings | Use Tab, Shift+Tab, arrows, Space, and Enter; change and save every setting. | Logical focus order, visible focus, control operation, validation, and save feedback require no pointer. | Not run | Not run | — | — | — | — |
| Keyboard-only chooser | Open a multi-code chooser and use only the keyboard to inspect, copy, open, copy all, and dismiss. | Every permitted action is reachable; blocked actions remain disabled; order is predictable. | Not run | Not run | — | — | — | — |
| Focus restoration | Open and close onboarding/chooser from Settings using each close path. | Focus returns to the initiating or sensible Settings control; it is never lost behind a window. | Not run | Not run | — | — | — | — |
| Escape behavior | Press Escape from onboarding, Settings controls, and the chooser. | Escape dismisses only the appropriate dialog/window and never performs a result action. | Not run | Not run | — | — | — | — |
| Enter behavior | Press Enter on each focused button, text field, checkbox, select, and chooser action. | Enter activates only the focused/default action and never opens a blocked payload. | Not run | Not run | — | — | — | — |
| 200% display scaling | Run the complete keyboard workflow at 200% scaling. | Content reflows without clipping, horizontal traps, overlapping controls, or hidden focus. | Not run | Not run | — | — | — | — |
| Reduced motion | Enable Windows animation effects off/reduced motion; open and close all UI surfaces. | No essential state depends on animation and motion is absent or minimized. | Not run | Not run | — | — | — | — |
| Long Unicode payload | Scan a long payload containing multibyte scripts and emoji. | Preview truncates safely by characters, remains inert, and the UI stays responsive. | Not run | Not run | — | — | — | — |
| Long URL | Scan an overlong valid HTTPS URL and inspect/copy it. | Preview is bounded; Rust policy controls actions; no overflow or accidental navigation occurs. | Not run | Not run | — | — | — | — |
| Mixed blocked and allowed chooser | Scan multiple QRs containing allowed HTTP(S), blocked schemes, malformed text, plain text, and binary/control data. | Classification, labels, enabled actions, copy-all order, and blocked behavior match Rust policy. | Not run | Not run | — | — | — | — |

## Physical interaction

| Configuration | Steps | Expected result | Actual result | Pass/fail | Evidence | Tester | Date | Issue link |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Real physical hotkey | Trigger repeated one-shot scans with the configured physical key chord from several foreground apps. | One activation creates one scan; concurrent activation is rejected with clear feedback. | Not run | Not run | — | — | — | — |
| Hotkey conflict | Reserve the requested chord in another application; attempt to save it in QRForge. | Conflict is reported; the previous QRForge hotkey and persisted setting remain active. | Not run | Not run | — | — | — | — |
| Hotkey replacement and rollback | Save a valid new chord, verify it, then attempt a conflicting replacement. | New registration replaces the old exactly once; failed replacement restores the last working chord. | Not run | Not run | — | — | — | — |
| Tray Scan Now | Trigger scans from the tray with no code, one code, and multiple codes. | Each path produces the same policy and feedback as the hotkey path. | Not run | Not run | — | — | — | — |
| Tray Open Settings | Open Settings from a closed state and while it is already open. | One Settings webview exists and the existing window is shown and focused. | Not run | Not run | — | — | — | — |
| Closing Settings | Close Settings with the title-bar close control; trigger a scan from the tray. | Settings webview is destroyed while the tray app remains operational. | Not run | Not run | — | — | — | — |
| Reopening Settings repeatedly | Open and close Settings at least 25 times while observing Task Manager. | No duplicate windows, persistent hidden webviews, crash, or upward memory trend remains. | Not run | Not run | — | — | — | — |
| Explicit Quit | Quit from the tray with Settings and chooser both open. | Process, tray icon, webviews, and hotkey registration terminate promptly. | Not run | Not run | — | — | — | — |
| Launch at sign-in | Enable launch at sign-in; perform a real Windows sign-out and sign-in; then disable and repeat. | QRForge starts only when enabled and remains a single tray instance. | Not run | Not run | — | — | — | — |
| Install/reinstall/uninstall lifecycle | Install the RC, launch, reinstall or upgrade over it, verify settings behavior, quit, and uninstall. | Installer paths and shortcuts work; upgrade behavior is recorded; uninstall removes app files without claiming undocumented data cleanup. | Not run | Not run | — | — | — | — |

## Security and privacy

| Configuration | Steps | Expected result | Actual result | Pass/fail | Evidence | Tester | Date | Issue link |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `https:` URL | Scan a standard HTTPS URL and choose Open. | Rust approves and opens exactly that URL through the default browser. | Not run | Not run | — | — | — | — |
| Optional `http:` URL | Confirm the documented policy; scan an HTTP URL and choose Open if offered. | Behavior matches the configured/documented Rust policy and is clearly distinguished from HTTPS. | Not run | Not run | — | — | — | — |
| `file:` URL | Scan a file URL and attempt every offered action. | Open is unavailable; content is treated as blocked inert text subject to copy policy. | Not run | Not run | — | — | — | — |
| `javascript:` URL | Scan script-looking JavaScript scheme content. | It is never executed or opened; preview remains inert text. | Not run | Not run | — | — | — | — |
| `data:` URL | Scan a data URL containing HTML/script-looking content. | It is never interpreted, rendered as HTML, or opened. | Not run | Not run | — | — | — | — |
| Custom protocol | Scan a representative custom scheme such as `myapp:`. | Arbitrary protocol dispatch is blocked; no external application launches. | Not run | Not run | — | — | — | — |
| Malformed URL-like text | Scan malformed HTTP-like content with whitespace and invalid authority forms. | It is labeled malformed, never opened, and copied only as explicit inert text. | Not run | Not run | — | — | — | — |
| Unicode hostname | Scan an HTTP(S) URL with a Unicode hostname. | Auto-open/open is blocked according to spoofing policy; no normalization bypass occurs. | Not run | Not run | — | — | — | — |
| Punycode hostname | Scan an HTTP(S) URL with an `xn--` hostname. | Open is blocked according to spoofing policy and classification is clear. | Not run | Not run | — | — | — | — |
| Plain text | Scan ordinary text and choose Copy. | No browser opens; exact text reaches the clipboard only after the permitted action. | Not run | Not run | — | — | — | — |
| HTML/script-looking text | Scan tags, entities, and script-looking text as plain payload content. | UI displays escaped inert text; no markup, event, or script executes. | Not run | Not run | — | — | — | — |
| Multiple QR codes | Scan a frame containing multiple mixed-classification QRs. | No automatic action occurs; chooser order and Rust-owned permissions are correct. | Not run | Not run | — | — | — | — |
| No-code image | Trigger a scan on a display with no QR-family symbol. | Explicit no-code feedback appears without opening a chooser or logging display content. | Not run | Not run | — | — | — | — |
| Screenshot/frame persistence | Search app data, temp directories, install directory, and working directories before and after 25 scans. | No screenshot, frame, pixel dump, or captured-image file is created. | Not run | Not run | — | — | — | — |
| Sensitive payload log redaction | Enable diagnostics; scan unique secret-marker payloads across allowed and blocked classes; inspect every QRForge log. | No full marker or raw payload bytes appear; only bounded non-sensitive metadata is recorded. | Not run | Not run | — | — | — | — |

## Completion summary

| RC version | Windows build | Overall result | Open blocking issues | Evidence index | Lead tester | Completion date |
| --- | --- | --- | --- | --- | --- | --- |
| Not assigned | Not assigned | Not run | — | — | — | — |
