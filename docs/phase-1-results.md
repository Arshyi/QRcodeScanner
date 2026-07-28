# Phase 1 completion report

- Audit date: 2026-07-29
- Repository: `C:\Users\DELL\Desktop\QRcodeScanner`
- Starting commit: `3764877a5ac3d43be5ca3bc3accf8d768be18706`
- Branch at takeover: `main`, aligned with `origin/main`
- Starting worktree: clean
- Final verdict: **COMPLETE for the narrow Windows Phase 1 MVP**

This report supersedes the 2026-07-14 report. That report marked Phase 1
complete while also recording a failed Tauri build and unperformed lifecycle,
hotkey, fixture, installer, privacy, and performance checks.

Evidence below is labeled as automated test, host measurement, strongest
feasible automated Windows UI/runtime test, code inspection, or untested. No
inspection-only result is presented as a runtime test.

## Starting repository and Copilot-era work

The takeover note still named `E:\QRcodeScanner\QRcodeScanner`, but the user
had moved the repository to C:. `.git` existed, the worktree was clean, and
the checked-out commit matched `origin/main`.

Changes made after the older handoff snapshot were:

| Commit | Work discovered |
|---|---|
| `d6c6aca` | Changed the frontend lockfile and documentation, including the unsupported Phase 1 completion claim |
| `3764877` | Added capture monitor/scaling metadata, tray-tooltip changes, and a custom file-open single-instance attempt |

Copied Release artifacts from 2026-07-14 were present, but they predated
`3764877` and were not accepted as evidence.

## Defects found

1. The custom `single_instance.rs` only opened a file. It acquired no
   exclusive Windows lock, so it did not enforce one host.
2. The starting `cargo fmt --all --check` failed in `tray.rs`.
3. The tray was created before initial hotkey registration, producing a stale
   initial tooltip.
4. Notification reset restored a hard-coded `Ctrl+Shift+Q` tooltip after a
   user selected another shortcut.
5. Every activation spawned a thread before overlap rejection, allowing
   unbounded short-lived workers during hotkey spam.
6. UTF-8 control characters were classified as copyable plain text.
7. Credential-bearing and IDN/punycode HTTP(S) URLs were eligible for
   automatic opening.
8. Zero-width and zero-height captured frames passed validation.
9. Malformed settings fallback lived only in the Tauri composition root and
   was not a consistent storage behavior.
10. `core:default` and a permissive CSP exposed frontend authority and sources
    the Settings webview did not need.
11. `cmake\msvc-runtime.cmake` existed but Cargo did not reproducibly pass it
    to bundled ZXing-C++.
12. A clean npm install omitted the native Windows Tauri CLI package, so
    `tauri build` failed despite a lockfile being present.
13. The C: host initially lacked Rust, MSVC, the Windows SDK, and CMake.

## Corrections and improvements

- Replaced the ineffective custom lock with Tauri's platform
  `single-instance` plugin, registered first. A duplicate opens or focuses
  Settings and exits.
- Added a one-worker `ScanDispatcher`; overlap is rejected before another OS
  thread is created.
- Registered the startup hotkey before creating the tray.
- Centralized tray idle-tooltip generation and refresh it after settings saves
  and notification feedback.
- Added zero-dimension frame rejection and encapsulated capture metadata.
- Added a non-copyable `UnsafeText` class for empty/control-character text.
- Blocked credential-bearing and IDN/punycode HTTP(S) destinations from
  automatic opening while retaining optional data-only clipboard behavior.
- Moved malformed-JSON fallback into the settings repository without deleting
  the corrupt source; added Windows replacement and migration coverage.
- Reduced the Settings capability to its registered app commands, removed
  unused core permissions, and tightened CSP to local assets and Tauri IPC.
- Added `.cargo/config.toml`, passing the checked-in CRT policy through
  `CMAKE_TOOLCHAIN_FILE`.
- Pinned `@tauri-apps/cli-win32-x64-msvc` 2.11.4 as an explicit optional
  dependency so `npm ci` reproduces the Windows package build.
- Added deterministic plain-text and malformed-URL fixtures and regression
  tests.
- Replaced stale status documentation and added a Windows development/build
  guide.

## Toolchain used

| Component | Verified version |
|---|---|
| Rust | `rustc 1.97.1` |
| Cargo | `cargo 1.97.1` |
| Node.js | `v24.18.0` |
| npm | `11.16.0` |
| Visual Studio Build Tools | 2022 `17.14.37`, complete and launchable |
| CMake | `3.31.6-msvc6` |
| Tauri CLI | `2.11.4` |

Rustup was installed from the official x64 installer after its published
SHA-256 was verified. Visual Studio Build Tools was installed with the
Desktop C++ workload, x64 MSVC toolset, Windows SDK, and CMake.

## Automated command evidence

