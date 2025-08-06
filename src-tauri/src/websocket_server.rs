use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tauri::{State, Emitter};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;
use chrono;

// WebSocket客户端连接
#[allow(dead_code)]
pub struct WebSocketClient {
    pub id: String,
    pub addr: SocketAddr,
    pub sender: mpsc::UnboundedSender<Message>,
}

// WebSocket服务器
pub struct WebSocketServer {
    pub host: String,
    pub port: u16,
    pub server_id: String,
    pub clients: Arc<RwLock<HashMap<String, WebSocketClient>>>,
    pub server_handle: Option<JoinHandle<()>>,
    pub shutdown_sender: Option<mpsc::UnboundedSender<()>>,
    pub app_handle: Option<tauri::AppHandle>,
}

// WebSocket服务器管理器
pub struct WebSocketServerManager {
    pub servers: HashMap<String, WebSocketServer>,
}

impl WebSocketServerManager {
    pub fn new() -> Self {
        WebSocketServerManager {
            servers: HashMap::new(),
        }
    }
}

impl Default for WebSocketServerManager {
    fn default() -> Self {
        Self::new()
    }
}

// 启动WebSocket服务器的参数
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartServerParams {
    pub host: String,
    pub port: u16,
    pub server_id: Option<String>,
}

// 发送消息的参数
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageParams {
    pub server_id: String,
    pub message: String,
    pub target_client_id: Option<String>, // 如果为None则广播给所有客户端
}

// 服务器状态信息
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub server_id: String,
    pub host: String,
    pub port: u16,
    pub client_count: usize,
    pub is_running: bool,
}

// WebSocket事件数据（发送给前端）
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WebSocketServerEvent {
    pub server_id: String,
    pub event_type: String,
    pub client_id: String,
    pub message: String,
    pub timestamp: String,
}

impl WebSocketServer {
    pub fn new(host: String, port: u16, server_id: String) -> Self {
        WebSocketServer {
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
                        println!("WebSocket server shutting down...");
                        break;
                    }
                    // 接受新的连接
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((stream, addr)) => {
                                let clients_clone = Arc::clone(&clients);
                                let app_handle_clone = app_handle.clone();
                                let server_id_clone = server_id.clone();
                                tokio::spawn(handle_connection(stream, addr, clients_clone, app_handle_clone, server_id_clone));
                            }
                            Err(e) => {
                                eprintln!("Failed to accept connection: {}", e);
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
            handle.await.map_err(|e| format!("Failed to stop server: {}", e))?;
        }

        // 关闭所有客户端连接
        let mut clients = self.clients.write().await;
        for (_, client) in clients.drain() {
            let _ = client.sender.send(Message::Close(None));
        }

        Ok(())
    }

    pub async fn send_message_to_client(&self, client_id: &str, message: &str) -> Result<(), String> {
        let clients = self.clients.read().await;
        if let Some(client) = clients.get(client_id) {
            client
                .sender
                .send(Message::Text(message.to_string()))
                .map_err(|e| format!("Failed to send message to client {}: {}", client_id, e))?;
            Ok(())
        } else {
            Err(format!("Client {} not found", client_id))
        }
    }

    pub async fn broadcast_message(&self, message: &str) -> Result<usize, String> {
        let clients = self.clients.read().await;
        let mut sent_count = 0;
        let msg = Message::Text(message.to_string());

        for (_, client) in clients.iter() {
            if client.sender.send(msg.clone()).is_ok() {
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

// 处理WebSocket连接
async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    clients: Arc<RwLock<HashMap<String, WebSocketClient>>>,
    app_handle: Option<tauri::AppHandle>,
    server_id: String,
) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Failed to accept WebSocket connection from {}: {}", addr, e);
            return;
        }
    };

    let client_id = Uuid::new_v4().to_string();
    println!("New WebSocket client connected: {} ({})", client_id, addr);

