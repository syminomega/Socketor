namespace Socketor.DataModel.Configs;

public class WebSocketClientConfig : IConnectionConfig
{
    public string ConnectionType { get; set; } = nameof(WebSocketClientConfig);
    public string WebSocketAddress { get; set; } = "ws://127.0.0.1:8080";
    public MessageBoxConfig MessageBoxConfig { get; set; } = new();
    public SendBoxConfig SendBoxConfig { get; set; } = new();

}