All results below are from the final working tree.

| Command | Result | Evidence |
|---|---|---|
| `cargo fmt --all --check` | PASS | automated |
| spike-workspace `cargo fmt --all --check` | PASS | automated |
| `cargo check --workspace --all-targets` | PASS | automated |
| `cargo test --workspace` | PASS, 40 tests | automated |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS | automated |
| `cargo build --release --workspace` | PASS | automated |
| clean `npm ci` | PASS, 233 packages | automated |
| `npm run format:check` | PASS | automated |
| `npm run lint` | PASS | automated |
| `npm run typecheck` | PASS, 0 errors and 0 warnings | automated |
| `npm run test` | PASS, 1 file / 3 tests | automated |
| `npm run build` | PASS, 113 modules | automated |
| `npm run tauri -- build --bundles nsis` | PASS | automated |
| decoder comparison corpus | PASS generation, 15 fixtures | automated |

The 40 Rust tests comprise 19 application, 4 decoder, 2 desktop-host, 10
domain, and 5 storage tests. All doc tests also pass.

## Native ZXing verification

ZXing-C++ 0.5.2 was removed from Cargo's Debug and Release outputs and rebuilt
through the completed standard Visual Studio generator. Both current CMake
caches report:

```text
CMAKE_GENERATOR:INTERNAL=Visual Studio 17 2022
CMAKE_MSVC_RUNTIME_LIBRARY:STRING=MultiThreadedDLL
CMAKE_TOOLCHAIN_FILE:FILEPATH=C:/Users/DELL/Desktop/QRcodeScanner/cmake/msvc-runtime.cmake
```

The generated `ZXing.vcxproj` uses `MultiThreadedDLL` in every configuration,
and compiler flags contain `/MD`, not `/MDd`. The toolchain file remains
necessary because Rust's MSVC Debug profile also uses the release dynamic CRT.

## Windows runtime and UI validation

These tests used the final Release host. Checked-in fixtures were displayed
fullscreen. The strongest feasible automated harness sent `WM_HOTKEY` through
the plugin's actual registered `global_hotkey_app` receiver; the Settings UI
and tray tooltip independently confirmed `Ctrl+Shift+Q` was registered. This
tests the production hotkey callback, worker dispatch, screen capture, ZXing
decode, payload policy, clipboard/browser adapters, feedback, and
diagnostics. It is not a claim that a human physically pressed the keys.

| Behavior | Result | Evidence |
|---|---|---|
| Tray-first launch | PASS, one host and no idle WebView2 child | Windows runtime test |
| Tray tooltip | PASS, `QRForge — press Ctrl+Shift+Q to scan` | Windows accessibility inspection |
| Duplicate launch | PASS, second process exited 0 and focused Settings | Windows runtime test |
| Real hotkey registration | PASS, Settings showed `Active: Ctrl+Shift+Q` | Windows UI inspection |
| Real hotkey conflict | PASS, Windows reserved `Ctrl+Alt+M`; UI showed conflict; disk and active shortcut rolled back to `Ctrl+Shift+Q` | Windows UI/runtime test |
| Settings IPC | PASS, Notifications toggled off/on and both UI and JSON agreed | Windows UI/runtime test |
| Launch at sign-in adapter | PASS, toggled on/off; HKCU Run value appeared and was removed; final state is off | Windows UI/runtime test |
| Tray `Scan Now` | PASS, diagnostic trigger was `tray` | Windows UI/runtime test |
| Tray `Open Settings` | PASS | Windows UI/runtime test |
| Settings close | PASS, six WebView2 children returned to zero and tray host stayed alive | Windows runtime test |
| Tray `Quit` | PASS, host and WebView2 counts returned to zero | Windows UI/runtime test |
| Ten Settings cycles | PASS, each cycle was 6 WebView2 processes open and 0 after close; one host remained | Windows runtime test |
| Ten warmed scans | PASS, all completed with no overlapping worker | Windows runtime test |
| Deliberate overlap | PASS, exactly one `already_in_progress` result and one completed capture | Windows runtime test |
| No capture persistence | PASS, app data contained only `settings.json` and opt-in `diagnostics.jsonl`; no image/frame-like files | filesystem inspection plus source inspection |

Representative scan results:

| Fixture | Expected policy result | Observed |
|---|---|---|
| normal HTTP(S) URL | open approved URL | `url_opened`, 1 detection |
| plain text | copy as data | `text_copied`, exact clipboard match |
| multiple | no automatic action | `multiple_codes`, 3 detections |
| false-positive background | no code | `no_code`, 0 detections |
| inverted | decode and copy text | `text_copied`, exact clipboard match |
| Unicode | block custom-scheme interpretation but copy as data | `blocked_payload_copied`, exact clipboard match |
| `javascript:` | never open; optional copy only | `blocked_payload_copied`, exact clipboard match |
| malformed `https://[invalid` | treat as plain text | `text_copied`, exact clipboard match |

