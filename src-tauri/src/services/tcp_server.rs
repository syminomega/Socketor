use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Mutex};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use lazy_static::lazy_static;
use tauri::{AppHandle, Window};
use crate::services::communication::{ActionResult, show_message};

// all instances of the server
lazy_static! {
    static ref SERVER_INSTANCES: Mutex<HashMap<String, TcpServerInstance>> = {
        let map = HashMap::new();
        Mutex::new(map)
    };
}

pub struct TcpServerInstance {
    label: String,
    host: String,
    window: Window,
    server_should_shutdown: AtomicBool,
    local_addr: Mutex<Option<SocketAddr>>,
    clients: Mutex<Vec<TcpStream>>,
}

impl TcpServerInstance {
    pub fn new(window: Window, host: &str) -> TcpServerInstance {
        TcpServerInstance {
            label: window.label().to_string(),
            host: host.into(),
            window,
            server_should_shutdown: AtomicBool::new(false),
            local_addr: Mutex::new(None),
            clients: Mutex::new(Vec::new()),
        }
    }
    pub fn init(self) -> Result<(), Box<dyn Error>> {
        let instance_label = self.label.clone();
        // insert the instance into the global instances map
        SERVER_INSTANCES.lock()?.insert(self.label.clone(), self);
        let instance = SERVER_INSTANCES.lock()?;
        let instance = instance.get(&instance_label).unwrap();

        let server = TcpListener::bind(&instance.host)?;
        println!("TCP Server started at {}", &instance.host);

        let local_addr = server.local_addr().unwrap();
        instance.local_addr.lock().unwrap().replace(local_addr);

        instance.handle_incoming_connection(server);
        Ok(())
    }
    fn handle_incoming_connection(&self, server: TcpListener) {
        //let (sender, receiver) = mpsc::channel();
        let label = self.label.to_string();
        let window = self.window.clone();
        std::thread::spawn(move || {
            for stream in server.incoming() {
                let instance = SERVER_INSTANCES.lock().unwrap();
                let instance = instance.get(&label).unwrap();
                //收到停止信号
                if instance.server_should_shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                //处理新的连接
                match stream {
                    Ok(stream) => {
                        println!("新的客户端连接");
                        instance.clients.lock().unwrap().push(stream);
                        show_message("新的客户端连接", crate::services::communication::MessageType::Log, &window);
                    }
                    Err(e) => {
                        println!("连接失败:{}", e);
                    }
                }
            }
        });
    }


}


fn handle_connection(mut stream: &TcpStream) {
    let mut buffer = [0; 1024];
    let read = stream.read(&mut buffer);
    //println!("收到客户端结果:{}", String::from_utf8_lossy(&buffer[..]));
}

#[tauri::command]
pub fn is_tcp_server_running(window: Window) -> bool {
    let instances = SERVER_INSTANCES.lock().unwrap();
    instances.contains_key(window.label())
}

#[tauri::command]
pub fn start_tcp_server(window: Window, host: &str) -> ActionResult {
    let label = window.label();
    let instance = TcpServerInstance::new(window, host);
    match instance.init() {
        Ok(_) => {
            ActionResult {
                success: true,
                error_message: "".to_string(),
            }
        }
        Err(e) => {
            ActionResult {
                success: false,
                error_message: e.to_string(),
            }
        }
    }
}

#[tauri::command]
pub fn stop_tcp_server(window: Window) -> ActionResult {
    stop_tcp_server_base(&window)
}

pub fn stop_tcp_server_base(window: &Window) -> ActionResult {
    let label = window.label();
    //set the flag to stop the server
    let instances = SERVER_INSTANCES.lock().unwrap();

    match instances.get(label) {
        Some(instance) => {
            instance.server_should_shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
            //send a dummy connection to unblock the server
            if let Some(local_addr) = instance.local_addr.lock().unwrap().take() {
                let _ = TcpStream::connect_timeout(&local_addr, std::time::Duration::from_secs(1));
                println!("stopping server at {}", local_addr);
            }
            //disconnect and clear all clients
            let mut clients = instance.clients.lock().unwrap();
            for client in clients.iter() {
                match client.shutdown(std::net::Shutdown::Both) {
                    Ok(_) => {
                        println!("关闭客户端连接");
                    }
                    Err(e) => {
                        println!("关闭客户端连接失败:{}", e);
                    }
                }
            }
            clients.clear();
        }
        None => {
            return ActionResult {
                success: false,
                error_message: "未找到实例".to_string(),
            };
        }
    }

    ActionResult {
        success: true,
        error_message: "".to_string(),
    }
}
