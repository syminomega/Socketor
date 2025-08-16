namespace Socketor.DataModel.Configs;

public class UdpClientConfig : IConnectionConfig
{
    public string ConnectionType { get; set; } = nameof(UdpClientConfig);
    public int? LocalPort { get; set; } = null; // null表示系统自动分配端口
    public string TargetHost { get; set; } = "127.0.0.1"; // 目标主机
    public int TargetPort { get; set; } = 8082; // 目标端口
    public MessageBoxConfig MessageBoxConfig { get; set; } = new();
    public SendBoxConfig SendBoxConfig { get; set; } = new();
}
