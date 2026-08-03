# Windows release procedure

This procedure produces an **unsigned release candidate**. It never grants a
script permission to push, tag, publish, install, or sign anything.

## Version and identity policy

`Cargo.toml` `[workspace.package].version` is the canonical QRForge version.
For Tauri and npm tooling, the same value is deliberately repeated in:

- `apps/desktop/package.json`
- the root and root-package entries in `apps/desktop/package-lock.json`
- `apps/desktop/src-tauri/tauri.conf.json`

Every Rust workspace package inherits the canonical value. Run
`scripts/release/Test-VersionConsistency.ps1`; CI runs the same check and fails
on any drift. Release builds embed `QRFORGE_BUILD_COMMIT`, or a validated local
`git rev-parse HEAD` fallback, for Settings and Copy Diagnostics.

Phase 1.2 uses version 0.1.2. Changing this internal version does not create a
tag or published release.

## Prerequisites

- Windows x64 on the current GitHub-hosted Windows image or an equivalent host
- Git and GitHub CLI for human release preparation
- Rust 1.97.1 with `rustfmt` and `clippy`
- Node.js 24.18.0 and npm
- CMake plus a supported Visual Studio Build Tools MSVC installation
- NSIS prerequisites used by Tauri
- `cargo-audit` 0.22.2 and `cargo-deny` 0.20.2

The repository-owned CMake toolchain selects the release MSVC runtime for the
bundled native decoder. The validation script checks tools but does not install
software, modify PowerShell policy, or change global Git configuration.

## Candidate build

1. Begin from a clean checkout of the intended branch, tag, or commit. Fetch
   remote state and verify `git status --short`, `git branch -vv`, and
   `git rev-parse HEAD`.
2. Install Rust dependencies through locked Cargo commands and the frontend
   through `npm ci`.
3. Install the two pinned audit tools explicitly if missing:

   ```powershell
   cargo install --locked cargo-audit --version 0.22.2
   cargo install --locked cargo-deny --version 0.20.2
   ```

4. Ensure the NSIS bundle directory contains no installer from another
   version. Preserve a required comparison artifact outside that directory.
5. Run the fail-closed validation from the repository root:

   ```powershell
   .\scripts\release\Validate-Release.ps1 `
     -ExpectedVersion 0.1.2 `
     -ExpectedRef feat/phase-1-2-release-hardening `
     -ExpectedCommit (git rev-parse HEAD)
   ```

The script verifies a clean worktree and intended ref/commit, version
consistency, Rust and frontend formatting/lint/tests/builds, spike checks,
dependency policy, fixture benchmark, Release builds, Tauri, and NSIS. It
requires exactly one `QRForge_<version>_x64-setup.exe`, then writes
`SHA256SUMS.txt` and `release-manifest.json` beside it. Generated files remain
under ignored `target` paths.

## Manual smoke and upgrade validation

Use a disposable local Windows user or VM when possible. Verify the installer
hash before execution. Record evidence—do not convert inspection or automation
into a manual PASS.

1. Clean-install the candidate and launch it.
2. Quit from the tray and confirm the complete process tree exits.
3. Install the same version over itself, then recheck executable identity,
   settings, startup preference, shortcuts, and process behavior.
4. Install the exact Phase 1.1 artifact whose SHA-256 is
   `619E0647DB1A63BAE0C2370A090B0561FE519D92EEC2CA3D508F5B2183F3037D`,
   create distinctive settings, quit, upgrade to 0.1.2, and verify those
   settings and startup state.
5. Exercise a controlled upgrade attempt while QRForge is running and record
   the NSIS handling.
6. Search install and user-data locations for screenshots, frames, QR
   payloads, and unexpected logs.
7. Uninstall after use. Confirm installed program files and shortcuts are
   removed, no process remains, and record whether per-user settings are
   retained.
8. Run the remaining physical cases in
   `docs/phase-1-1-rc-validation.md` only with the required hardware and human
   interaction.

## Tag and GitHub Release preparation

Only after reviewed evidence and explicit authorization, prepare an annotated
`v<version>` tag at the manifest commit. Verify the tag target, installer hash,
manifest commit, and unsigned/signed status before any push. Draft release
notes should include the installer, checksum, manifest, third-party notices,
test boundary, and known unsigned SmartScreen limitations. This repository's
automation does not create or push the tag and does not publish a GitHub
Release.

## Rollback

If any gate fails, retain its logs privately, discard the candidate artifact,
and fix forward on the feature branch. Do not move an already published tag.
If a bad release was published, mark it clearly, remove only the affected
download after authorization, publish a corrected patch version, and document
settings compatibility. The Phase 1.1 installer is the comparison baseline,
not an automatic downgrade mechanism.
