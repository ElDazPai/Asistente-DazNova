//! Estados de ventana, posición persistida y comportamiento flotante.
//!
//! Una sola ventana (`main`) cambia de tamaño entre tres estados. El backend es
//! el único que toca la ventana: el frontend solo pide un estado y escucha el
//! evento `window-state-changed`.
//!
//! Regla de oro de esta etapa: invocar el asistente no debe alterar el trabajo
//! del usuario. La ventana flota sobre la aplicación activa, no la reemplaza.

pub mod shortcut;
pub mod tray;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewWindow, WindowEvent,
};

/// Etiqueta de la única ventana de la aplicación.
const LABEL: &str = "main";

/// Margen mínimo respecto al borde de la pantalla, en píxeles físicos.
const SCREEN_MARGIN: i32 = 8;

/// Como máximo se persiste la posición una vez por segundo mientras se arrastra.
const SAVE_THROTTLE: Duration = Duration::from_secs(1);

/// Los tres tamaños de la ventana.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WindowState {
    /// Orbe de 72 px, siempre visible, sin entrada de texto.
    #[default]
    Widget,
    /// Panel de conversación compacto.
    Expanded,
    /// Panel completo con historial, memoria y ajustes.
    Dashboard,
}

impl WindowState {
    /// Tamaño lógico de la ventana en este estado.
    fn size(self) -> (f64, f64) {
        match self {
            WindowState::Widget => (72.0, 72.0),
            WindowState::Expanded => (420.0, 600.0),
            WindowState::Dashboard => (1100.0, 760.0),
        }
    }

    /// Tamaño mínimo permitido (solo el panel completo es redimensionable).
    fn min_size(self) -> (f64, f64) {
        match self {
            WindowState::Dashboard => (880.0, 600.0),
            other => other.size(),
        }
    }

    /// Si flota sobre el resto de ventanas y vive fuera de la barra de tareas.
    fn floating(self) -> bool {
        !matches!(self, WindowState::Dashboard)
    }
}

/// Estado en memoria que comparten los comandos y el manejador de eventos.
#[derive(Default)]
pub struct Runtime {
    state: Mutex<WindowState>,
    last_save: Mutex<Option<Instant>>,
}

/// Forma en disco de la posición recordada.
#[derive(Debug, Serialize, Deserialize)]
struct SavedPosition {
    x: i32,
    y: i32,
}

fn window(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window(LABEL)
        .ok_or_else(|| "la ventana principal no está disponible".to_string())
}

/// Diferencias por sistema operativo al manipular la ventana.
#[cfg(target_os = "windows")]
mod platform {
    use std::ffi::c_void;

    use tauri::WebviewWindow;

    const GWL_STYLE: i32 = -16;
    /// Barra de título y botones de sistema.
    const WS_SYSTEM_CHROME: i32 = 0x00CB_0000;
    const SWP_NOSIZE: u32 = 0x0001;
    const SWP_NOMOVE: u32 = 0x0002;
    const SWP_NOZORDER: u32 = 0x0004;
    const SWP_NOACTIVATE: u32 = 0x0010;
    const SWP_FRAMECHANGED: u32 = 0x0020;

    const SW_SHOWNOACTIVATE: i32 = 4;

    #[link(name = "user32")]
    extern "system" {
        fn GetWindowLongW(hwnd: *mut c_void, index: i32) -> i32;
        fn SetWindowLongW(hwnd: *mut c_void, index: i32, value: i32) -> i32;
        fn ShowWindow(hwnd: *mut c_void, cmd: i32) -> i32;
        fn SetWindowPos(
            hwnd: *mut c_void,
            insert_after: *mut c_void,
            x: i32,
            y: i32,
            cx: i32,
            cy: i32,
            flags: u32,
        ) -> i32;
    }

