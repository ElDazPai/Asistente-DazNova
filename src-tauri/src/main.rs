// En release no se abre consola en Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    asistente_daznova_lib::run()
}
