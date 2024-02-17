use std::net::TcpListener;



#[tauri::command]
pub fn start_tcp_server(host: &str) -> bool {
    let mut TCP_SERVER: Option<TcpListener> = None;
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
