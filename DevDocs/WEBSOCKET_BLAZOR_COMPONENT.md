# WebSocket Server Blazor 组件使用说明


## 🔧 与Tauri后端的交互

>注意：Rust 默认会使用 snake_case 命名风格，而 JavaScript 使用 camelCase。为了保持一致性，Tauri 命令和参数在 JavaScript 中使用 camelCase。

组件使用以下Tauri命令与Rust后端通信：

### 启动服务器
```javascript
await Tauri.Core.Invoke<string>("start_websocket_server", {
  params: {
    Host: "127.0.0.1",
    Port: 8080,
    ServerId: null // 可选，为null时自动生成
  }
});
```

### 停止服务器
```javascript
await Tauri.Core.Invoke("stop_websocket_server", {
  server_id: "服务器ID"
});
```

### 发送消息
```javascript
await Tauri.Core.Invoke<string>("send_websocket_message", {
  params: {
    ServerId: "服务器ID",
    Message: "消息内容",
    TargetClientId: null // null表示广播
  }
});
```

### 获取服务器信息
```javascript
await Tauri.Core.Invoke<ServerInfoResponse>("get_websocket_server_info", {
  server_id: "服务器ID"
});
```

## 📁 相关文件

- **组件文件**: `src/Components/WebSocketServerComp.razor`
- **样式文件**: `src/Components/WebSocketServerComp.razor.css`
- **配置类**: `src/DataModel/Configs/WebSocketServerConfig.cs`
- **本地化**: `src/Locales/en-US.json` 和 `src/Locales/zh-CN.json`


## 🔄 生命周期

- **组件初始化**：从LocalStorage加载保存的配置
- **组件销毁**：自动保存当前配置到LocalStorage
- **配置变更**：每次启动服务器前自动保存配置

组件已经完全实现并与你现有的Rust后端WebSocket服务器无缝集成！
