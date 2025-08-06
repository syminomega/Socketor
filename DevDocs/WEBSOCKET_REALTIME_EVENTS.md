# WebSocket Server 实时事件功能说明

我已经为你的WebSocket服务器添加了完整的实时事件功能，现在服务器可以将客户端的连接、断开和消息接收等事件实时显示到Blazor前端界面上。

## 🔥 新增功能

### 1. **Rust后端事件发送**
在 `websocket_server.rs` 中添加了以下事件类型：

#### **事件类型**
- `client_connected` - 客户端连接事件
- `client_disconnected` - 客户端断开事件  
- `message_received` - 接收到文本消息事件
- `binary_received` - 接收到二进制数据事件

#### **事件数据结构**
```rust
pub struct WebSocketServerEvent {
    pub server_id: String,     // 服务器ID
    pub event_type: String,    // 事件类型
    pub client_id: String,     // 客户端ID
    pub message: String,       // 消息内容
    pub timestamp: String,     // 时间戳
}
```

### 2. **Blazor前端事件监听**
在 `WebSocketServerComp.razor` 中添加了：

#### **实时事件处理**
- ✅ 监听 `websocket-server-event` 事件
- ✅ 过滤当前服务器的事件（多服务器支持）
- ✅ 根据事件类型显示不同样式的消息
- ✅ 自动刷新客户端连接数

#### **消息显示逻辑**
```csharp
var messageOwner = serverEvent.EventType switch
{
    "client_connected" => MessageOwner.Info,      // 蓝色信息消息
    "client_disconnected" => MessageOwner.Info,   // 蓝色信息消息
    "message_received" => MessageOwner.Receive,   // 绿色接收消息
    "binary_received" => MessageOwner.Receive,    // 绿色接收消息
    _ => MessageOwner.Info
};
```

## 📱 **用户体验**

### **实时显示效果**
1. **客户端连接时**：
   ```
   [client-uuid-123] Client connected from 127.0.0.1:54321
   ```

2. **接收消息时**：
   ```
   [client-uuid-123] Hello from client!
   ```

3. **接收二进制数据时**：
   ```
   [client-uuid-123] Binary data: 1024 bytes
   ```

4. **客户端断开时**：
   ```
   [client-uuid-123] Client disconnected
   ```

### **自动更新**
- ✅ 客户端连接/断开时自动刷新连接数
- ✅ 所有事件实时显示在MessageBox中
- ✅ 支持消息筛选和格式化

## 🔧 **技术实现细节**

### **后端事件发送**
```rust
// 在handle_connection函数中
if let Some(ref app) = app_handle {
    let event = WebSocketServerEvent {
        server_id: server_id.clone(),
        event_type: "message_received".to_string(),
        client_id: client_id.clone(),
        message: text.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    if let Err(e) = app.emit("websocket-server-event", &event) {
        eprintln!("Failed to emit event to frontend: {}", e);
    }
}
```

### **前端事件监听**
```csharp
// 在OnAfterRenderAsync中
_unlistenServerMessage = await Tauri.Event.Listen<WebSocketServerEvent>(
    "websocket-server-event", 
    OnServerEventReceived
);
```

## 🎯 **使用流程**

1. **启动服务器** - 开始监听WebSocket连接
2. **客户端连接** - 自动显示连接事件和更新计数
3. **接收消息** - 实时显示客户端发送的消息
4. **客户端断开** - 自动显示断开事件和更新计数

## 📦 **新增依赖**

在 `Cargo.toml` 中添加了：
```toml
chrono = { version = "0.4", features = ["serde"] }
```

用于生成ISO 8601格式的时间戳。

## 🎉 **完整功能列表**


- ✅ 启动/停止WebSocket服务器
- ✅ 广播消息给所有客户端
- ✅ 实时显示客户端连接状态
- ✅ 实时显示接收到的消息
- ✅ 自动更新客户端连接数
- ✅ 多服务器支持（每个服务器独立事件）
- ✅ 完整的错误处理和用户反馈
- ✅ 配置持久化

