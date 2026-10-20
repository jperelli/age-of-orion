// Hides the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Age of Orion desktop app. The game itself (Rust core compiled to WebAssembly and the
//! HTML/canvas UI from `web/`) runs inside the system webview, exactly as on the web.

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running Age of Orion");
}
