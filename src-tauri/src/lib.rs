use websocket_server::{WebSocketServerManager};
use tokio::sync::Mutex;
use tauri::Manager;

mod websocket_server;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(WebSocketServerManager::default()));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            websocket_server::start_websocket_server,
            websocket_server::stop_websocket_server,
            websocket_server::send_websocket_message,
            websocket_server::get_websocket_servers,
            websocket_server::get_websocket_server_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
