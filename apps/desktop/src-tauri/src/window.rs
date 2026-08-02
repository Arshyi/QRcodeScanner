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
        .inner_size(580.0, 760.0)
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

/// Creates or focuses the lazy multi-code chooser webview.
pub fn open_results(app: &tauri::AppHandle<Wry>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("results") {
        window.eval("window.location.reload()")?;
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    let started = Instant::now();
    let window = WebviewWindowBuilder::new(
        app,
        "results",
        WebviewUrl::App("index.html?surface=results".into()),
    )
    .title("QRForge Scan Results")
    .inner_size(660.0, 680.0)
    .min_inner_size(460.0, 420.0)
    .resizable(true)
    .maximizable(false)
    .visible(true)
    .center()
    .build()?;
    let state = app.state::<RuntimeState>();
    state
        .diagnostics
        .record_window("results_window_created", Some(started.elapsed()));
    let diagnostics = state.diagnostics.clone();
    let results = state.results.clone();
    window.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Destroyed) {
            results.clear(None);
            diagnostics.record_window("results_window_destroyed", None);
        }
    });
    Ok(())
}

/// Closes the chooser if it is currently alive.
pub fn close_results(app: &tauri::AppHandle<Wry>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("results") {
        window.close()?;
    }
    if let Some(settings) = app.get_webview_window("settings") {
        settings.set_focus()?;
    }
    Ok(())
}
