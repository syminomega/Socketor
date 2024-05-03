use std::io::Read;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Mutex};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;

//TODO:Multi windows support

//use lazy_static::lazy_static;
static CLIENTS: Mutex<Vec<TcpStream>> = Mutex::new(Vec::new());
static SERVER_HANDLE: Mutex<Option<std::thread::JoinHandle<bool>>> = Mutex::new(None);
static SERVER_SHOULD_SHUTDOWN: AtomicBool = AtomicBool::new(false);
static LOCAL_ADDR: Mutex<Option<SocketAddr>> = Mutex::new(None);


#[tauri::command]
pub fn start_tcp_server(host: &str) -> bool {
    let server = TcpListener::bind(host);
    let server = match server {
        Err(_error) => {
            println!("{}", _error);
            //TODO:返回错误信息
            return false;
        }
        Ok(_server) => {
            println!("Listening at {}", host);
            _server
        }
    };

    //let (tx, rx) = mpsc::channel();
    // 克隆一个发送者
    //let tx1 = tx.clone();

    SERVER_SHOULD_SHUTDOWN.store(false, std::sync::atomic::Ordering::Relaxed);
    let local_addr = server.local_addr().unwrap();
    LOCAL_ADDR.lock().unwrap().replace(local_addr);

    let server_thread = std::thread::spawn(move || {
        for stream in server.incoming() {
            //收到停止信号
            if SERVER_SHOULD_SHUTDOWN.load(std::sync::atomic::Ordering::Relaxed) {
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
        true
    });
    SERVER_HANDLE.lock().unwrap().replace(server_thread);
    true
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let read = stream.read(&mut buffer);
    //println!("收到客户端结果:{}", String::from_utf8_lossy(&buffer[..]));
}

#[tauri::command]
pub fn stop_tcp_server() -> bool {
    let mut stopped = false;
    //set the flag to stop the server
    SERVER_SHOULD_SHUTDOWN.store(true, std::sync::atomic::Ordering::Relaxed);
    //send a dummy connection to unblock the server
    if let Some(local_addr) = LOCAL_ADDR.lock().unwrap().take() {
        let _ = TcpStream::connect_timeout(&local_addr, std::time::Duration::from_secs(1));
    }

    if let Some(handle) = SERVER_HANDLE.lock().unwrap().take() {
        stopped = handle.join().unwrap();
    }
    //disconnect and clear all clients

    let mut clients = CLIENTS.lock().unwrap();
    for client in clients.iter() {
        client.shutdown(std::net::Shutdown::Both);
    }
    clients.clear();

    if stopped {
        println!("TCP Server 停止监听");
        true
    } else {
        println!("TCP Server 停止监听失败");
        false
    }
}
