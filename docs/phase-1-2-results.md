# Phase 1.2 Windows release-hardening results

Status: implementation complete and locally validated; unsigned and not
published. GitHub-hosted release-candidate execution and the explicitly listed
physical/manual cases remain `Not run`.

## Repository baseline

- Repository: `C:\Users\DELL\Desktop\QRcodeScanner`
- Starting branch: `main`
- Starting/local/remote commit:
  `b341b4c6e660b86a4a0b95eec18079ef03ef47f5`
- Working branch: `feat/phase-1-2-release-hardening`
- GitHub authentication: verified for `Arshyi`
- Stale Phase 1.1 local branch: deleted only after its tip tree and `main` tree
  both resolved to `f7778a341f0ab5a76bdd5de66f639f26c5c168b5` and `git diff --quiet`
  returned success. The squash merge explains why commit ancestry alone was
  false.

The release candidate was built and validated from implementation commit
`a5cd633e748fd8c2296e15dd70e4b7cc9027ee82`. A later documentation-only
evidence commit does not change that candidate's bytes or manifest identity.

## Implemented hardening

- Version 0.1.2 with the root Cargo workspace version as the documented source
  and a fail-closed cross-manifest/Cargo-metadata consistency check.
- Compile-time Git commit identity in Settings and Copy Diagnostics.
- Privacy-safe Copy Diagnostics action implemented in Rust, plus stable recent
  error categories with bounded in-process retention.
- Opt-in JSONL logs changed from startup truncation/unbounded append to a 256
  KiB active log plus one archive. No polling, logging worker, network request,
  or keep-alive handle was added.
- Settings recovery coverage for stale partial files and failed atomic
  replacement, in addition to malformed-source preservation and existing
  startup/hotkey/chooser recovery tests.
- PowerShell version, dependency-policy, manifest/checksum, release-tooling,
  and full release-validation scripts. They do not install software, change
  global configuration, push, publish, tag, or sign.
- Separate least-privilege manual unsigned-RC workflow with full-SHA action
  pins, exact ref/commit/version inputs, 30-day evidence artifacts, and no
  repository write permission.
- Dependency, native decoder provenance, third-party notices, installer,
  release, diagnostics, and signing policies.
- Prettier accepts the checkout's existing line endings so a clean Windows
  repository with system `core.autocrlf=true` does not fail solely because
  tracked text is materialized as CRLF.

## Bugs discovered and corrected

1. The first clean release-validation checkout failed formatting because
   Prettier's implicit line-ending policy disagreed with normal Windows Git
   materialization. `endOfLine: auto` makes the check content-focused without
   changing tracked files.
2. `npm audit` found high-severity vulnerable `brace-expansion` versions under
   two `minimatch` generations. Narrow parent-scoped overrides select patched
   1.1.18 and 5.0.9 without a broad dependency refresh.
3. `cargo audit` initially found seven locked-graph findings: affected
   `quick-xml` generations and `time`. The reachable Windows plist path was
   upgraded to plist 1.10.0, quick-xml 0.41.0, and time 0.3.55. This raises the
   honest minimum Rust version to 1.88.
4. Existing diagnostic logging truncated on every launch and had no size
   bound. It now appends, rotates synchronously only when writing an event, and
   retains at most one 256 KiB archive.
5. Interrupted-settings recovery did not explicitly cover stale partial files
   or a Windows sharing-violation replacement failure. Focused tests now prove
   the committed settings remain authoritative and unchanged.

## Automated local validation

The full clean release validator passed on the candidate commit in 301.3
seconds. It ran or verified:

| Gate                                                                               | Result                                                                   |
| ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| Version consistency across seven workspace packages and repeated manifests         | PASS (`0.1.2`)                                                           |
| Invalid expected version                                                           | PASS (rejected)                                                          |
| Dirty-tree release state                                                           | PASS (rejected)                                                          |
| Rust formatting/check/all tests/doc tests/strict Clippy                            | PASS; 67 unit tests plus doc tests                                       |
| Spike formatting/check/test/strict Clippy                                          | PASS; generated-fixture test passed                                      |
| Frontend clean install/format/lint/typecheck/tests/build                           | PASS; 0 warnings/errors, 2 files / 6 tests                               |
| `npm audit`                                                                        | PASS; 0 vulnerabilities                                                  |
| npm lifecycle and license policy                                                   | PASS; only esbuild approved, fsevents denied, 292 packages checked       |
| `cargo audit` policy                                                               | PASS with two documented target-irrelevant exceptions described below    |
| `cargo deny` advisories/licenses/sources                                           | PASS; contextual workspace policy emitted 19 allowed transitive warnings |
| Release manifest/checksum/tooling tests                                            | PASS; invalid version and installer name rejected                        |
| Release workspace, Tauri Release, and NSIS build                                   | PASS                                                                     |
| Exact one-installer, checksum, manifest, unsigned label, and path-redaction checks | PASS                                                                     |
| `git diff --check` and source/artifact hygiene                                     | PASS                                                                     |

