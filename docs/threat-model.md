# QRForge threat model

## Security objectives

1. Screen and camera pixels remain local and ephemeral.
2. QR payloads are untrusted data and never executable instructions.
3. A compromised webview cannot acquire broad host authority.
4. Automatic browser actions are bounded, visible, and cancelable where possible.
5. Dependencies and updates cannot silently bypass review controls.

## Assets and trust boundaries

Assets include screen/camera imagery, decoded payloads, history, clipboard/browser authority, settings, and release-signing material.

```text
Untrusted pixels and payloads
          |
native capture -> native decoder -> Rust safety policy
                                        |
                                typed narrow IPC
                                        |
                                 untrusted webview
                                        |
                             explicit OS adapters
```

## Threats and required controls

| Threat | Required control |
|---|---|
| Dangerous URL schemes or command-like text | Parse in Rust; auto-open only HTTP(S); never invoke a shell |
| Homograph or credential-bearing URL | Show normalized host; warn on IDN/punycode and embedded credentials |
| Repeated detections during scrolling | Cooldown, stable-detection rule, bounded queue, tab cap, emergency stop |
| Multiple URLs in one frame | Result chooser by default; explicit capped automation only |
| Webview compromise | Strict CSP, no remote content, per-window capabilities, typed validated commands |
| Retained screen/camera data | Reusable memory-only buffers, redacted logs, no screenshot history |
| Camera remains active | Explicit visible session, cancellation on close/suspend, OS permission flow |
| Malformed image or decoder bug | Bounds/stride validation, isolated FFI, fuzzing and corpus tests |
| Sensitive local history | Disable/retention options, minimal fields, least-privilege storage |
| Export path overwrite | User-selected canonical path and overwrite confirmation |
| Supply-chain compromise | Lockfiles, pinned CI actions, audits, signed releases and update manifests |

## Privacy invariants

- No telemetry or network client capability by default.
- No captured image is written to disk.
- Full payloads are excluded from logs and crash metadata.
- History is disclosed, configurable, and clearable.
- Browser launch occurs only after Rust-side parsing and policy evaluation.

## Residual risks

- A valid HTTPS destination still receives normal browser/network metadata.
- Malware running as the user may independently observe the screen or read local files.
- OS capture/camera APIs and the system webview remain trusted dependencies.

