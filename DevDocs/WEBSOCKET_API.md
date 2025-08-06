# WebSocket Server API 使用说明

这个WebSocket服务器实现提供了以下功能：

## 主要功能

1. **启动WebSocket服务器** - 可以指定host和port
2. **停止WebSocket服务器** - 优雅关闭服务器和所有连接
3. **发送消息** - 支持发送给特定客户端或广播给所有客户端
4. **获取服务器状态** - 查看服务器列表和连接状态

## Tauri命令接口

### 1. 启动服务器
```rust
start_websocket_server(params: StartServerParams) -> Result<String, String>
```

参数：
```json
{
  "host": "127.0.0.1",
  "port": 8080,
  "server_id": "optional_custom_id"  // 可选，不提供则自动生成UUID
}
```

返回：服务器ID

### 2. 停止服务器
```rust
stop_websocket_server(server_id: String) -> Result<(), String>
```

### 3. 发送消息
```rust
send_websocket_message(params: SendMessageParams) -> Result<String, String>
```

参数：
```json
{
  "server_id": "server_id_here",
  "message": "Hello, WebSocket!",
  "target_client_id": null  // null表示广播，否则发送给特定客户端
}
```

### 4. 获取服务器列表
```rust
get_websocket_servers() -> Result<Vec<ServerInfo>, String>
```

### 5. 获取特定服务器信息
```rust
get_websocket_server_info(server_id: String) -> Result<ServerInfo, String>
```

## 使用流程示例

1. **启动服务器**：
   ```javascript
   const serverId = await invoke('start_websocket_server', {
     params: {
       host: '127.0.0.1',
       port: 8080
     }
   });
   ```

2. **客户端连接**：
   客户端可以通过 `ws://127.0.0.1:8080` 连接到服务器

3. **发送消息**：
   ```javascript
   // 广播消息
   await invoke('send_websocket_message', {
     params: {
       server_id: serverId,
       message: 'Hello everyone!',
       target_client_id: null
     }
   });
   ```

4. **停止服务器**：
   ```javascript
   await invoke('stop_websocket_server', { server_id: serverId });
   ```

## 特性

- ✅ 支持多个WebSocket服务器实例
- ✅ 自动客户端连接管理（连接/断开自动处理）
- ✅ 消息广播和点对点发送
- ✅ 优雅的服务器关闭
- ✅ 实时客户端连接数统计
- ✅ 唯一客户端ID生成（UUID）
- ✅ 异步非阻塞实现

## 依赖的Crate

- `tokio-tungstenite`: WebSocket实现
- `futures-util`: 异步流处理
- `uuid`: 生成唯一ID
- `serde`: 序列化/反序列化
- `tauri`: 与前端通信

