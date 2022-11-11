#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use socketor::services::tcp_server;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            tcp_server::start_tcp_server
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
