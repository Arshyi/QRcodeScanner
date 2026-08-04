# Dependency security and licensing

QRForge uses locked Rust and npm graphs. Release validation does not perform
bulk dependency updates.

## Required checks

From a clean repository after `npm ci`:

```powershell
cargo audit
cargo deny check advisories licenses sources
npm audit --audit-level=high
.\scripts\release\Test-DependencyPolicy.ps1
```

`cargo audit` blocks known Rust vulnerabilities. `cargo deny` blocks advisories
and yanked packages, non-approved licenses, unknown registries, and unknown Git
sources. Unmaintained and unsound advisories fail when they affect a workspace
crate and remain visible warnings for transitive crates that upstream projects
must replace; the release maintainer reviews those warnings for reachability.
Multiple versions and wildcard declarations are permitted because the Tauri
graph commonly requires parallel compatible versions. npm audit blocks high
and critical findings; lower findings require recorded reachability and
exposure review. Every npm lockfile package must carry license metadata.

Never silently waive a reachable high-severity issue. A temporary exception
must name the advisory, affected component and path, reachability conclusion,
mitigation, owner, expiry date, and removal issue. Exceptions belong in the
reviewed policy file, not command-line flags.

### Current target-scoped audit exception

On 2026-08-03, RustSec advisories RUSTSEC-2026-0194 and
RUSTSEC-2026-0195 identified denial-of-service flaws in `quick-xml` below
0.41.0. The supported Windows graph was updated from `plist` 1.8.0 to 1.10.0,
which selects `quick-xml` 0.41.0 and `time` 0.3.55. The only older
`quick-xml` packages left in `Cargo.lock` are build dependencies under xcap's
xcb/Wayland paths; `cargo tree --target x86_64-pc-windows-msvc` proves they are
absent from the QRForge Windows build.

Because `cargo audit` cannot target-filter a lockfile, `.cargo/audit.toml`
records those two advisory IDs as explicit exceptions. `cargo deny` evaluates
the configured Windows target and has no corresponding vulnerable path. Owner:
QRForge release maintainer. Review on every candidate and no later than
2026-11-01; remove the exceptions when xcap's non-Windows graph updates. This
does not waive any vulnerable package reachable in the Windows product.

## Native decoder provenance

`crates/qrforge-decoder/Cargo.toml` pins `zxing-cpp` exactly to 0.5.2 with
default features disabled and the bundled native build enabled. `Cargo.lock`
pins crates.io checksum
`d412e2db33c4afe7aac2e90c829938e8dac4dba2e9572d856b3d8eefc702eae9`.
The archive reports ZXing-C++ 3.1.0 and includes libzint 2.16.0/libzueci
sources. Attribution is in `THIRD_PARTY_NOTICES.md`.

For an update, inspect the crate diff and bundled native source, verify the new
registry checksum and license files, rebuild with the repository CMake
toolchain, run all decoder fixtures and benchmarks, rerun the two supply-chain
tools, and update notices. Do not accept an unreviewed decoder update merely to
silence automation.

## Vulnerability response

1. Reproduce the finding against the locked graph.
2. Determine whether QRForge reaches the vulnerable feature on Windows.
3. For a reachable high/critical issue, stop release and prepare the smallest
   compatible update or mitigation.
4. Rerun all gates, installer inspection, and applicable manual smoke tests.
5. Document affected versions, remediation, and whether credential or payload
   exposure was possible.
6. Revoke signing material immediately if it may have been exposed; signing
   guidance is in `docs/signing-readiness.md`.

GitHub Actions are pinned to full commit SHAs. Updating a pin requires checking
the upstream action repository, intended release tag, changelog, permissions,
and resolved commit before review.

The 2026-08-03 npm audit also found GHSA-mh99-v99m-4gvg in two development
tool paths. Root npm overrides constrain only minimatch 3.1.5 to
brace-expansion 1.1.18 and minimatch 10.2.5 to brace-expansion 5.0.9. Both
patched versions remain inside the parents' declared semver ranges; the rest of
the dependency graph is not bulk-updated.

A 2026-08-04 refresh found the moderate PostCSS advisory
GHSA-fxqj-rqcc-2cmp in the development build/lint graph. The lockfile was
narrowly updated from PostCSS 8.5.18 to patched 8.5.25 without adding a direct
dependency or changing application packages. The same refresh found
RUSTSEC-2026-0221 in `event-listener` 5.4.1. `cargo tree --target
x86_64-pc-windows-msvc` proves that crate is absent from the Windows build; its
all-target path is through the non-Windows zbus/notification stack. The warning
is not suppressed. Review it on each candidate and no later than 2026-11-01,
then remove the old lockfile node when upstream target dependencies permit.

npm install scripts are fail-closed through `strict-allow-scripts=true`.
`esbuild@0.28.1` is the sole approved script: its package install hook selects
and verifies the locked platform binary required by Vite. The approval is
version-pinned; any version change or new install-script package blocks
`npm ci` pending review. `fsevents@2.3.3` is explicitly denied because it is a
macOS-only optional watcher and has no role in the Windows product.
