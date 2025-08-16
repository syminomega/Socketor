use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tauri::{State, Emitter};
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex, broadcast};
use tokio::task::JoinHandle;
use uuid::Uuid;
use chrono;
use std::net::SocketAddr;

// UDP客户端连接状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UdpClientState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

// UDP客户端
pub struct UdpClient {
    pub local_port: Option<u16>,   // 本地绑定端口，None表示系统自动分配
    pub actual_port: u16,          // 实际绑定的端口
    pub client_id: String,
    pub state: UdpClientState,
    pub socket: Option<UdpSocket>,
    pub receive_handle: Option<JoinHandle<()>>,
    pub send_handle: Option<JoinHandle<()>>,
    pub shutdown_sender: Option<broadcast::Sender<()>>,
    pub message_sender: Option<mpsc::UnboundedSender<(Vec<u8>, SocketAddr)>>,
    pub app_handle: Option<tauri::AppHandle>,
}

// UDP客户端管理器
pub struct UdpClientManager {
    pub clients: HashMap<String, UdpClient>,
}

impl UdpClientManager {
    pub fn new() -> Self {
        UdpClientManager {
            clients: HashMap::new(),
        }
    }
}

impl Default for UdpClientManager {
    fn default() -> Self {
        Self::new()
    }
}

// 启动UDP客户端的参数
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartUdpClientParams {
    pub local_port: Option<u16>, // 本地绑定端口，None表示系统自动分配
    pub client_id: Option<String>,
}

// 发送消息的参数
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendUdpClientMessageParams {
    pub client_id: String,
    pub target_host: String,  // 目标主机地址
    pub target_port: u16,     // 目标端口
    pub message: String,
    pub message_type: Option<String>, // "text" 或 "hex"，默认为 "text"
}

// 客户端状态信息
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UdpClientInfo {
    pub client_id: String,
    pub local_port: u16,      // 本地绑定端口
    pub state: UdpClientState,
}

// UDP客户端事件数据（发送给前端）
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UdpClientEvent {
    pub client_id: String,
    pub event_type: String,
    pub message: String,
    pub timestamp: String,
}

