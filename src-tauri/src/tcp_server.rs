use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tauri::{State, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::JoinHandle;
use uuid::Uuid;
use chrono;

// TCP客户端连接
#[allow(dead_code)]
pub struct TcpClient {
    pub id: String,
    pub addr: SocketAddr,
    pub sender: mpsc::UnboundedSender<Vec<u8>>,
}

// TCP服务器
pub struct TcpServer {
    pub host: String,
    pub port: u16,
    pub server_id: String,
    pub clients: Arc<RwLock<HashMap<String, TcpClient>>>,
    pub server_handle: Option<JoinHandle<()>>,
    pub shutdown_sender: Option<mpsc::UnboundedSender<()>>,
    pub app_handle: Option<tauri::AppHandle>,
}

// TCP服务器管理器
pub struct TcpServerManager {
    pub servers: HashMap<String, TcpServer>,
}

impl TcpServerManager {
    pub fn new() -> Self {
        TcpServerManager {
            servers: HashMap::new(),
        }
    }
}

impl Default for TcpServerManager {
    fn default() -> Self {
        Self::new()
    }
}

// 启动TCP服务器的参数
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartTcpServerParams {
    pub host: String,
    pub port: u16,
    pub server_id: Option<String>,
}

// 发送消息的参数
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendTcpMessageParams {
    pub server_id: String,
    pub message: String,
    pub target_client_id: Option<String>, // 如果为None则广播给所有客户端
    pub message_type: Option<String>, // "text" 或 "hex"，默认为 "text"
}

// 服务器状态信息
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcpServerInfo {
    pub server_id: String,
    pub host: String,
    pub port: u16,
    pub client_count: usize,
    pub is_running: bool,
}

// TCP事件数据（发送给前端）
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TcpServerEvent {
    pub server_id: String,
    pub event_type: String,
    pub client_id: String,
    pub message: String,
    pub timestamp: String,
}

impl TcpServer {
    pub fn new(host: String, port: u16, server_id: String) -> Self {
        TcpServer {
            host,
            port,
            server_id,
            clients: Arc::new(RwLock::new(HashMap::new())),
            server_handle: None,
            shutdown_sender: None,
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, app_handle: tauri::AppHandle) {
        self.app_handle = Some(app_handle);
    }

    pub async fn start(&mut self) -> Result<(), String> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

        let clients = Arc::clone(&self.clients);
        let app_handle = self.app_handle.clone();
        let server_id = self.server_id.clone();
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        self.shutdown_sender = Some(shutdown_tx);

        let server_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 检查是否收到关闭信号
                    _ = shutdown_rx.recv() => {
                        println!("TCP server shutting down...");
                        break;
                    }
                    // 接受新的连接
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((stream, addr)) => {
                                let clients_clone = Arc::clone(&clients);
                                let app_handle_clone = app_handle.clone();
                                let server_id_clone = server_id.clone();
                                tokio::spawn(handle_tcp_connection(stream, addr, clients_clone, app_handle_clone, server_id_clone));
                            }
                            Err(e) => {
                                eprintln!("Failed to accept TCP connection: {}", e);
                            }
                        }
                    }
                }
            }
        });

        self.server_handle = Some(server_handle);
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        // 发送关闭信号
        if let Some(shutdown_sender) = self.shutdown_sender.take() {
            let _ = shutdown_sender.send(());
        }

        // 等待服务器任务完成
        if let Some(handle) = self.server_handle.take() {
            handle.await.map_err(|e| format!("Failed to stop TCP server: {}", e))?;
        }

        // 关闭所有客户端连接
        let mut clients = self.clients.write().await;
        clients.clear();

        Ok(())
    }

    pub async fn send_message_to_client(&self, client_id: &str, data: Vec<u8>) -> Result<(), String> {
        let clients = self.clients.read().await;
        if let Some(client) = clients.get(client_id) {
            client
                .sender
                .send(data)
                .map_err(|e| format!("Failed to send message to client {}: {}", client_id, e))?;
            Ok(())
        } else {
            Err(format!("Client {} not found", client_id))
        }
    }

    pub async fn broadcast_message(&self, data: Vec<u8>) -> Result<usize, String> {
        let clients = self.clients.read().await;
        let mut sent_count = 0;

        for (_, client) in clients.iter() {
            if client.sender.send(data.clone()).is_ok() {
                sent_count += 1;
            }
        }

        Ok(sent_count)
    }

    pub async fn get_client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    pub fn is_running(&self) -> bool {
        self.server_handle.is_some()
    }
}

