use tauri::Manager;
use tokio::sync::Mutex;

use tcp_client::TcpClientManager;
use tcp_server::TcpServerManager;
use websocket_server::WebSocketServerManager;

mod tcp_client;
mod tcp_server;
mod websocket_server;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(WebSocketServerManager::default()));
            app.manage(Mutex::new(TcpServerManager::default()));
            app.manage(Mutex::new(TcpClientManager::default()));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            websocket_server::start_websocket_server,
            websocket_server::stop_websocket_server,
            websocket_server::send_websocket_message,
            websocket_server::get_websocket_servers,
            websocket_server::get_websocket_server_info,
            tcp_server::start_tcp_server,
            tcp_server::stop_tcp_server,
            tcp_server::send_tcp_message,
            tcp_server::get_tcp_servers,
            tcp_server::get_tcp_server_info,
            tcp_client::connect_tcp_client,
            tcp_client::disconnect_tcp_client,
            tcp_client::send_tcp_client_message,
            tcp_client::get_tcp_clients,
            tcp_client::get_tcp_client_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}