    /// Tao vuelve a añadir `WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX |
    /// WS_MAXIMIZEBOX` en cada recálculo de estilo, aunque la ventana sea sin
    /// bordes. Con esos bits, Windows impone un ancho mínimo de 120 px (136 con
    /// caption), así que el orbe de 72 px no cabría. Se quitan justo antes de
    /// fijar el tamaño.
    pub fn strip_system_chrome(window: &WebviewWindow) {
        let Ok(handle) = window.hwnd() else {
            return;
        };
        let hwnd = handle.0 as *mut c_void;

        unsafe {
            let style = GetWindowLongW(hwnd, GWL_STYLE);
            let stripped = style & !WS_SYSTEM_CHROME;
            if stripped == style {
                return;
            }

            SetWindowLongW(hwnd, GWL_STYLE, stripped);
            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }

    /// El cálculo de tamaño de Tao no coincide con el de una ventana sin
    /// caption, y su `set_size` se ejecuta de forma asíncrona en el hilo
    /// principal: si se combina con una llamada directa a Win32, gana la última
    /// en llegar. Por eso aquí el tamaño se fija siempre de forma directa y
    /// síncrona.
    pub fn set_exact_size(window: &WebviewWindow, width: f64, height: f64) -> Result<(), String> {
        let handle = window.hwnd().map_err(|error| error.to_string())?;
        let hwnd = handle.0 as *mut c_void;
        let scale = window.scale_factor().unwrap_or(1.0);

        let ok = unsafe {
            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                0,
                0,
                (width * scale).round() as i32,
                (height * scale).round() as i32,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };

        if ok == 0 {
            Err("no se pudo fijar el tamaño de la ventana".to_string())
        } else {
            Ok(())
        }
    }

    /// Muestra la ventana sin activarla. Evita que el arranque robe el teclado
    /// de la aplicación en la que el usuario estaba escribiendo.
    pub fn show_inactive(window: &WebviewWindow) -> Result<(), String> {
        let handle = window.hwnd().map_err(|error| error.to_string())?;
        let hwnd = handle.0 as *mut c_void;
        unsafe {
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use tauri::{LogicalSize, WebviewWindow};

    pub fn strip_system_chrome(_window: &WebviewWindow) {}

    pub fn set_exact_size(window: &WebviewWindow, width: f64, height: f64) -> Result<(), String> {
        window
            .set_size(LogicalSize::new(width, height))
            .map_err(|error| error.to_string())
    }

    pub fn show_inactive(window: &WebviewWindow) -> Result<(), String> {
        window.show().map_err(|error| error.to_string())
    }
}

fn position_file(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join("window-position.json"))
}

fn save_position(app: &AppHandle) -> Result<(), String> {
    let win = window(app)?;
    let position = win.outer_position().map_err(|e| e.to_string())?;

    let Some(path) = position_file(app) else {
        return Ok(());
    };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }

    let payload = SavedPosition {
        x: position.x,
        y: position.y,
    };
    let bytes = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
    fs::write(path, bytes).map_err(|e| e.to_string())
}

fn load_position(app: &AppHandle) -> Option<SavedPosition> {
    let path = position_file(app)?;
    let bytes = fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Deja la ventana con el tamaño exacto del estado: quita el cromo del sistema
/// y fija el rectángulo exterior.
fn reassert(win: &WebviewWindow, state: WindowState) {
    platform::strip_system_chrome(win);
    let (width, height) = state.size();
    let _ = platform::set_exact_size(win, width, height);
}

/// Repite el ajuste en el hilo principal, al final de la cola de eventos.
///
/// Los comandos del frontend corren en un hilo aparte: sus llamadas a Tauri se
/// encolan, así que un ajuste directo desde ese hilo podría llegar antes que
/// ellas y perder la carrera. Al encolarlo aquí, se ejecuta después de los
/// cambios de estilo y del `show()`, y siempre con el estado más reciente.
fn reassert_deferred(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(win) = handle.get_webview_window(LABEL) {
            let state = *handle.state::<Runtime>().state.lock().unwrap();
            reassert(&win, state);
        }
    });
}

/// Aplica un estado: tamaño, posición anclada al centro anterior, siempre visible
/// y presencia en la barra de tareas. Emite el cambio al frontend.
pub fn apply(app: &AppHandle, next: WindowState) -> Result<(), String> {
    let win = window(app)?;

    let scale = win.scale_factor().map_err(|e| e.to_string())?;
    let previous_position = win.outer_position().map_err(|e| e.to_string())?;
    let previous_size = win.outer_size().map_err(|e| e.to_string())?;

    let (width, height) = next.size();
    let target_width = (width * scale).round() as i32;
    let target_height = (height * scale).round() as i32;

    // Crece y decrece desde su propio centro, para que el orbe no salte de sitio.
    let center_x = previous_position.x + previous_size.width as i32 / 2;
    let center_y = previous_position.y + previous_size.height as i32 / 2;
    let mut x = center_x - target_width / 2;
    let mut y = center_y - target_height / 2;

    // Nunca fuera de la pantalla actual.
    if let Ok(Some(monitor)) = win.current_monitor() {
        let origin = monitor.position();
        let size = monitor.size();
        let min_x = origin.x + SCREEN_MARGIN;
        let min_y = origin.y + SCREEN_MARGIN;
        let max_x = origin.x + size.width as i32 - target_width - SCREEN_MARGIN;
        let max_y = origin.y + size.height as i32 - target_height - SCREEN_MARGIN;
        if max_x >= min_x {
            x = x.clamp(min_x, max_x);
        }
        if max_y >= min_y {
            y = y.clamp(min_y, max_y);
        }
    }

    // El mínimo solo importa en el panel completo, que es el único
    // redimensionable; en los estados flotantes el tamaño es fijo.
    if !next.floating() {
        let (min_width, min_height) = next.min_size();
        win.set_min_size(Some(LogicalSize::new(min_width, min_height)))
            .map_err(|e| e.to_string())?;
    }
    win.set_resizable(!next.floating())
        .map_err(|e| e.to_string())?;
    win.set_always_on_top(next.floating())
        .map_err(|e| e.to_string())?;
    win.set_skip_taskbar(next.floating())
        .map_err(|e| e.to_string())?;

    *app.state::<Runtime>().state.lock().unwrap() = next;

    // Debe ir después de los cambios de estilo y antes de fijar el tamaño.
    platform::strip_system_chrome(&win);
    platform::set_exact_size(&win, width, height)?;
    win.set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;

    let _ = app.emit("window-state-changed", next);
    let _ = save_position(app);
    reassert_deferred(app);
    Ok(())
}

