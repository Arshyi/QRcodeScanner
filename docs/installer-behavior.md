# Windows installer and upgrade behavior

## Configured behavior

Inspection of `apps/desktop/src-tauri/tauri.conf.json` establishes:

- Tauri product name: `QRForge`
- stable application identifier: `app.qrforge.desktop`
- target: NSIS x64 installer
- install mode: current user (no machine-wide install requested)
- expected artifact: `QRForge_<version>_x64-setup.exe`
- current signing state: unsigned

Tauri/NSIS owns installed executable, uninstaller, Start Menu entry, and
uninstall registration. The application owns `settings.json` under the Tauri
per-user app-data directory. Launch-at-sign-in is owned by the Tauri autostart
plugin and reflects the saved setting after successful transactional updates.
Screenshots and decoded payloads have no storage path. Optional bounded logs
are described in `docs/diagnostics.md`.

The Phase 1.2 installed-runtime exercise confirms the configured layout:

- `%LOCALAPPDATA%\QRForge\qrforge.exe`
- `%LOCALAPPDATA%\QRForge\uninstall.exe`
- `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\QRForge`
- `%APPDATA%\Microsoft\Windows\Start Menu\Programs\QRForge.lnk`
- user data: `%APPDATA%\app.qrforge.desktop`
- optional log: `%APPDATA%\app.qrforge.desktop\diagnostics.jsonl`

The shortcut targets the installed executable and the uninstall string targets
the adjacent uninstaller. Only `qrforge.exe` and `uninstall.exe` were present
in the install directory. Both tested binaries reported Authenticode
`NotSigned`; the installed executable reported product version 0.1.2.

## Validation matrix

| Case                      | Phase 1.2 evidence                                                 | Status  |
| ------------------------- | ------------------------------------------------------------------ | ------- |
| Baseline install          | Verified 0.1.1 artifact installed with expected files/key/shortcut | PASS    |
| Clean candidate install   | 0.1.2 exit 0; expected layout/version; no auto-launch              | PASS    |
| Same-version install      | 0.1.2 over running 0.1.2, exit 0; settings retained                | PASS    |
| 0.1.1 to 0.1.2 upgrade    | Exact hash-verified baseline upgraded to candidate                 | PASS    |
| Settings retention        | Distinctive settings file hash unchanged                           | PASS    |
| Startup retention         | Absent Run entry remained absent with saved false state            | PASS    |
| Running-process handling  | Upgrade/reinstall/uninstall each reduced host count 1 to 0         | PASS    |
| Application launch        | One tray host; second launch opened Settings and exited            | PASS    |
| Tray Quit                 | Notification area not targetable by automation helper              | Not run |
| Uninstall after use       | Program files/key/shortcut removed; zero process                   | PASS    |
| User data after uninstall | Settings and optional log retained                                 | PASS    |
| Sensitive-file search     | No screenshot/frame/payload image in install or roaming data       | PASS    |

The exact Phase 1.1 input must be named
`QRForge_0.1.1_x64-setup.exe`, have size 2,233,256 bytes, and match SHA-256
`619E0647DB1A63BAE0C2370A090B0561FE519D92EEC2CA3D508F5B2183F3037D`.
Do not run an artifact that fails any identity check.

The tested 0.1.2 candidate is 2,238,807 bytes with SHA-256
`C4FCFD8D1A0AB8A630A822F6B60A089536F3FE20494E1A7FFF1CDAE9603C9FD2`.
Exact timings, settings hashes, process behavior, and final machine restoration
are recorded in `docs/phase-1-2-results.md`.

## Safety and data ownership

Before installer testing, back up `settings.json` and record startup state.
Tests must not treat uninstalling program files as authorization to erase user
data. If the uninstaller offers data deletion in a future version, it must be
explicit, unchecked by default, and covered by a separate destructive-action
review. Exact observed install paths, registry keys, file layout, upgrade
behavior, and uninstall retention belong in `docs/phase-1-2-results.md`.
Phase 1.2 testing followed this procedure: pre-existing roaming data and startup
state were backed up, test-generated data was archived under ignored `target`,
and the original two files were restored with exact hash matches. The machine
was left with QRForge uninstalled, the startup entry absent, and no QRForge
process running.