The normal fixture opened `https://example.com/qrforge/normal` through the
system browser. The dangerous, malformed, multiple, and no-code cases did not
open a destination.

## Performance measurements on this host

Measurements used the final Release build on the primary 1920x1080 display at
100% scaling.

| Metric | Measured result |
|---|---|
| host-only idle CPU | 0.000 CPU seconds over 30.016 seconds; 0.00% of one core |
| host-only idle working set | 40.70 MiB after fixture/window warm-up |
| host-only idle private bytes | 7.63 MiB |
| host-only process count | 1 host, 0 WebView2 |
| Settings-open aggregate working set | 374.43 MiB |
| Settings-open aggregate private bytes | 197.00 MiB |
| Settings-open process count | 1 host + 6 WebView2 = 7 |
| after Settings destruction | 1 host + 0 WebView2; 40.88 MiB working set / 7.48 MiB private |
| cold Settings creation | 310 ms |
| later Settings creation | 267 ms |
| cold first scan | 26 ms capture, 6 ms decode, 33 ms total, 34 ms hotkey-to-result |
| ten warmed scans, capture | 24.5 ms median / 34 ms p95 |
| ten warmed scans, decode | 6 ms median / 11 ms p95 |
| ten warmed scans, total use case | 32.5 ms median / 41 ms p95 |
| ten warmed scans, hotkey-to-result | 33 ms median / 41 ms p95 |

Across ten additional warmed scans, host working set changed from 40.88 MiB
to 40.86 MiB and private bytes from 7.48 MiB to 7.26 MiB. After notification
timers settled, the host used 40.65 MiB working set and 7.02 MiB private.
This run found no accumulating scan or WebView leak.

## Build and installer artifacts

Final artifacts:

| Artifact | Size | SHA-256 |
|---|---:|---|
| `target\release\qrforge.exe` | 7,437,312 bytes | `AEF7872A504CA14810664EE055CFE1952BB0277A011A56D839EFD1400402D235` |
| `target\release\bundle\nsis\QRForge_0.1.0_x64-setup.exe` | 2,202,859 bytes | `82EBDBA042905A4028024B94E02ADAF36C6E2DE7A8B914006EF58F75228126C8` |

The exact final installer was installed silently for the current user, created
the expected 0.1.0 uninstall registration, installed and launched one tray
host, and exited through its native Quit item. Its test installation was then
uninstalled successfully; the repository artifact remains. The installer and
executable are unsigned, as expected for Phase 1.

## Security and privacy result

- Pixel buffers remain in native memory and never cross the Settings IPC
  boundary.
- The only capture-path file writes are payload-free opt-in diagnostics.
- Full QR payloads and pixels are absent from diagnostics.
- Rust owns URL classification and action policy.
- Only normalized ASCII-host HTTP(S) URLs without credentials can auto-open.
- Control-sequence text and binary data are not copied.
- Multiple detections never trigger browser or clipboard actions.
- No generic shell-execution adapter exists.
- The webview loads local content under a restrictive CSP and a single
  Settings capability.

## Files changed

- Host/configuration: `.cargo/config.toml`, Tauri Cargo/config/capability files,
  commands, composition root, runtime dispatcher, notification and tray code;
  the ineffective `single_instance.rs` was removed.
- Core crates: scan policy, frame validation, payload classification, decoder
  regression coverage, and settings storage.
- Frontend packaging: `package.json` and `package-lock.json`.
- Fixtures: generator, README, `plain-text.png`, and `malformed-url.png`.
- Documentation: root README, development guide, threat model, and this report.

## Remaining limitations and deferred work

- Phase 1 captures only the primary monitor; mixed-DPI monitor selection is
  deferred.
- Multiple detections deliberately produce no automatic action; there is no
  chooser yet.
- The perspective-stress fixture remains a known ZXing limitation.
- A valid approved HTTP(S) destination is still an untrusted internet site.
- The installer is unsigned and therefore not ready for broad public
  distribution.
- Physical-keyboard input and an actual Windows sign-out/sign-in cycle were
  not performed; registration, callbacks, conflict rollback, and HKCU startup
  state were tested through the strongest automated Windows harness available.
- Smart Scroll, webcam UI, history, updater infrastructure, and broad visual
  redesign remain deferred.

## Recommendation

Proceed to a small Phase 1.5 hardening release: add repeatable Windows CI,
code signing/release provenance, accessibility checks, mixed-monitor/DPI
coverage, and a user-confirmed multi-code chooser. Do not begin Smart Scroll
or webcam work until those release-quality foundations are in place.

The implementation audit ended with this Phase 1 work uncommitted and
unpushed; publication is handled separately.
