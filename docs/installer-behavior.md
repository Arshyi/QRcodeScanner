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

The stable product name and identifier are prerequisites for same-product
replacement. Configuration inspection alone does **not** prove same-version
reinstallation, live-process upgrade handling, settings retention, shortcut
cleanup, or uninstall behavior.

## Validation matrix

| Case | Evidence required | Status before physical execution |
|---|---|---|
| Clean install | install UI, file/registry/shortcut inventory, launch | Not run |
| Same-version install | second install over exact candidate | Not run |
| 0.1.1 to 0.1.2 upgrade | verified Phase 1.1 installer then candidate | Not run |
| Settings retention | distinctive settings before/after upgrade | Not run |
| Startup retention | registry/task state before/after upgrade | Not run |
| Running-process upgrade | observed installer prompt/handling | Not run |
| Tray Quit | zero QRForge/WebView2 descendants after Quit | Not run |
| Uninstall after use | installed files/shortcut/registration removed | Not run |
| User data after uninstall | settings retained or explicitly removed | Not run |
| Sensitive-file search | no image/frame/payload data in install/data dirs | Not run |

The exact Phase 1.1 input must be named
`QRForge_0.1.1_x64-setup.exe`, have size 2,233,256 bytes, and match SHA-256
`619E0647DB1A63BAE0C2370A090B0561FE519D92EEC2CA3D508F5B2183F3037D`.
Do not run an artifact that fails any identity check.

## Safety and data ownership

Before installer testing, back up `settings.json` and record startup state.
Tests must not treat uninstalling program files as authorization to erase user
data. If the uninstaller offers data deletion in a future version, it must be
explicit, unchecked by default, and covered by a separate destructive-action
review. Exact observed install paths, registry keys, file layout, upgrade
behavior, and uninstall retention belong in `docs/phase-1-2-results.md`.
