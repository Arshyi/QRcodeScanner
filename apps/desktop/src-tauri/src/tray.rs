use crate::{runtime::RuntimeState, window};
use tauri::{
    AppHandle, Manager, Wry,
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

    let state = app.state::<RuntimeState>();
    let hotkey_text = idle_tooltip(state.settings.snapshot().active_hotkey.as_deref());

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip(&hotkey_text)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            SCAN_ID => {
                let state = app.state::<RuntimeState>();
                state.scans.spawn("tray");
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

/// Restores the non-sensitive idle tooltip using the actual registered hotkey.
pub fn refresh_idle_tooltip(app: &AppHandle<Wry>) -> tauri::Result<()> {
    let state = app.state::<RuntimeState>();
    let tooltip = idle_tooltip(state.settings.snapshot().active_hotkey.as_deref());
    app.tray_by_id("main-tray").map_or_else(
        || Err(tauri::Error::AssetNotFound("main-tray".to_owned())),
        |tray| tray.set_tooltip(Some(tooltip)),
    )
}

fn idle_tooltip(active_hotkey: Option<&str>) -> String {
    active_hotkey.map_or_else(
        || "QRForge — configure hotkey in Settings".to_owned(),
        |hotkey| format!("QRForge — press {hotkey} to scan"),
    )
}

#[cfg(test)]
mod tests {
    use super::idle_tooltip;

    #[test]
    fn idle_tooltip_never_claims_a_stale_default_hotkey() {
        assert_eq!(
            idle_tooltip(Some("Ctrl+Alt+M")),
            "QRForge — press Ctrl+Alt+M to scan"
        );
        assert_eq!(idle_tooltip(None), "QRForge — configure hotkey in Settings");
    }
}
