use crate::runtime::RuntimeState;
use std::time::Instant;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, Wry};

/// Creates or focuses the lazily owned settings webview.
pub fn open(app: &tauri::AppHandle<Wry>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    let started = Instant::now();
    let window = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("QRForge Settings")
        .inner_size(560.0, 720.0)
        .min_inner_size(480.0, 620.0)
        .resizable(true)
        .maximizable(false)
        .visible(true)
        .center()
        .build()?;
    let diagnostics = app.state::<RuntimeState>().diagnostics.clone();
    diagnostics.record_window("settings_window_created", Some(started.elapsed()));
    window.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Destroyed) {
            diagnostics.record_window("settings_window_destroyed", None);
        }
    });
    Ok(())
}
