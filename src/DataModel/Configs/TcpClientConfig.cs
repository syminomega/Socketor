namespace Socketor.DataModel.Configs;

public class TcpClientConfig : IConnectionConfig
{
    public string ConnectionType { get; set; } = nameof(TcpClientConfig);
    public string Host { get; set; } = "127.0.0.1";
    public int Port { get; set; } = 8081;
    public MessageBoxConfig MessageBoxConfig { get; set; } = new();
    public SendBoxConfig SendBoxConfig { get; set; } = new();
}
