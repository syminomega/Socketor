use std::collections::HashMap;
use tauri::State;
use tokio::net::unix::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

struct TcpServer {
    listener: TcpListener,
    clients: HashMap<SocketAddr, TcpStream>,
}

pub(crate) struct TcpServerManager {
    tcp_server_collection: HashMap<SocketAddr, TcpServer>,
}

impl TcpServerManager {
    pub(crate) fn default() -> Self {
        TcpServerManager {
            tcp_server_collection: HashMap::new(),
        }
    }
}

#[tauri::command]
pub(crate) async fn start_tcp_server(address: &str, state: State<'_, Mutex<TcpServerManager>>) -> Result<(), String> {
    let listener = TcpListener::bind(address).await.unwrap();
    Ok(())
}
