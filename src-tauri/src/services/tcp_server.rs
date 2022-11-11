use std::net::TcpListener;

static mut TCP_SERVER: Option<TcpListener> = None;

#[tauri::command]
pub fn start_tcp_server(host: &str) -> bool {
    if let Some(_tcp_server)= &TCP_SERVER {
        
    }

    let server = TcpListener::bind(host);
    return match server {
        Err(_error) => {
            println!("{}", _error);
            false
        }
        Ok(_server) => unsafe {
            TCP_SERVER = Some(_server);
            println!("Listening at {}", host);
            true
        },
    };
}
