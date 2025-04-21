using System.ComponentModel;

namespace Socketor.DataModel;

public class MessageData(MessageOwner messageOwner, byte[] data, MessageType messageType= MessageType.Text)
{
    public DateTime Time { get; init; } = DateTime.Now;
    public MessageOwner MessageOwner { get; init; } = messageOwner;
    public MessageType MessageType { get; set; } = messageType;
    public byte[] RawMessage { get; set; } = data;
}

public enum MessageOwner
{
    [Description("receive")]
    Receive,
    [Description("send")]
    Send,
    [Description("info")]
    Info,
    [Description("error")]
    Error,
    [Description("warning")]
    Warning,
}

public enum MessageType
{
    Text,
    Binary,
    Image,
    Video,
    Audio,
    File,
}