Rust test distribution was 30 application, 3 capture, 5 decoder, 8 desktop, 13
domain, and 8 storage tests. Diagnostics formatting/redaction, log append and
rotation, settings atomicity/recovery, startup and hotkey rollback, worker
cleanup, and chooser cleanup are covered by precise behavioral assertions.

The script's negative clean-state and version checks were exercised
independently. The final wrong-ref negative check is recorded after the
documentation-only evidence commit.

## CI validation

`windows-ci.yml` and `release-candidate.yml` were parsed and their supported
constituent commands passed locally. Actions are pinned to full commit SHAs and
the RC workflow has read-only contents permission. The branch is intentionally
unpushed, so no GitHub-hosted run or uploaded RC artifact exists and CI status
is `Not run`.

## Security and dependency evidence

- `npm audit`: zero vulnerabilities after the narrow brace-expansion fixes.
- The supported `x86_64-pc-windows-msvc` Cargo graph is clear of the discovered
  `quick-xml` findings. The remaining quick-xml 0.30/0.39 copies occur only
  beneath xcap's xcb/Wayland non-Windows build graph.
- Because `cargo audit` cannot target-filter, `.cargo/audit.toml` records
  time-bounded exceptions for `RUSTSEC-2026-0194` and `RUSTSEC-2026-0195`,
  owner QRForge release maintainer, review deadline 2026-11-01. They are not
  silently ignored.
- `cargo deny` scopes its graph to `x86_64-pc-windows-msvc`, denies unknown
  registries/Git sources and yanked crates, and enforces the reviewed license
  allow-list. Its 19 contextual warnings are transitive unmaintained/unsound
  packages, primarily non-Windows GTK paths plus `paste` through ZXing and
  Unicode dependencies through Tauri.
- ZXing remains `zxing-cpp` 0.5.2, a bundled static C++ core under Apache-2.0;
  source, version, linkage, and attribution are documented.
- Full-SHA GitHub Action pins, dependency review instructions, vulnerability
  response policy, and `THIRD_PARTY_NOTICES.md` are present.
- Diff and worktree scans found no secrets, certificates, keys, tokens,
  installers, executables, caches, logs, diagnostics, or machine paths staged.

## Release and installer evidence

The complete validator generated exactly these ignored artifacts:

- Installer: `target\release\bundle\nsis\QRForge_0.1.2_x64-setup.exe`
- Size: 2,238,807 bytes
- SHA-256:
  `C4FCFD8D1A0AB8A630A822F6B60A089536F3FE20494E1A7FFF1CDAE9603C9FD2`
- Checksum: `target\release\bundle\nsis\SHA256SUMS.txt`
- Manifest: `target\release\bundle\nsis\release-manifest.json`
- Manifest identity: version 0.1.2, commit
  `a5cd633e748fd8c2296e15dd70e4b7cc9027ee82`, Windows x86_64,
  `unsigned-release-candidate`
- Authenticode: installer and executable both `NotSigned`

The preserved Phase 1.1 baseline artifact was verified before use:

- Path:
  `target\ci-artifacts\main-30752264967\release\bundle\nsis\QRForge_0.1.1_x64-setup.exe`
- Size: 2,233,256 bytes
- SHA-256:
  `619E0647DB1A63BAE0C2370A090B0561FE519D92EEC2CA3D508F5B2183F3037D`

### Physically exercised installer matrix

| Case                                     | Observed result                                                           | Status  |
| ---------------------------------------- | ------------------------------------------------------------------------- | ------- |
| Baseline 0.1.1 install                   | Exit 0; version 0.1.1; executable/uninstaller and one Start Menu shortcut | PASS    |
| Launch after install                     | One tray host; a second launch opened Settings and exited 0               | PASS    |
| Forced termination/restart               | Forced exact host PID; restart produced one host and no orphan            | PASS    |
| 0.1.1 to 0.1.2 while running             | Exit 0; old host stopped; registry/executable became 0.1.2                | PASS    |
| Upgrade settings retention               | Distinctive Notifications=false settings SHA-256 unchanged                | PASS    |
| Upgrade startup retention                | Absent Run entry remained absent, matching saved false state              | PASS    |
| Same-version 0.1.2 install while running | Exit 0 in 1,755 ms; process 1 to 0; settings hash unchanged               | PASS    |
| Clean 0.1.2 program install              | Exit 0 in 1,295 ms; correct files/key/shortcut; no auto-launched process  | PASS    |
| Old-file cleanup                         | Install root contained only `qrforge.exe` and `uninstall.exe`             | PASS    |
| Sensitive-file search                    | No screenshot/frame/payload image extension in install or roaming data    | PASS    |
| Uninstall after use, stopped host        | Exit 0 in 1,497 ms; program/key/shortcut removed; data retained           | PASS    |
| Uninstall while tray host running        | Exit 0 in 2,097 ms; process 1 to 0; program/key/shortcut removed          | PASS    |
| User-data retention                      | `settings.json` hash unchanged; both roaming files remained               | PASS    |
| Tray-menu Quit                           | Automation helper could not target the notification area                  | Not run |

