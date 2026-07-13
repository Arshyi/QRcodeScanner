use qrforge_application::{HotkeyPort, PortError};
use qrforge_domain::{Hotkey, HotkeyKey};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Wry};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// Tauri global-shortcut adapter with transactional replacement.
pub struct TauriHotkey {
    app: AppHandle<Wry>,
    callback: Arc<dyn Fn() + Send + Sync>,
    active: Mutex<Option<Hotkey>>,
}

impl TauriHotkey {
    /// Creates an unregistered hotkey adapter.
    #[must_use]
    pub fn new(app: AppHandle<Wry>, callback: Arc<dyn Fn() + Send + Sync>) -> Self {
        Self {
            app,
            callback,
            active: Mutex::new(None),
        }
    }

    fn register(&self, hotkey: &Hotkey) -> Result<(), PortError> {
        let shortcut = shortcut(hotkey);
        let callback = self.callback.clone();
        self.app
            .global_shortcut()
            .on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    callback();
                }
            })
            .map_err(|error| PortError::new("hotkey_register", error.to_string()))
    }

    fn unregister(&self, hotkey: &Hotkey) -> Result<(), PortError> {
        self.app
            .global_shortcut()
            .unregister(shortcut(hotkey))
            .map_err(|error| PortError::new("hotkey_unregister", error.to_string()))
    }
}

impl HotkeyPort for TauriHotkey {
    fn active(&self) -> Option<Hotkey> {
        self.active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn replace(&self, requested: &Hotkey) -> Result<(), PortError> {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if active.as_ref() == Some(requested) {
            return Ok(());
        }
        let previous = active.clone();
        if let Some(previous) = previous.as_ref() {
            self.unregister(previous)?;
        }
        if let Err(registration_error) = self.register(requested) {
            if let Some(previous) = previous.as_ref() {
                if let Err(rollback_error) = self.register(previous) {
                    *active = None;
                    return Err(PortError::new(
                        "hotkey_rollback",
                        format!(
                            "replacement failed ({registration_error}); previous registration could not be restored ({rollback_error})"
                        ),
                    ));
                }
            }
            return Err(registration_error);
        }
        *active = Some(requested.clone());
        Ok(())
    }
}

fn shortcut(hotkey: &Hotkey) -> Shortcut {
    let mut modifiers = Modifiers::empty();
    if hotkey.control() {
        modifiers |= Modifiers::CONTROL;
    }
    if hotkey.alt() {
        modifiers |= Modifiers::ALT;
    }
    if hotkey.shift() {
        modifiers |= Modifiers::SHIFT;
    }
    if hotkey.super_key() {
        modifiers |= Modifiers::SUPER;
    }
    Shortcut::new(Some(modifiers), code(hotkey.key()))
}

fn code(key: &HotkeyKey) -> Code {
    match key {
        HotkeyKey::Letter('A') => Code::KeyA,
        HotkeyKey::Letter('B') => Code::KeyB,
        HotkeyKey::Letter('C') => Code::KeyC,
        HotkeyKey::Letter('D') => Code::KeyD,
        HotkeyKey::Letter('E') => Code::KeyE,
        HotkeyKey::Letter('F') => Code::KeyF,
        HotkeyKey::Letter('G') => Code::KeyG,
        HotkeyKey::Letter('H') => Code::KeyH,
        HotkeyKey::Letter('I') => Code::KeyI,
        HotkeyKey::Letter('J') => Code::KeyJ,
        HotkeyKey::Letter('K') => Code::KeyK,
        HotkeyKey::Letter('L') => Code::KeyL,
        HotkeyKey::Letter('M') => Code::KeyM,
        HotkeyKey::Letter('N') => Code::KeyN,
        HotkeyKey::Letter('O') => Code::KeyO,
        HotkeyKey::Letter('P') => Code::KeyP,
        HotkeyKey::Letter('Q') => Code::KeyQ,
        HotkeyKey::Letter('R') => Code::KeyR,
        HotkeyKey::Letter('S') => Code::KeyS,
        HotkeyKey::Letter('T') => Code::KeyT,
        HotkeyKey::Letter('U') => Code::KeyU,
        HotkeyKey::Letter('V') => Code::KeyV,
        HotkeyKey::Letter('W') => Code::KeyW,
        HotkeyKey::Letter('X') => Code::KeyX,
        HotkeyKey::Letter('Y') => Code::KeyY,
        HotkeyKey::Letter('Z') => Code::KeyZ,
        HotkeyKey::Digit('0') => Code::Digit0,
        HotkeyKey::Digit('1') => Code::Digit1,
        HotkeyKey::Digit('2') => Code::Digit2,
        HotkeyKey::Digit('3') => Code::Digit3,
        HotkeyKey::Digit('4') => Code::Digit4,
        HotkeyKey::Digit('5') => Code::Digit5,
        HotkeyKey::Digit('6') => Code::Digit6,
        HotkeyKey::Digit('7') => Code::Digit7,
        HotkeyKey::Digit('8') => Code::Digit8,
        HotkeyKey::Digit('9') => Code::Digit9,
        HotkeyKey::Function(1) => Code::F1,
        HotkeyKey::Function(2) => Code::F2,
        HotkeyKey::Function(3) => Code::F3,
        HotkeyKey::Function(4) => Code::F4,
        HotkeyKey::Function(5) => Code::F5,
        HotkeyKey::Function(6) => Code::F6,
        HotkeyKey::Function(7) => Code::F7,
        HotkeyKey::Function(8) => Code::F8,
        HotkeyKey::Function(9) => Code::F9,
        HotkeyKey::Function(10) => Code::F10,
        HotkeyKey::Function(11) => Code::F11,
        HotkeyKey::Function(12) => Code::F12,
        HotkeyKey::Function(13) => Code::F13,
        HotkeyKey::Function(14) => Code::F14,
        HotkeyKey::Function(15) => Code::F15,
        HotkeyKey::Function(16) => Code::F16,
        HotkeyKey::Function(17) => Code::F17,
        HotkeyKey::Function(18) => Code::F18,
        HotkeyKey::Function(19) => Code::F19,
        HotkeyKey::Function(20) => Code::F20,
        HotkeyKey::Function(21) => Code::F21,
        HotkeyKey::Function(22) => Code::F22,
        HotkeyKey::Function(23) => Code::F23,
        HotkeyKey::Function(24) => Code::F24,
        HotkeyKey::Letter(_) | HotkeyKey::Digit(_) | HotkeyKey::Function(_) => {
            unreachable!("Hotkey validation only permits supported keys")
        }
    }
}
