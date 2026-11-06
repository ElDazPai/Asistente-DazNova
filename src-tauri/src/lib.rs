//! Asistente DazNova — backend nativo.
//!
//! Aquí se ensambla la aplicación: plugins de Tauri, estado compartido y
//! registro de comandos. La lógica de cada dominio vive en su propio módulo.

mod ai_engine;
mod calendar;
mod db;
mod gmail;
mod memory;
mod notifications;
mod scheduler;
mod secrets;
mod system;
mod web_search;
mod window;

use serde::Serialize;

/// Información que el frontend usa para confirmar que el backend responde.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendStatus {
    pub version: String,
    pub os: String,
    pub arch: String,
}

#[tauri::command]
fn backend_status(app: tauri::AppHandle) -> BackendStatus {
    BackendStatus {
        version: app.package_info().version.to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(window::Runtime::default())
        .manage(ai_engine::AiRuntime::new())
        .invoke_handler(tauri::generate_handler![
            backend_status,
            window::set_window_state,
            window::get_window_state,
            window::toggle_window,
            window::hide_window,
            window::start_drag,
            window::is_autostart_enabled,
            window::set_autostart,
            ai_engine::get_ai_settings,
            ai_engine::save_ai_settings,
            ai_engine::save_api_key,
            ai_engine::validate_api_key,
            ai_engine::get_daily_usage,
            ai_engine::list_conversations,
            ai_engine::create_conversation,
            ai_engine::archive_conversation,
            ai_engine::list_messages,
            ai_engine::cancel_generation,
            ai_engine::send_message,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            match db::init(&handle) {
                Ok(database) => {
                    handle.manage(database);
                }
                Err(error) => {
                    eprintln!("[bd] no se pudo abrir: {error}");
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        error,
                    )));
                }
            }

            // Ningún fallo de arranque debe impedir que la ventana aparezca:
            // se registra y se sigue, para que el usuario vea algo y no un proceso mudo.
            if let Err(error) = window::init(&handle) {
                eprintln!("[ventana] no se pudo inicializar: {error}");
            }
            if let Err(error) = window::shortcut::init(&handle) {
                eprintln!("[atajo] no se pudo registrar: {error}");
            }
            if let Err(error) = window::tray::init(&handle) {
                eprintln!("[bandeja] no se pudo crear: {error}");
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                window::handle_event(window, event);
            }
        })
        .run(tauri::generate_context!())
        .expect("error al iniciar Asistente DazNova");
}