The installer creates:

- `%LOCALAPPDATA%\QRForge\qrforge.exe`
- `%LOCALAPPDATA%\QRForge\uninstall.exe`
- `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\QRForge`
- `%APPDATA%\Microsoft\Windows\Start Menu\Programs\QRForge.lnk`

The shortcut targets `%LOCALAPPDATA%\QRForge\qrforge.exe`; the uninstall string
targets the adjacent `uninstall.exe`. Settings and opt-in logs remain under
`%APPDATA%\app.qrforge.desktop` and are intentionally retained by uninstall.
The machine's pre-test settings/logs and absent startup entry were backed up,
then restored hash-for-hash after testing. QRForge is left uninstalled with no
running process.

## Crash and abnormal-shutdown recovery evidence

- Five additional installed-build forced-termination cycles all restarted to
  exactly one host, had zero direct children while tray-only, and settled to
  zero QRForge processes 21-30 ms after force termination. The harness's first
  attempt observed Windows's brief teardown race; adding a bounded settlement
  poll made the assertion deterministic and is a test-harness correction, not
  an application defect.
- Unit tests pass for scan-worker RAII cleanup after failure/panic, malformed
  settings, stale partial settings, failed atomic replacement preserving the
  original file, startup rollback, hotkey rollback, and chooser cleanup after
  creation failure.
- Code inspection confirms settings use same-directory temporary persistence,
  pending chooser state is process-local and session-scoped, the scan gate is
  RAII-owned, and logging has no keep-alive worker.
- Physical Windows shutdown, logout/sign-in, injected webview crash, and an
  actual inaccessible per-user directory remain `Not run`. Their lower-level
  recovery paths are tested but are not presented as physical OS evidence.

## Diagnostics and logging evidence

The production Settings UI showed version 0.1.2 and commit `a5cd633e748f`.
Copy Diagnostics completed in 75 ms. The clipboard contained 20 fixed-order
lines (514 characters) with SHA-256
`E22BD3427C6F30398A5F49FC6F37E61F55C81894E635A19C6A6C33977AA8ADF6`.
It reported the correct version/full commit/target/schema, registered-hotkey
boolean, disabled-startup boolean, safe automatic monitor state/count, ZXing
version, logical log location/retention, no recent errors, PID, and uptime.

A corrected case-insensitive privacy scan found zero usernames, absolute user
paths, environment assignments, URLs, image data, tokens/secrets, long
payload-like encodings, or private-key material. Only logical path text is
included. Diagnostics were generated directly into the clipboard as inert
text; no diagnostic file was created.

With explicit `QRFORGE_DIAGNOSTICS=1`, one startup plus one Settings
create/destroy lifecycle grew the existing log by 154 bytes. Startup/open alone
grew it by 99 bytes. Logs append across launches, rotate at 256 KiB to one
archive, exclude payloads/images/monitor labels, and perform no idle writes.
Rotation, append, redaction, stable formatting, and inaccessible-log graceful
degradation pass automated tests.

## Performance before and after

Reference machine: one physical display. Phase 1.1 comparison values are from
`docs/phase-1-1-results.md`; Phase 1.2 values below are fresh installed-candidate
measurements unless explicitly labeled benchmark-only.

