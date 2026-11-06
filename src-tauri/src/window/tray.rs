//! Icono en la bandeja del sistema con menú de control.

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::AppHandle;

use super::WindowState;

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Abrir asistente", true, None::<&str>)?;
    let dashboard = MenuItem::with_id(app, "dashboard", "Panel completo", true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, "toggle", "Mostrar u ocultar", true, None::<&str>)?;

    let autostart = CheckMenuItem::with_id(
        app,
        "autostart",
        "Iniciar con el sistema",
        true,
        super::is_autostart_enabled(app.clone()).unwrap_or(false),
        None::<&str>,
    )?;

    let separator_top = PredefinedMenuItem::separator(app)?;
    let separator_bottom = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;

    let menu = Menu::new(app)?;
    menu.append(&open)?;
    menu.append(&dashboard)?;
    menu.append(&toggle)?;
    menu.append(&separator_top)?;
    menu.append(&autostart)?;
    menu.append(&separator_bottom)?;
    menu.append(&quit)?;

    TrayIconBuilder::with_id("main-tray")
        .icon(
            app.default_window_icon()
                .expect("la aplicación siempre tiene icono")
                .clone(),
        )
        .tooltip("Asistente DazNova")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                let _ = super::open(app, WindowState::Expanded);
            }
            "dashboard" => {
                let _ = super::open(app, WindowState::Dashboard);
            }
            "toggle" => super::toggle(app),
            "autostart" => {
                // El elemento nativo ya cambió su marca al pulsarse.
                let _ = super::toggle_autostart(app);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}
