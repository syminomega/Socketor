# Socketor
一个简单的网络调试工具 \
 Simple tool for WebSocket, TCP/UDP testing.

 >但是，这是一个非常奇葩的项目！用 .net 开发网页，用 rust 开发后端逻辑，然后强行用胶水粘在了一起，纯整活+验证技术。要用的话下载我编译好的程序就行了XD，别试图理解这个项目。

![Web Socket Tool Preview](/preview/ws_preview.png)

## Requirement
* Rust 1.63 +
* Tauri 1.0 +
* .Net 6 + with Blazor WebAssembly Tool
* WebView2 on Windows / webkit2gtk on Linux

## How To Build
Run 'cargo tauri build' under the main directory to pack the executable file.

## Utilities
|           |           |
|  ----  | ----  |
| WebSocket Client  | Finished |
| WebSocket Server  | Plan |
| TCP Client|  In Progress |
| TCP Server|  In Progress |
| UDP Client|  Plan |