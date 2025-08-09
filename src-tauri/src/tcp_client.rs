use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tauri::{State, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex, broadcast};
use tokio::task::JoinHandle;
use uuid::Uuid;
use chrono;

// TCP客户端连接状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TcpClientState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

// TCP客户端
pub struct TcpClient {
    pub host: String,
    pub port: u16,
    pub client_id: String,
    pub state: TcpClientState,
    pub stream: Option<TcpStream>,
    pub receive_handle: Option<JoinHandle<()>>,
    pub send_handle: Option<JoinHandle<()>>,
    pub shutdown_sender: Option<broadcast::Sender<()>>,
    pub message_sender: Option<mpsc::UnboundedSender<Vec<u8>>>,
    pub app_handle: Option<tauri::AppHandle>,
}

// TCP客户端管理器
pub struct TcpClientManager {
    pub clients: HashMap<String, TcpClient>,
}

impl TcpClientManager {
    pub fn new() -> Self {
        TcpClientManager {
            clients: HashMap::new(),
        }
    }
}

impl Default for TcpClientManager {
    fn default() -> Self {
        Self::new()
    }
}

// 连接TCP服务器的参数
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectTcpClientParams {
    pub host: String,
    pub port: u16,
    pub client_id: Option<String>,
}

// 发送消息的参数
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendTcpClientMessageParams {
    pub client_id: String,
    pub message: String,
    pub message_type: Option<String>, // "text" 或 "hex"，默认为 "text"
}

// 客户端状态信息
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcpClientInfo {
    pub client_id: String,
    pub host: String,
    pub port: u16,
    pub state: TcpClientState,
}

// TCP客户端事件数据（发送给前端）
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TcpClientEvent {
    pub client_id: String,
    pub event_type: String,
    pub message: String,
    pub timestamp: String,
}

