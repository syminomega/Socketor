use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Mutex};
use std::sync::atomic::AtomicBool;
use lazy_static::lazy_static;

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
    server_should_shutdown: AtomicBool,
    local_addr: Mutex<Option<SocketAddr>>,
}

impl TcpServerInstance {
    pub fn new(label: &str, host: &str) -> TcpServerInstance {
        let instance = TcpServerInstance {
            label: label.into(),
            host: host.into(),
            server_should_shutdown: AtomicBool::new(false),
            local_addr: Mutex::new(None),
        };
        instance
    }
    pub fn init(self) -> Result<(),Box<dyn Error>>  {
        let instance_label = self.label.clone();
        // insert the instance into the global instances map
        let instance = &SERVER_INSTANCES.lock()?.insert(self.label.clone(), self).unwrap();

        let server = TcpListener::bind(&instance.host)?;

        let local_addr = server.local_addr().unwrap();
        instance.local_addr.lock().unwrap().replace(local_addr);

        let instance_label_thread = instance_label.clone();
        std::thread::spawn(move || {
            for stream in server.incoming() {
                let instance = SERVER_INSTANCES.lock().unwrap();
                let instance = instance.get(&instance_label_thread).unwrap();
                //收到停止信号
                if instance.server_should_shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                //处理新的连接
                match stream {
                    Ok(stream) => {
                        println!("新的客户端连接");
                        CLIENTS.lock().unwrap().push(stream);
                    }
                    Err(e) => {
                        println!("连接失败:{}", e);
                    }
                }
            }

        });
        Ok(())
    }
}

static CLIENTS: Mutex<Vec<TcpStream>> = Mutex::new(Vec::new());


#[tauri::command]
pub fn start_tcp_server(label: &str, host: &str) -> bool {
    true
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let read = stream.read(&mut buffer);
    //println!("收到客户端结果:{}", String::from_utf8_lossy(&buffer[..]));
}

#[tauri::command]
pub fn stop_tcp_server(label: &str) -> bool {
    //set the flag to stop the server
    let mut instances = SERVER_INSTANCES.lock().unwrap();

    match instances.get(label) {
        Some(instance) => {
            instance.server_should_shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
            //send a dummy connection to unblock the server
            if let Some(local_addr) = instance.local_addr.lock().unwrap().take() {
                let _ = TcpStream::connect_timeout(&local_addr, std::time::Duration::from_secs(1));
            }
            instances.remove(label);
        }
        None => {
            println!("未找到实例:{}", label);
            return false;
        }
    }

    //disconnect and clear all clients

    let mut clients = CLIENTS.lock().unwrap();
    for client in clients.iter() {
        client.shutdown(std::net::Shutdown::Both);
    }
    clients.clear();

    println!("TCP Server 停止监听");
    true
}
