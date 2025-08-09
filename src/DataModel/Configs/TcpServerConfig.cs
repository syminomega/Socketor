using Socketor.DataModel.Configs;

namespace Socketor.DataModel.Configs;

public class TcpServerConfig
{
    public string Host { get; set; } = "127.0.0.1";
    public int Port { get; set; } = 8081;
    public string ServerId { get; set; } = string.Empty;
    public MessageBoxConfig MessageBoxConfig { get; set; } = new();
    public SendBoxConfig SendBoxConfig { get; set; } = new();
}
