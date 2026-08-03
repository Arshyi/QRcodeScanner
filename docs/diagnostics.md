# Diagnostics and logging

QRForge diagnostics are local, inert text and are disabled by default.

## Copy Diagnostics

Settings provides an explicit **Copy diagnostics** action. Rust constructs the
text and writes it directly to the Windows clipboard; the webview receives only
a success or sanitized failure message. Fields appear in a fixed order and
include the application version, build commit, Windows release/build,
architecture, settings schema, boolean hotkey/startup state, safe monitor
selection state and count, decoder version, stable recent error categories,
PID, uptime, and a logical log location.

The snapshot intentionally excludes configured hotkey text and raw monitor
identifiers/labels. It also excludes screenshots, frames, decoded payloads,
clipboard history, window/application titles, browser history, usernames,
absolute paths, environment variables, tokens, and command lines. Registry
text used for the Windows version is ASCII-whitelisted and bounded before it is
included. Diagnostic text is never executed or interpreted as a URL.

## Optional JSONL logging

Set `QRFORGE_DIAGNOSTICS=1` before launch to enable the local log. QRForge does
not copy the environment or its value into diagnostics. The logical locations
are:

- `QRForge app data\diagnostics.jsonl`
- `QRForge app data\diagnostics.jsonl.1`

The active log is capped at 256 KiB. Before a record would exceed the cap, the
active file replaces the single archive; at most two files are retained. An
existing log is appended rather than truncated at startup. Events contain only
lifecycle names, fixed trigger/error categories, durations, result categories,
detection counts, dimensions, and scaling. Monitor labels and QR content are
not logged. If the directory or file cannot be opened, logging disables itself
without blocking startup.

Logging creates no timer, polling loop, network request, worker, or keep-alive
handle. Writes occur only on startup, scan completion, and window lifecycle
events, so tray Quit is not delayed by background logging work.

## Support handling

Ask the user to inspect copied text before sharing it. Treat even sanitized
diagnostics as support data, retain only as long as necessary, and never ask
for QR payloads or screen captures unless the user separately and knowingly
chooses to provide them.