// 处理TCP连接
async fn handle_tcp_connection(
    stream: TcpStream,
    addr: SocketAddr,
    clients: Arc<RwLock<HashMap<String, TcpClient>>>,
    app_handle: Option<tauri::AppHandle>,
    server_id: String,
) {
    let client_id = Uuid::new_v4().to_string();
    println!("New TCP client connected: {} ({})", client_id, addr);

    // 发送客户端连接事件到前端
    if let Some(ref app) = app_handle {
        let event = TcpServerEvent {
            server_id: server_id.clone(),
            event_type: "client_connected".to_string(),
            client_id: client_id.clone(),
            message: format!("Client connected from {}", addr),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        if let Err(e) = app.emit("tcp-server-event", &event) {
            eprintln!("Failed to emit connection event to frontend: {}", e);
        }
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();

    // 添加客户端到集合
    {
        let mut clients_guard = clients.write().await;
        clients_guard.insert(
            client_id.clone(),
            TcpClient {
                id: client_id.clone(),
                addr,
                sender: tx,
            },
        );
    }

    // 分离读写流 - 使用into_split来获得拥有所有权的流
    let (mut reader, mut writer) = stream.into_split();

    // 启动发送任务
    let client_id_sender = client_id.clone();
    let send_task = tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            if writer.write_all(&data).await.is_err() {
                println!("Failed to send data to client {}", client_id_sender);
                break;
            }
            if writer.flush().await.is_err() {
                println!("Failed to flush data to client {}", client_id_sender);
                break;
            }
        }
    });

    // 接收消息循环
    let client_id_receiver = client_id.clone();
    let clients_clone = Arc::clone(&clients);
    let app_handle_clone = app_handle.clone();
    let server_id_clone = server_id.clone();
    let receive_task = tokio::spawn(async move {
        let mut buffer = [0; 1024];
        
        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => {
                    // 连接关闭
                    println!("Client {} disconnected", client_id_receiver);
                    
                    // 发送客户端断开事件到前端
                    if let Some(ref app) = app_handle_clone {
                        let event = TcpServerEvent {
                            server_id: server_id_clone.clone(),
                            event_type: "client_disconnected".to_string(),
                            client_id: client_id_receiver.clone(),
                            message: "Client disconnected".to_string(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                        
                        if let Err(e) = app.emit("tcp-server-event", &event) {
                            eprintln!("Failed to emit disconnection event to frontend: {}", e);
                        }
                    }
                    break;
                }
                Ok(n) => {
                    let received_data = &buffer[..n];
                    
                    // 尝试将数据转换为文本，如果失败则作为十六进制处理
                    let message = match std::str::from_utf8(received_data) {
                        Ok(text) => {
                            println!("Received text from {}: {}", client_id_receiver, text);
                            text.to_string()
                        }
                        Err(_) => {
                            let hex_string = received_data.iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<Vec<String>>()
                                .join(" ");
                            println!("Received binary data from {}: {}", client_id_receiver, hex_string);
                            format!("Binary data: {}", hex_string)
                        }
                    };
                    
                    // 发送事件到前端
                    if let Some(ref app) = app_handle_clone {
                        let event = TcpServerEvent {
                            server_id: server_id_clone.clone(),
                            event_type: "message_received".to_string(),
                            client_id: client_id_receiver.clone(),
                            message,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                        
                        if let Err(e) = app.emit("tcp-server-event", &event) {
                            eprintln!("Failed to emit event to frontend: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("TCP error for client {}: {}", client_id_receiver, e);
                    break;
                }
            }
        }

        // 从客户端集合中移除
        clients_clone.write().await.remove(&client_id_receiver);
    });

    // 等待任何一个任务完成
    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }

    // 清理：从客户端集合中移除
    clients.write().await.remove(&client_id);
    println!("Client {} disconnected and cleaned up", client_id);
}

// 辅助函数：解析十六进制字符串为字节数组
fn parse_hex_string(hex_str: &str) -> Result<Vec<u8>, String> {
    let cleaned = hex_str.replace(" ", "").replace("\n", "").replace("\r", "");
    
    if cleaned.len() % 2 != 0 {
        return Err("Hex string must have even number of characters".to_string());
    }
    
    let mut bytes = Vec::new();
    for i in (0..cleaned.len()).step_by(2) {
        let hex_byte = &cleaned[i..i+2];
        match u8::from_str_radix(hex_byte, 16) {
            Ok(byte) => bytes.push(byte),
            Err(_) => return Err(format!("Invalid hex characters: {}", hex_byte)),
        }
    }
    
    Ok(bytes)
}

// Tauri命令：启动TCP服务器
#[tauri::command]
pub async fn start_tcp_server(
    app_handle: tauri::AppHandle,
    start_params: StartTcpServerParams,
    state: State<'_, Mutex<TcpServerManager>>,
) -> Result<String, String> {
    let server_id = start_params.server_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let mut manager = state.lock().await;

    // 检查服务器ID是否已存在
    if manager.servers.contains_key(&server_id) {
        return Err(format!("TCP Server with ID {} already exists", server_id));
    }

    let mut server = TcpServer::new(start_params.host.clone(), start_params.port, server_id.clone());
    server.set_app_handle(app_handle);
    server.start().await?;

    manager.servers.insert(server_id.clone(), server);
    Ok(server_id)
}

// Tauri命令：停止TCP服务器
#[tauri::command]
pub async fn stop_tcp_server(
    server_id: Option<String>,
    state: State<'_, Mutex<TcpServerManager>>,
) -> Result<(), String> {
    let server_id = server_id.ok_or("Server ID is required")?;
    
    if server_id.is_empty() {
        return Err("Server ID cannot be empty".to_string());
    }
    
    let mut manager = state.lock().await;

    if let Some(server) = manager.servers.get_mut(&server_id) {
        server.stop().await?;
        manager.servers.remove(&server_id);
        Ok(())
    } else {
        Err(format!("TCP Server with ID {} not found", server_id))
    }
}

// Tauri命令：发送消息
#[tauri::command]
pub async fn send_tcp_message(
    send_params: SendTcpMessageParams,
    state: State<'_, Mutex<TcpServerManager>>,
) -> Result<String, String> {
    if send_params.server_id.is_empty() {
        return Err("Server ID cannot be empty".to_string());
    }
    
    if send_params.message.is_empty() {
        return Err("Message cannot be empty".to_string());
    }
    
    let manager = state.lock().await;

    if let Some(server) = manager.servers.get(&send_params.server_id) {
        // 根据消息类型处理数据
        let data = match send_params.message_type.as_deref().unwrap_or("text") {
            "hex" => {
                // 解析十六进制字符串
                parse_hex_string(&send_params.message)?
            }
            _ => {
                // 默认作为文本处理
                send_params.message.as_bytes().to_vec()
            }
        };

        if let Some(target_client_id) = send_params.target_client_id {
            // 发送给特定客户端
            server.send_message_to_client(&target_client_id, data).await?;
            Ok(format!("Message sent to client {}", target_client_id))
        } else {
            // 广播给所有客户端
            let sent_count = server.broadcast_message(data).await?;
            Ok(format!("Message broadcast to {} clients", sent_count))
        }
    } else {
        Err(format!("TCP Server with ID {} not found", send_params.server_id))
    }
}

// Tauri命令：获取服务器列表
#[tauri::command]
pub async fn get_tcp_servers(
    state: State<'_, Mutex<TcpServerManager>>,
) -> Result<Vec<TcpServerInfo>, String> {
    let manager = state.lock().await;
    let mut servers_info = Vec::new();

    for (server_id, server) in manager.servers.iter() {
        servers_info.push(TcpServerInfo {
            server_id: server_id.clone(),
            host: server.host.clone(),
            port: server.port,
            client_count: server.get_client_count().await,
            is_running: server.is_running(),
        });
    }

    Ok(servers_info)
}

// Tauri命令：获取特定服务器信息
#[tauri::command]
pub async fn get_tcp_server_info(
    server_id: Option<String>,
    state: State<'_, Mutex<TcpServerManager>>,
) -> Result<TcpServerInfo, String> {
    let server_id = server_id.ok_or("Server ID is required")?;
    println!("Fetching info for TCP server ID: {}", server_id);
    if server_id.is_empty() {
        return Err("Server ID cannot be empty".to_string());
    }
    
    let manager = state.lock().await;
    println!("Current TCP servers: {:?}", manager.servers.keys());

    if let Some(server) = manager.servers.get(&server_id) {
        println!("Found TCP server: {}:{}, is running: {}", server.host, server.port, server.is_running());
        Ok(TcpServerInfo {
            server_id: server_id.clone(),
            host: server.host.clone(),
            port: server.port,
            client_count: server.get_client_count().await,
            is_running: server.is_running(),
        })
    } else {
        println!("TCP Server with ID {} not found", server_id);
        Err(format!("TCP Server with ID {} not found", server_id))
    }
}
