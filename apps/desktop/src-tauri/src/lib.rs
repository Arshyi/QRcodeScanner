//! QRForge Tauri lifecycle and composition root.

mod commands;
mod diagnostics;
mod notification;
mod runtime;
mod startup;
mod tray;
mod window;

use crate::{
    diagnostics::Diagnostics,
    notification::TauriNotifications,
    runtime::{RuntimeState, ScanDispatcher},
    startup::TauriStartup,
};
use qrforge_application::{
    CapturePort, HotkeyPort, Notification, NotificationPort, ResultService, ScanPorts, ScanService,
    SettingsRepository, SettingsService, SettingsState,
};
use qrforge_capture::XcapCapture;
use qrforge_decoder::ZxingDecoder;
use qrforge_domain::AppSettings;
use qrforge_platform::{SystemBrowser, SystemClipboard, SystemClock, TauriHotkey};
use qrforge_storage::FileSettingsRepository;
use std::{sync::Arc, time::Instant};
use tauri::{Manager, RunEvent};

/// Starts the tray-first desktop host.
///
/// # Panics
///
/// Panics if the Tauri runtime cannot be initialized (for example, if the
/// bundled resource context is missing or the system fails to register the
/// tray icon). Process startup and the initial setup closure may also return
/// errors that the host has no sensible recovery for, so they propagate as
/// panics on the main thread.
///
/// Tauri's single-instance plugin owns the platform lock. A second launch
/// asks the running process to open or focus Settings, then exits.
pub fn run() {
    let process_started = Instant::now();
    let app = tauri::Builder::default()
        // This must remain the first plugin so it can stop duplicate hosts
        // before any other plugin performs process-wide initialization.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = window::open(app);
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::update_settings,
            commands::complete_onboarding,
            commands::copy_diagnostics,
            commands::get_pending_results,
            commands::perform_result_action
        ])
        .setup(move |app| {
            let data_dir = app.path().app_data_dir()?;
            let diagnostics = Arc::new(Diagnostics::new(&data_dir, process_started));
            let repository: Arc<dyn SettingsRepository> =
                Arc::new(FileSettingsRepository::new(data_dir.join("settings.json")));
            let initial_settings = repository.load().unwrap_or_else(|_| {
                diagnostics.record_error("settings_load_failed");
                AppSettings::default()
            });
            let settings_state = Arc::new(SettingsState::new(initial_settings.clone()));
            let notifications: Arc<dyn NotificationPort> =
                Arc::new(TauriNotifications::new(app.handle().clone()));
            let capture: Arc<dyn CapturePort> = Arc::new(XcapCapture);
            let browser = Arc::new(SystemBrowser);
            let clipboard = Arc::new(SystemClipboard);
            let results = Arc::new(ResultService::new(browser.clone(), clipboard.clone()));
            let scan = Arc::new(ScanService::new(
                ScanPorts {
                    capture: capture.clone(),
                    decoder: Arc::new(ZxingDecoder),
                    browser,
                    clipboard: clipboard.clone(),
                    notifications: notifications.clone(),
                    clock: Arc::new(SystemClock::new()),
                },
                settings_state.clone(),
            ));
            let scans = Arc::new(ScanDispatcher::new(
                scan,
                diagnostics.clone(),
                results.clone(),
                app.handle().clone(),
            ));
            let hotkeys = Arc::new(TauriHotkey::new(app.handle().clone(), {
                let scans = scans.clone();
                Arc::new(move || scans.spawn("hotkey"))
            }));
            let startup = Arc::new(TauriStartup::new(app.handle().clone()));
            let settings = Arc::new(SettingsService::new(
                repository,
                hotkeys.clone(),
                startup,
                settings_state,
            ));
            let hotkey_conflict = hotkeys.replace(&initial_settings.hotkey).is_err();
            app.manage(RuntimeState {
                scans,
                settings,
                capture,
                results,
                notifications: notifications.clone(),
                clipboard,
                diagnostics: diagnostics.clone(),
            });
            tray::create(app)?;
            if hotkey_conflict {
                diagnostics.record_error("hotkey_registration_failed");
                let _ = notifications.notify(Notification::HotkeyConflict);
            }
            if hotkey_conflict || !initial_settings.onboarding_completed {
                let _ = window::open(app.handle());
            }
            diagnostics.record_startup(process_started.elapsed());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("QRForge failed to initialize");

    app.run(|_app, event| {
        if let RunEvent::ExitRequested {
            code: None, api, ..
        } = event
        {
            api.prevent_exit();
        }
    });
}