    // 发送客户端连接事件到前端
    if let Some(ref app) = app_handle {
        let event = WebSocketServerEvent {
            server_id: server_id.clone(),
            event_type: "client_connected".to_string(),
            client_id: client_id.clone(),
            message: format!("Client connected from {}", addr),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        if let Err(e) = app.emit("websocket-server-event", &event) {
            eprintln!("Failed to emit connection event to frontend: {}", e);
        }
    }

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // 添加客户端到集合
    {
        let mut clients_guard = clients.write().await;
        clients_guard.insert(
            client_id.clone(),
            WebSocketClient {
                id: client_id.clone(),
                addr,
                sender: tx,
            },
        );
    }

    // 启动发送任务
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // 接收消息循环
    let client_id_clone2 = client_id.clone();
    let clients_clone = Arc::clone(&clients);
    let app_handle_clone = app_handle.clone();
    let server_id_clone = server_id.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received from {}: {}", client_id_clone2, text);
                    
                    // 发送事件到前端
                    if let Some(ref app) = app_handle_clone {
                        let event = WebSocketServerEvent {
                            server_id: server_id_clone.clone(),
                            event_type: "message_received".to_string(),
                            client_id: client_id_clone2.clone(),
                            message: text.clone(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                        
                        if let Err(e) = app.emit("websocket-server-event", &event) {
                            eprintln!("Failed to emit event to frontend: {}", e);
                        }
                    }
                }
                Ok(Message::Binary(bin)) => {
                    println!("Received binary data from {}: {} bytes", client_id_clone2, bin.len());
                    
                    // 发送二进制数据事件到前端
                    if let Some(ref app) = app_handle_clone {
                        let event = WebSocketServerEvent {
                            server_id: server_id_clone.clone(),
                            event_type: "binary_received".to_string(),
                            client_id: client_id_clone2.clone(),
                            message: format!("Binary data: {} bytes", bin.len()),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                        
                        if let Err(e) = app.emit("websocket-server-event", &event) {
                            eprintln!("Failed to emit event to frontend: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("Client {} disconnected", client_id_clone2);
                    
                    // 发送客户端断开事件到前端
                    if let Some(ref app) = app_handle_clone {
                        let event = WebSocketServerEvent {
                            server_id: server_id_clone.clone(),
                            event_type: "client_disconnected".to_string(),
                            client_id: client_id_clone2.clone(),
                            message: "Client disconnected".to_string(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                        
                        if let Err(e) = app.emit("websocket-server-event", &event) {
                            eprintln!("Failed to emit event to frontend: {}", e);
                        }
                    }
                    break;
                }
                Err(e) => {
                    eprintln!("WebSocket error for client {}: {}", client_id_clone2, e);
                    break;
                }
                _ => {}
            }
        }

        // 从客户端集合中移除
        clients_clone.write().await.remove(&client_id_clone2);
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

// Tauri命令：启动WebSocket服务器
#[tauri::command]
pub async fn start_websocket_server(
    app_handle: tauri::AppHandle,
    start_params: StartServerParams,
    state: State<'_, Mutex<WebSocketServerManager>>,
) -> Result<String, String> {
    let server_id = start_params.server_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let mut manager = state.lock().await;

    // 检查服务器ID是否已存在
    if manager.servers.contains_key(&server_id) {
        return Err(format!("Server with ID {} already exists", server_id));
    }

    let mut server = WebSocketServer::new(start_params.host.clone(), start_params.port, server_id.clone());
    server.set_app_handle(app_handle);
    server.start().await?;

    manager.servers.insert(server_id.clone(), server);
    Ok(server_id)
}

// Tauri命令：停止WebSocket服务器
#[tauri::command]
pub async fn stop_websocket_server(
    server_id: Option<String>,
    state: State<'_, Mutex<WebSocketServerManager>>,
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
        Err(format!("Server with ID {} not found", server_id))
    }
}

// Tauri命令：发送消息
#[tauri::command]
pub async fn send_websocket_message(
    send_params: SendMessageParams,
    state: State<'_, Mutex<WebSocketServerManager>>,
) -> Result<String, String> {
    if send_params.server_id.is_empty() {
        return Err("Server ID cannot be empty".to_string());
    }
    
    if send_params.message.is_empty() {
        return Err("Message cannot be empty".to_string());
    }
    
    let manager = state.lock().await;

    if let Some(server) = manager.servers.get(&send_params.server_id) {
        if let Some(target_client_id) = send_params.target_client_id {
            // 发送给特定客户端
            server.send_message_to_client(&target_client_id, &send_params.message).await?;
            Ok(format!("Message sent to client {}", target_client_id))
        } else {
            // 广播给所有客户端
            let sent_count = server.broadcast_message(&send_params.message).await?;
            Ok(format!("Message broadcast to {} clients", sent_count))
        }
    } else {
        Err(format!("Server with ID {} not found", send_params.server_id))
    }
}

// Tauri命令：获取服务器列表
#[tauri::command]
pub async fn get_websocket_servers(
    state: State<'_, Mutex<WebSocketServerManager>>,
) -> Result<Vec<ServerInfo>, String> {
    let manager = state.lock().await;
    let mut servers_info = Vec::new();

    for (server_id, server) in manager.servers.iter() {
        servers_info.push(ServerInfo {
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
pub async fn get_websocket_server_info(
    server_id: Option<String>,
    state: State<'_, Mutex<WebSocketServerManager>>,
) -> Result<ServerInfo, String> {
    let server_id = server_id.ok_or("Server ID is required")?;
    println!("Fetching info for server ID: {}", server_id);
    if server_id.is_empty() {
        return Err("Server ID cannot be empty".to_string());
    }
    
    let manager = state.lock().await;
    println!("Current servers: {:?}", manager.servers.keys());

    if let Some(server) = manager.servers.get(&server_id) {
        println!("Found server: {}:{}, is running: {}", server.host, server.port, server.is_running());
        Ok(ServerInfo {
            server_id: server_id.clone(),
            host: server.host.clone(),
            port: server.port,
            client_count: server.get_client_count().await,
            is_running: server.is_running(),
        })
    } else {
        println!("Server with ID {} not found", server_id);
        Err(format!("Server with ID {} not found", server_id))
    }
}
