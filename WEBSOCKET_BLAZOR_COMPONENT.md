# WebSocket Server Blazor 组件使用说明

这个 `WebSocketServerComp` 组件提供了一个完整的WebSocket服务器管理界面，参考了现有的 `WebSocketClientComp` 的布局和设计。

## 🎯 主要功能

### 1. **服务器管理**
- ✅ 启动/停止WebSocket服务器
- ✅ 配置主机地址和端口
- ✅ 显示服务器运行状态
- ✅ 自动生成和管理服务器ID

### 2. **客户端监控**
- ✅ 实时显示连接的客户端数量
- ✅ 刷新服务器状态和客户端信息

### 3. **消息处理**
- ✅ 广播消息给所有连接的客户端
- ✅ 在MessageBox中显示发送的消息和结果
- ✅ 错误处理和状态反馈

### 4. **配置管理**
- ✅ 本地存储服务器配置
- ✅ 组件销毁时自动保存配置

## 🎨 界面布局

组件采用了与WebSocketClient相同的布局模式：

```
┌─────────────────────────────┬─────────────────┐
│                             │                 │
│        Message Box          │  Server         │
│                             │  Settings       │
│                             │                 │
├─────────────────────────────┤                 │
│                             │                 │
│        Send Box             │                 │
│                             │                 │
└─────────────────────────────┴─────────────────┘
```

### 左侧面板：
- **上半部分**：服务器地址显示、主机/端口配置、启动/停止按钮、消息显示区
- **下半部分**：消息发送区

### 右侧面板：
- 服务器设置卡片（服务器状态、客户端数量、服务器ID）
- MessageBox和SendBox的配置区域

## 🔧 与Tauri后端的交互

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

## 🚀 使用方法

1. **在页面中引用组件**：
   ```html
   <WebSocketServerComp />
   ```

2. **启动服务器**：
   - 配置主机地址（默认：127.0.0.1）
   - 配置端口（默认：8080）
   - 点击"启动"按钮

3. **发送消息**：
   - 在SendBox中输入消息
   - 点击发送按钮广播给所有客户端

4. **监控状态**：
   - 使用刷新按钮更新服务器状态
   - 查看连接的客户端数量

## 🔄 生命周期

- **组件初始化**：从LocalStorage加载保存的配置
- **组件销毁**：自动保存当前配置到LocalStorage
- **配置变更**：每次启动服务器前自动保存配置

组件已经完全实现并与你现有的Rust后端WebSocket服务器无缝集成！
