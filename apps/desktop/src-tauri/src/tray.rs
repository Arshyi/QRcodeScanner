use crate::{runtime, runtime::RuntimeState, window};
use tauri::{
    Manager,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

const SCAN_ID: &str = "scan-now";
const SETTINGS_ID: &str = "open-settings";
const QUIT_ID: &str = "quit";

/// Creates the sole persistent user-interface surface: the native tray icon.
///
/// The tooltip displays the currently registered hotkey or a message if registration failed.
pub fn create(app: &tauri::App) -> tauri::Result<()> {
    let scan = MenuItem::with_id(app, SCAN_ID, "Scan Now", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, SETTINGS_ID, "Open Settings", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&scan, &settings, &separator, &quit])?;
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::AssetNotFound("default window icon".to_owned()))?;

    // Construct tooltip showing the actual registered hotkey (or fallback message).
    let state = app.state::<RuntimeState>();
    let hotkey_text = state
        .settings
        .snapshot()
        .active_hotkey
        .map_or_else(
            || "QRForge — configure hotkey in Settings".to_string(),
            |hk| format!("Press {hk} to scan"),
        );

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip(&hotkey_text)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            SCAN_ID => {
                let state = app.state::<RuntimeState>();
                runtime::spawn_scan(state.scan.clone(), state.diagnostics.clone(), "tray");
            }
            SETTINGS_ID => {
                let _ = window::open(app);
            }
            QUIT_ID => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}
