# Socketor
一个简单的网络调试工具，当前支持WebSocket，更多功能开发中。 \
 Simple tool for WebSocket, TCP/UDP testing.

 >这曾经是一个非常奇葩的项目，但现在好好休整了一下，接下来会持续优化。

![Web Socket Tool Preview](/preview/ws_preview.png)

## Requirement
* Tauri 2.0 + Rust edition 2021
* .Net 9 + with Blazor WebAssembly Tool
* WebView2 *(Windows)*
* webkit2gtk *(Linux)*

## Development
Run `dotnet watch run --project src/Socketor.csproj`
Run `cargo tauri dev` under the main directory to start the project.

## How To Build
Run `cargo tauri build` under the main directory to pack the executable file.

## Utilities
|                  |             |
|------------------|-------------|
| WebSocket Client | Finished    |
| WebSocket Server | Plan        |
| TCP Client       | In Progress |
| TCP Server       | In Progress |
| UDP Client       | Plan        |