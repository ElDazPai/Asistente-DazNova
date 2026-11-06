//! Atajo global que muestra u oculta el asistente desde cualquier aplicación.

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// Combinación por defecto. Se elige para no chocar con IME (`Ctrl+Space`)
/// ni con el menú de ventana de Windows (`Alt+Space`).
pub const DEFAULT_SHORTCUT: &str = "Alt+Shift+Space";

pub fn init(app: &AppHandle) -> Result<(), String> {
    let shortcut = Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::Space);

    app.global_shortcut()
        .on_shortcut(shortcut, |app, _shortcut, event| {
            // Solo al pulsar: el evento también llega al soltar.
            if event.state() == ShortcutState::Pressed {
                super::toggle(app);
            }
        })
        .map_err(|error| format!("{DEFAULT_SHORTCUT}: {error}"))
}