impl UdpClient {
    pub fn new(local_port: Option<u16>, client_id: String) -> Self {
        UdpClient {
            local_port,
            actual_port: 0,
            client_id,
            state: UdpClientState::Disconnected,
            socket: None,
            receive_handle: None,
            send_handle: None,
            shutdown_sender: None,
            message_sender: None,
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, app_handle: tauri::AppHandle) {
        self.app_handle = Some(app_handle);
    }

    pub async fn start(&mut self) -> Result<(), String> {
        if self.state == UdpClientState::Connected {
            return Err("Already started".to_string());
        }

        self.state = UdpClientState::Connecting;
        
        // 绑定到本地端口，如果没有指定端口则使用0让系统自动分配
        let bind_addr = format!("0.0.0.0:{}", self.local_port.unwrap_or(0));
        
        match UdpSocket::bind(&bind_addr).await {
            Ok(socket) => {
                // 获取实际绑定的端口
                self.actual_port = socket.local_addr()
                    .map_err(|e| format!("Failed to get local address: {}", e))?
                    .port();
                
                self.socket = Some(socket);
                self.state = UdpClientState::Connected;
                
                // 发送启动成功事件
                if let Some(app_handle) = &self.app_handle {
                    let event = UdpClientEvent {
                        client_id: self.client_id.clone(),
                        event_type: "connected".to_string(),
                        message: format!("UDP client started on port {}", self.actual_port),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };
                    let _ = app_handle.emit("udp-client-event", &event);
                }

                // 启动接收和发送任务
                self.start_tasks().await?;
                Ok(())
            }
            Err(e) => {
                self.state = UdpClientState::Error;
                Err(format!("Failed to bind UDP socket to {}: {}", bind_addr, e))
            }
        }
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        if self.state != UdpClientState::Connected {
            return Ok(());
        }

        // 发送关闭信号
        if let Some(shutdown_sender) = &self.shutdown_sender {
            let _ = shutdown_sender.send(());
        }

        // 等待任务完成
        if let Some(receive_handle) = self.receive_handle.take() {
            let _ = receive_handle.await;
        }
        if let Some(send_handle) = self.send_handle.take() {
            let _ = send_handle.await;
        }

        // 关闭连接
        self.socket = None;
        self.shutdown_sender = None;
        self.message_sender = None;
        self.state = UdpClientState::Disconnected;

        // 发送断开连接事件
        if let Some(app_handle) = &self.app_handle {
            let event = UdpClientEvent {
                client_id: self.client_id.clone(),
                event_type: "disconnected".to_string(),
                message: "UDP client stopped".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            let _ = app_handle.emit("udp-client-event", &event);
        }

        Ok(())
    }

    async fn start_tasks(&mut self) -> Result<(), String> {
        let socket = self.socket.take().ok_or("No socket available")?;
        let socket = std::sync::Arc::new(socket);
        let socket_send = socket.clone();
        let socket_recv = socket.clone();

        let (shutdown_tx, _shutdown_rx) = broadcast::channel(1);
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        
        self.shutdown_sender = Some(shutdown_tx.clone());
        self.message_sender = Some(message_tx);

        // 启动接收任务
        let client_id = self.client_id.clone();
        let app_handle = self.app_handle.clone();
        let shutdown_rx_clone = shutdown_tx.subscribe();
        self.receive_handle = Some(tokio::spawn(async move {
            handle_udp_client_receive(socket_recv, client_id, app_handle, shutdown_rx_clone).await;
        }));

        // 启动发送任务
        let shutdown_rx_clone = shutdown_tx.subscribe();
        self.send_handle = Some(tokio::spawn(async move {
            handle_udp_client_send(socket_send, message_rx, shutdown_rx_clone).await;
        }));

        // 将socket放回，但实际上已经被Arc包装了
        self.socket = None; // UDP不需要持续持有socket引用，任务中的Arc会处理

        Ok(())
    }

    pub async fn send_message(&self, message: Vec<u8>, target_addr: SocketAddr) -> Result<(), String> {
        if self.state != UdpClientState::Connected {
            return Err("Not started".to_string());
        }

        if let Some(sender) = &self.message_sender {
            sender.send((message, target_addr)).map_err(|e| format!("Failed to send message: {}", e))?;
            Ok(())
        } else {
            Err("Message sender not available".to_string())
        }
    }
}

// 处理UDP客户端接收消息
async fn handle_udp_client_receive(
    socket: std::sync::Arc<UdpSocket>,
    client_id: String,
    app_handle: Option<tauri::AppHandle>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let mut buffer = vec![0; 1024];
    
    loop {
        tokio::select! {
            // 检查是否收到关闭信号
            _ = shutdown_rx.recv() => {
                break;
            }
            // 读取数据
            result = socket.recv_from(&mut buffer) => {
                match result {
                    Ok((n, from_addr)) => {
                        let received_data = buffer[..n].to_vec();
                        
                        // 发送接收到的消息事件
                        if let Some(app_handle) = &app_handle {
                            let message = String::from_utf8_lossy(&received_data).to_string();
                            let event = UdpClientEvent {
                                client_id: client_id.clone(),
                                event_type: "message_received".to_string(),
                                message: format!("From {}: {}", from_addr, message),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };
                            let _ = app_handle.emit("udp-client-event", &event);
                        }
                    }
                    Err(e) => {
                        if let Some(app_handle) = &app_handle {
                            let event = UdpClientEvent {
                                client_id: client_id.clone(),
                                event_type: "error".to_string(),
                                message: format!("Read error: {}", e),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };
                            let _ = app_handle.emit("udp-client-event", &event);
                        }
                        break;
                    }
                }
            }
        }
    }
}

// 处理UDP客户端发送消息
async fn handle_udp_client_send(
    socket: std::sync::Arc<UdpSocket>,
    mut message_rx: mpsc::UnboundedReceiver<(Vec<u8>, SocketAddr)>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    loop {
        tokio::select! {
            // 检查是否收到关闭信号
            _ = shutdown_rx.recv() => {
                break;
            }
            // 发送消息
            message = message_rx.recv() => {
                match message {
                    Some((data, addr)) => {
                        if let Err(e) = socket.send_to(&data, addr).await {
                            eprintln!("Failed to send UDP data: {}", e);
                            break;
                        }
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    }
}

// Tauri命令：启动UDP客户端
#[tauri::command]
pub async fn start_udp_client(
    start_params: StartUdpClientParams,
    manager: State<'_, Mutex<UdpClientManager>>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let client_id = start_params.client_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let mut client = UdpClient::new(start_params.local_port, client_id.clone());
    client.set_app_handle(app_handle);
    
    client.start().await?;
    
    let mut manager = manager.lock().await;
    manager.clients.insert(client_id.clone(), client);
    
    Ok(client_id)
}

// Tauri命令：停止UDP客户端
#[tauri::command]
pub async fn stop_udp_client(
    client_id: String,
    manager: State<'_, Mutex<UdpClientManager>>,
) -> Result<(), String> {
    let mut manager = manager.lock().await;
    
    if let Some(client) = manager.clients.get_mut(&client_id) {
        client.stop().await?;
        manager.clients.remove(&client_id);
        Ok(())
    } else {
        Err(format!("UDP client {} not found", client_id))
    }
}

// Tauri命令：发送UDP消息
#[tauri::command]
pub async fn send_udp_client_message(
    send_params: SendUdpClientMessageParams,
    manager: State<'_, Mutex<UdpClientManager>>,
) -> Result<(), String> {
    let message_type = send_params.message_type.as_deref().unwrap_or("text");
    let data = match message_type {
        "hex" => {
            // 将十六进制字符串转换为字节
            let hex_str = send_params.message.replace(" ", "");
            hex::decode(&hex_str).map_err(|e| format!("Invalid hex string: {}", e))?
        }
        _ => send_params.message.into_bytes(),
    };

    // 解析目标地址
    let target_addr = format!("{}:{}", send_params.target_host, send_params.target_port);
    let target_sockaddr: SocketAddr = target_addr.parse()
        .map_err(|e| format!("Invalid target address {}: {}", target_addr, e))?;

    let manager = manager.lock().await;
    if let Some(client) = manager.clients.get(&send_params.client_id) {
        client.send_message(data, target_sockaddr).await?;
        Ok(())
    } else {
        Err(format!("UDP client {} not found", send_params.client_id))
    }
}

// Tauri命令：获取所有UDP客户端
#[tauri::command]
pub async fn get_udp_clients(
    manager: State<'_, Mutex<UdpClientManager>>,
) -> Result<Vec<UdpClientInfo>, String> {
    let manager = manager.lock().await;
    let clients: Vec<UdpClientInfo> = manager
        .clients
        .values()
        .map(|client| UdpClientInfo {
            client_id: client.client_id.clone(),
            local_port: client.actual_port,
            state: client.state.clone(),
        })
        .collect();
    Ok(clients)
}

// Tauri命令：获取UDP客户端信息
#[tauri::command]
pub async fn get_udp_client_info(
    client_id: String,
    manager: State<'_, Mutex<UdpClientManager>>,
) -> Result<UdpClientInfo, String> {
    let manager = manager.lock().await;
    if let Some(client) = manager.clients.get(&client_id) {
        Ok(UdpClientInfo {
            client_id: client.client_id.clone(),
            local_port: client.actual_port,
            state: client.state.clone(),
        })
    } else {
        Err(format!("UDP client {} not found", client_id))
    }
}
