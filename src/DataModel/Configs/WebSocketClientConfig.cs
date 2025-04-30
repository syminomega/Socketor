namespace Socketor.DataModel.Configs;

public class WebSocketClientConfig : IConnectionConfig
{
    public string ConnectionType { get; set; } = nameof(WebSocketClientConfig);
    public string WebSocketAddress { get; set; } = "ws://localhost:5000/chat";
    public MessageBoxConfig MessageBoxConfig { get; set; } = new();
    public SendBoxConfig SendBoxConfig { get; set; } = new();

}