/// Restaura la posición guardada y arranca siempre en modo widget.
pub fn init(app: &AppHandle) -> Result<(), String> {
    let win = window(app)?;

    // Antes de medir nada: la ventana nace con cromo del sistema y Windows la
    // ensancha a 136 px. Si `apply` midiera ese tamaño, recentraría el orbe
    // desplazándolo 32 px en cada arranque.
    platform::strip_system_chrome(&win);
    let (width, height) = WindowState::Widget.size();
    let _ = platform::set_exact_size(&win, width, height);

    apply(app, WindowState::Widget)?;
    platform::show_inactive(&win)?;
    reassert(&win, WindowState::Widget);

    // `apply` recentra desde el tamaño anterior; en el arranque manda la
    // posición recordada, así que se restaura después.
    if let Some(saved) = load_position(app) {
        let _ = win.set_position(PhysicalPosition::new(saved.x, saved.y));
    }
    reassert_deferred(app);
    Ok(())
}

/// Muestra la ventana en el estado pedido y le da el foco.
pub fn open(app: &AppHandle, state: WindowState) -> Result<(), String> {
    apply(app, state)?;
    let win = window(app)?;
    win.show().map_err(|e| e.to_string())?;
    reassert(&win, state);
    reassert_deferred(app);
    win.set_focus().map_err(|e| e.to_string())
}

/// Alterna entre visible y oculta. Se usa desde el atajo global y la bandeja.
pub fn toggle(app: &AppHandle) {
    let Ok(win) = window(app) else {
        return;
    };

    if win.is_visible().unwrap_or(false) {
        let _ = win.hide();
        let _ = save_position(app);
    } else {
        let _ = win.show();
        let state = *app.state::<Runtime>().state.lock().unwrap();
        reassert(&win, state);
        reassert_deferred(app);
        let _ = win.set_focus();
    }
}

/// Reacciona a los eventos de la ventana: recuerda la posición y evita que
/// cerrarla termine con el proceso (el asistente vive en segundo plano).
pub fn handle_event(win: &tauri::Window, event: &WindowEvent) {
    match event {
        WindowEvent::Moved(_) => {
            let app = win.app_handle();
            let runtime = app.state::<Runtime>();
            let mut last_save = runtime.last_save.lock().unwrap();
            let now = Instant::now();

            let due = last_save.map_or(true, |previous| now.duration_since(previous) >= SAVE_THROTTLE);
            if due {
                *last_save = Some(now);
                drop(last_save);
                let _ = save_position(app);
            }
        }
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            let _ = win.hide();
            let _ = save_position(win.app_handle());
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Comandos expuestos al frontend
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn set_window_state(app: AppHandle, state: WindowState) -> Result<(), String> {
    open(&app, state)
}

#[tauri::command]
pub fn get_window_state(runtime: tauri::State<'_, Runtime>) -> WindowState {
    *runtime.state.lock().unwrap()
}

#[tauri::command]
pub fn toggle_window(app: AppHandle) {
    toggle(&app);
}

#[tauri::command]
pub fn hide_window(app: AppHandle) -> Result<(), String> {
    window(&app)?.hide().map_err(|e| e.to_string())?;
    let _ = save_position(&app);
    Ok(())
}

#[tauri::command]
pub fn start_drag(app: AppHandle) -> Result<(), String> {
    window(&app)?.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|e| e.to_string())
    } else {
        manager.disable().map_err(|e| e.to_string())
    }
}

/// Invierte el arranque automático. Lo usa el menú de la bandeja.
pub fn toggle_autostart(app: &AppHandle) -> Result<bool, String> {
    let enabled = is_autostart_enabled(app.clone())?;
    set_autostart(app.clone(), !enabled)?;
    Ok(!enabled)
}
