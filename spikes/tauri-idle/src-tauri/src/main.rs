fn main() {
    tauri::Builder::default()
        .setup(|app| {
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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Tauri idle spike failed");
}