impl TcpClient {
    pub fn new(host: String, port: u16, client_id: String) -> Self {
        TcpClient {
            host,
            port,
            client_id,
            state: TcpClientState::Disconnected,
            stream: None,
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

    pub async fn connect(&mut self) -> Result<(), String> {
        if self.state == TcpClientState::Connected {
            return Err("Already connected".to_string());
        }

        self.state = TcpClientState::Connecting;
        let addr = format!("{}:{}", self.host, self.port);
        
        match TcpStream::connect(&addr).await {
            Ok(stream) => {
                self.stream = Some(stream);
                self.state = TcpClientState::Connected;
                
                // 发送连接成功事件
                if let Some(app_handle) = &self.app_handle {
                    let event = TcpClientEvent {
                        client_id: self.client_id.clone(),
                        event_type: "connected".to_string(),
                        message: format!("Connected to {}", addr),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };
                    let _ = app_handle.emit("tcp-client-event", &event);
                }

                // 启动接收和发送任务
                self.start_tasks().await?;
                Ok(())
            }
            Err(e) => {
                self.state = TcpClientState::Error;
                Err(format!("Failed to connect to {}: {}", addr, e))
            }
        }
    }

    pub async fn disconnect(&mut self) -> Result<(), String> {
        if self.state != TcpClientState::Connected {
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
        self.stream = None;
        self.shutdown_sender = None;
        self.message_sender = None;
        self.state = TcpClientState::Disconnected;

        // 发送断开连接事件
        if let Some(app_handle) = &self.app_handle {
            let event = TcpClientEvent {
                client_id: self.client_id.clone(),
                event_type: "disconnected".to_string(),
                message: "Disconnected from server".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            let _ = app_handle.emit("tcp-client-event", &event);
        }

        Ok(())
    }

    async fn start_tasks(&mut self) -> Result<(), String> {
        let stream = self.stream.take().ok_or("No stream available")?;
        let (read_stream, write_stream) = stream.into_split();

        let (shutdown_tx, _shutdown_rx) = broadcast::channel(1);
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        
        self.shutdown_sender = Some(shutdown_tx.clone());
        self.message_sender = Some(message_tx);

        // 启动接收任务
        let client_id = self.client_id.clone();
        let app_handle = self.app_handle.clone();
        let shutdown_rx_clone = shutdown_tx.subscribe();
        self.receive_handle = Some(tokio::spawn(async move {
            handle_tcp_client_receive(read_stream, client_id, app_handle, shutdown_rx_clone).await;
        }));

        // 启动发送任务
        let shutdown_rx_clone = shutdown_tx.subscribe();
        self.send_handle = Some(tokio::spawn(async move {
            handle_tcp_client_send(write_stream, message_rx, shutdown_rx_clone).await;
        }));

        Ok(())
    }

    pub async fn send_message(&self, message: Vec<u8>) -> Result<(), String> {
        if self.state != TcpClientState::Connected {
            return Err("Not connected".to_string());
        }

        if let Some(sender) = &self.message_sender {
            sender.send(message).map_err(|e| format!("Failed to send message: {}", e))?;
            Ok(())
        } else {
            Err("Message sender not available".to_string())
        }
    }
}

// 处理TCP客户端接收消息
async fn handle_tcp_client_receive(
    mut read_stream: tokio::net::tcp::OwnedReadHalf,
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
            result = read_stream.read(&mut buffer) => {
                match result {
                    Ok(0) => {
                        // 连接已关闭
                        if let Some(app_handle) = &app_handle {
                            let event = TcpClientEvent {
                                client_id: client_id.clone(),
                                event_type: "disconnected".to_string(),
                                message: "Connection closed by server".to_string(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };
                            let _ = app_handle.emit("tcp-client-event", &event);
                        }
                        break;
                    }
                    Ok(n) => {
                        let received_data = buffer[..n].to_vec();
                        
                        // 发送接收到的消息事件
                        if let Some(app_handle) = &app_handle {
                            let message = String::from_utf8_lossy(&received_data).to_string();
                            let event = TcpClientEvent {
                                client_id: client_id.clone(),
                                event_type: "message_received".to_string(),
                                message,
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };
                            let _ = app_handle.emit("tcp-client-event", &event);
                        }
                    }
                    Err(e) => {
                        if let Some(app_handle) = &app_handle {
                            let event = TcpClientEvent {
                                client_id: client_id.clone(),
                                event_type: "error".to_string(),
                                message: format!("Read error: {}", e),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };
                            let _ = app_handle.emit("tcp-client-event", &event);
                        }
                        break;
                    }
                }
            }
        }
    }
}

// 处理TCP客户端发送消息
async fn handle_tcp_client_send(
    mut write_stream: tokio::net::tcp::OwnedWriteHalf,
    mut message_rx: mpsc::UnboundedReceiver<Vec<u8>>,
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
                    Some(data) => {
                        if let Err(e) = write_stream.write_all(&data).await {
                            eprintln!("Failed to write data: {}", e);
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

// Tauri命令：连接TCP服务器
#[tauri::command]
pub async fn connect_tcp_client(
    params: ConnectTcpClientParams,
    manager: State<'_, Mutex<TcpClientManager>>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let client_id = params.client_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let mut client = TcpClient::new(params.host, params.port, client_id.clone());
    client.set_app_handle(app_handle);
    
    client.connect().await?;
    
    let mut manager = manager.lock().await;
    manager.clients.insert(client_id.clone(), client);
    
    Ok(client_id)
}

// Tauri命令：断开TCP客户端
#[tauri::command]
pub async fn disconnect_tcp_client(
    client_id: String,
    manager: State<'_, Mutex<TcpClientManager>>,
) -> Result<(), String> {
    let mut manager = manager.lock().await;
    
    if let Some(client) = manager.clients.get_mut(&client_id) {
        client.disconnect().await?;
        manager.clients.remove(&client_id);
        Ok(())
    } else {
        Err(format!("TCP client {} not found", client_id))
    }
}

// Tauri命令：发送TCP消息
#[tauri::command]
pub async fn send_tcp_client_message(
    params: SendTcpClientMessageParams,
    manager: State<'_, Mutex<TcpClientManager>>,
) -> Result<(), String> {
    let message_type = params.message_type.as_deref().unwrap_or("text");
    let data = match message_type {
        "hex" => {
            // 将十六进制字符串转换为字节
            let hex_str = params.message.replace(" ", "");
            hex::decode(&hex_str).map_err(|e| format!("Invalid hex string: {}", e))?
        }
        _ => params.message.into_bytes(),
    };

    let manager = manager.lock().await;
    if let Some(client) = manager.clients.get(&params.client_id) {
        client.send_message(data).await?;
        Ok(())
    } else {
        Err(format!("TCP client {} not found", params.client_id))
    }
}

// Tauri命令：获取所有TCP客户端
#[tauri::command]
pub async fn get_tcp_clients(
    manager: State<'_, Mutex<TcpClientManager>>,
) -> Result<Vec<TcpClientInfo>, String> {
    let manager = manager.lock().await;
    let clients: Vec<TcpClientInfo> = manager
        .clients
        .values()
        .map(|client| TcpClientInfo {
            client_id: client.client_id.clone(),
            host: client.host.clone(),
            port: client.port,
            state: client.state.clone(),
        })
        .collect();
    Ok(clients)
}

// Tauri命令：获取TCP客户端信息
#[tauri::command]
pub async fn get_tcp_client_info(
    client_id: String,
    manager: State<'_, Mutex<TcpClientManager>>,
) -> Result<TcpClientInfo, String> {
    let manager = manager.lock().await;
    if let Some(client) = manager.clients.get(&client_id) {
        Ok(TcpClientInfo {
            client_id: client.client_id.clone(),
            host: client.host.clone(),
            port: client.port,
            state: client.state.clone(),
        })
    } else {
        Err(format!("TCP client {} not found", client_id))
    }
}
