mod lifecycle;

use std::sync::{Arc, atomic::AtomicU64};
use tauri::tray::TrayIconBuilder;

fn main() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let tray_icon = app
                .default_window_icon()
                .cloned()
                .ok_or("benchmark icon was not loaded")?;
            TrayIconBuilder::with_id("main-tray")
                .icon(tray_icon)
                .tooltip("QRForge lifecycle spike")
                .build(app)?;

            if std::env::args().any(|argument| argument == "--with-window") {
                tauri::WebviewWindowBuilder::new(
                    app,
                    "main",
                    tauri::WebviewUrl::App("index.html".into()),
                )
                .title("QRForge Idle Spike")
                .visible(false)
                .build()?;
            }
            if std::env::args().any(|argument| argument == "--lifecycle") {
                let heartbeat = Arc::new(AtomicU64::new(0));
                lifecycle::start_heartbeat(Arc::clone(&heartbeat));
                lifecycle::start(app.handle().clone(), heartbeat);
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Tauri idle spike failed to build");
    app.run(|_app, event| {
        if let tauri::RunEvent::ExitRequested {
            code: None, api, ..
        } = event
        {
            api.prevent_exit();
        }
    });
}
