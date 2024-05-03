namespace Socketor.DataModel;

public class MessageItemDto(string content, OwnerType ownerType)
{
    public DateTime Time { get; } = DateTime.Now;
    public string Content { get; } = content;
    public OwnerType OwnerType { get; } = ownerType;
}

public enum OwnerType
{
    Receive,
    Send,
    Log
}