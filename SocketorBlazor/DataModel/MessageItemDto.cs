namespace Socketor.DataModel;

public class MessageItemDto
{
    public MessageItemDto(string content, OwnerType ownerType)
    {
        Time = DateTime.Now;
        Content = content;
        OwnerType = ownerType;
    }
    public DateTime Time { get; }
    public string Content { get; }
    public OwnerType OwnerType { get; }
}

public enum OwnerType
{
    Receive,
    Send,
    Log
}