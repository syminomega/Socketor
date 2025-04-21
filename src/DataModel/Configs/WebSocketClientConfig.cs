namespace Socketor.DataModel.Configs;

public class WebSocketClientConfig
{
    public string WebSocketAddress { get; set; } = "ws://localhost:5000/chat";
    public MessageBoxConfig MessageBoxConfig { get; set; } = new();
    public SendBoxConfig SendBoxConfig { get; set; } = new();
}