| Metric                                         |                         Phase 1.1 |                                                     Phase 1.2 | Assessment                                 |
| ---------------------------------------------- | --------------------------------: | ------------------------------------------------------------: | ------------------------------------------ |
| 30 s tray-idle mean CPU                        |                                0% |                                         0% (0 ms sampled CPU) | no regression                              |
| Tray-idle host working set                     |                  33,591,296 bytes |                                                     31.79 MiB | no regression                              |
| Tray-idle host private memory                  |                   7,917,568 bytes |                                                      6.31 MiB | no regression                              |
| Settings-open total working set                |    415,600,640 bytes / 6 children |                                   171,335,680 bytes / 1 child | lower on this run                          |
| Settings-open total private memory             |                 209,698,816 bytes |                                              49,967,104 bytes | lower on this run                          |
| Host after Settings destruction working set    |                  33,665,024 bytes |                                              33,009,664 bytes | no regression                              |
| Host after Settings destruction private memory |                   8,036,352 bytes |                                               6,705,152 bytes | no regression                              |
| Settings creation                              |        256 ms median / 335 ms p95 |            311 ms cold sample; earlier upgraded sample 339 ms | below 750 ms budget; not a p95             |
| Single hotkey-to-result                        |          74 ms median / 85 ms p95 |                                                       Not run | physical fixture viewer unavailable        |
| Multi hotkey-to-chooser                        |          81 ms median / 92 ms p95 |                                                       Not run | physical fixture viewer unavailable        |
| ZXing 21-fixture aggregate decode              | 2.0115 ms median / 13.7658 ms p95 |                             1.8987 ms median / 12.0282 ms p95 | no regression; benchmark-only              |
| Startup                                        |                      not recorded | 12 ms internal; process visible in 66-135 ms over five cycles | informational                              |
| Tray Quit shutdown                             |                      not recorded |                                                       Not run | notification area not targetable by helper |
| Forced-exit settlement                         |                      not recorded |                                     21-30 ms over five cycles | recovery-only                              |
| Log growth                                     |                    not comparable |          154 bytes for startup/create/destroy; no idle growth | bounded/event-driven                       |
| Diagnostics generation                         |                       not present |                                                         75 ms | interactive only                           |
| Tray-only process tree                         |                          one host |                                one host, zero direct children | PASS                                       |
| Repeated restart                               |              prior lifecycle pass |                               5/5 clean forced-restart cycles | PASS                                       |

Normal-screen decoder median/p95 was 2.0870/2.1848 ms; the three-code fixture
was 4.8324/5.4855 ms. These are not substituted for physical end-to-end scan
latency. No sustained CPU, polling, telemetry, network call, or permanent
background work was observed or added. Repeated physical-scan memory growth was
not rerun and therefore has no Phase 1.2 claim.

## Evidence classification and remaining manual work

### Physically exercised on this host

- Installer/upgrade/reinstall/uninstall lifecycle described above
- One-display enumeration and Settings UI/version/commit/diagnostics action
- Single-instance behavior, forced termination, repeated restart, process tree,
  and Settings window creation/destruction
- Clipboard diagnostics privacy scan and opt-in log growth

### Inspected, inferred, or automated only

- Tray Quit calls the Tauri process-exit path and diagnostics own no worker;
  deterministic tray-menu exit remains inspected, not physically clicked.
- Atomic settings, stale scan/chooser state, adapter failures, inaccessible log
  storage, and webview creation cleanup have focused automated coverage.
- Windows shutdown/logout are expected to receive normal process teardown, but
  were not physically exercised.

### Not run

- All 51 Phase 1.1 manual RC cases remain `Not run`; this report does not change
  that matrix.
- Physical multi-monitor, mixed-DPI, portrait, Narrator, NVDA, High Contrast,
  Windows sign-out/sign-in/shutdown, injected webview crash, and inaccessible
  settings-directory cases.
- Physical single-result, multi-result chooser, repeated-scan memory, and tray
  Quit measurements in this pass. The image-viewer launch approval timed out,
  and the notification area was not exposed as a targetable automation window.
- Real signing, SmartScreen reputation, tag push, GitHub Release publication,
  and GitHub-hosted RC workflow execution.

## Files changed

- Delivery: `.cargo/audit.toml`, `deny.toml`, both Windows workflows, `.gitignore`,
  root manifests/locks, desktop npm/Cargo/Tauri manifests, and `.npmrc`.
- Host/frontend: build identity, diagnostics/log lifecycle, clipboard command,
  runtime error categories/state, Settings UI/API, and focused tests.
- Recovery: storage atomic-write recovery tests and hotkey rollback cleanup.
- Scripts: version consistency, dependency policy, manifest generation,
  release-tooling tests, and full release validation under `scripts/release`.
- Documentation: README, development/threat model, third-party notices, release
  procedure, installer behavior, diagnostics, dependency security, signing
  readiness, and this report.

Ignored `target`, `node_modules`, npm caches, build output, installers, logs,
diagnostics, machine settings, and runtime evidence are not committed.

## Signing readiness and publication state

Executable-then-installer signing order, certificate/EKU requirements,
timestamping, verification, CI secret boundaries, renewal/revocation, and
SmartScreen limitations are documented. No credential interface is enabled,
and no certificate, key, password, token, or PFX is present. Current artifacts
are explicitly unsigned.

No merge, tag, push, signing operation, installer publication, GitHub workflow
dispatch, or GitHub Release has been performed.

## Recommendation

Before treating 0.1.2 as a public signed release, run the remaining human and
hardware RC matrix, especially tray Quit, physical single/multi scan latency,
repeated-scan memory, accessibility, mixed-DPI/multi-monitor, and sign-out/
shutdown behavior. After review, push this branch and dispatch the unsigned RC
workflow; signing and publication remain separate authorized release actions.
