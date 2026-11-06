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
        .invoke_handler(tauri::generate_handler![backend_status])
        .run(tauri::generate_context!())
        .expect("error al iniciar Asistente DazNova");
}
