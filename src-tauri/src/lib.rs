use socketor_tcp_server::{TcpServerManager, start_tcp_server};
use websocket_server::{
    WebSocketServerManager, start_websocket_server, stop_websocket_server, 
    send_websocket_message, get_websocket_servers, get_websocket_server_info
};
use tokio::sync::Mutex;
use tauri::Manager;

mod socketor_tcp_server;
mod websocket_server;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(TcpServerManager::default()));
            app.manage(Mutex::new(WebSocketServerManager::default()));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            start_tcp_server,
            start_websocket_server,
            stop_websocket_server,
            send_websocket_message,
            get_websocket_servers,
            get_websocket_server_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
