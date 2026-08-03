# Phase 1.2 Windows release-hardening results

Status: implementation and local validation in progress; not published.

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

## Implemented hardening

- Version 0.1.2 with root Cargo workspace version as documented source and a
  fail-closed cross-manifest/metadata check.
- Compile-time Git commit identity in Settings and Copy Diagnostics.
- Explicit privacy-safe Copy Diagnostics action implemented in Rust.
- Stable recent error categories with bounded in-process retention.
- Opt-in JSONL logs changed from startup truncation/unbounded append to 256 KiB
  active log plus one archive; no polling or logging worker was added.
- Settings recovery regression coverage for stale partial files and failed
  atomic replacement, in addition to malformed-source preservation.
- PowerShell version, dependency, manifest/checksum, and complete release
  validation scripts. Scripts publish, push, sign, install, and modify global
  configuration nowhere.
- Separate least-privilege manual unsigned-RC workflow with full-SHA action
  pins, exact ref/commit/version inputs, 30-day evidence artifact, and no write
  permission.
- Dependency, native decoder provenance, third-party notices, installer,
  release, diagnostics, and signing policies.

## Evidence classification

### Automated local validation

Results will be recorded after the final committed release-validation run.

### CI validation

Not run for this unpushed branch. The workflow files were inspected locally;
GitHub execution remains Not run until an explicitly authorized push.

### Manual/runtime validation

Results will be recorded only for tests actually exercised on this Windows
host. Physical multi-monitor, mixed-DPI, screen-reader, High Contrast, and
sign-out/sign-in cases are outside the available one-display boundary.

### Inspected behavior

- NSIS is configured for current-user installation with stable product name and
  identifier.
- Settings writes use a same-directory temporary file, flush/sync, and atomic
  persist.
- Runtime scan admission uses an RAII permit, so worker completion or panic
  releases the in-progress gate.
- Pending result sessions exist only in process memory and a failed result
  webview clears only its matching session.
- Tray Quit calls the Tauri process exit path; diagnostics own no worker or
  timer that could keep the process alive.

### Inferred behavior

- Stable identifier/product/version should route a newer NSIS package through
  same-product upgrade handling. This remains inference until the exact 0.1.1
  to 0.1.2 path is executed.
- Per-user settings live outside the installation directory and therefore
  should survive executable replacement. This remains inference until verified
  before and after install/uninstall.

### Untested behavior

- The 51 Phase 1.1 manual RC cases remain `Not run`; this report does not change
  their status.
- Real signing, SmartScreen reputation, tag push, and GitHub Release publication
  are intentionally not performed.
- Physical multi-monitor, mixed-DPI, Narrator, NVDA, High Contrast, and Windows
  sign-out/sign-in tests remain `Not run`.

## Release and installer evidence

To be completed with the final artifact path, byte size, SHA-256, manifest,
installer inspection, upgrade, same-version, running-process, and uninstall
results. The preserved Phase 1.1 comparison artifact must first match:

- size: 2,233,256 bytes
- SHA-256:
  `619E0647DB1A63BAE0C2370A090B0561FE519D92EEC2CA3D508F5B2183F3037D`

## Security and dependency evidence

The initial `cargo audit` run correctly blocked seven findings: two advisories
across three `quick-xml` versions plus one `time` advisory. The supported
Windows plist path was narrowly updated to plist 1.10.0, quick-xml 0.41.0, and
time 0.3.55; this raised the honest minimum Rust version to 1.88. Two older
quick-xml versions remain only under xcap's non-Windows xcb/Wayland paths and
are explicitly, time-boundly documented in `.cargo/audit.toml` because
`cargo audit` cannot target-filter. Final audit, deny, npm, license, pin, and
secret/path scan results remain to be recorded.
The initial npm audit then blocked GHSA-mh99-v99m-4gvg in brace-expansion
1.1.16 and 5.0.7. Narrow parent-scoped npm overrides select patched 1.1.18 and
5.0.9 without a broad dependency refresh.

## Recovery evidence

To be completed with unit tests plus any real forced-termination, restart,
window failure, inaccessible-directory, process-tree, and Quit observations.

## Performance evidence

Phase 1.1 comparison values are preserved in `docs/phase-1-1-results.md`.
Phase 1.2 measurements will record idle CPU/memory, settings lifecycle, scan and
chooser latency, startup/shutdown, logging growth, diagnostics generation,
process counts, and restart behavior. No unperformed measurement will be
reported as passing.

## Publication state

No merge, tag, push, signing operation, installer publication, or GitHub
Release has been performed.
