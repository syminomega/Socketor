using System.Text.Json.Serialization;

namespace Socketor.DataModel;

public class BackendMessageDto
{
    [JsonPropertyName("content")] public required string Message { get; set; }
    [JsonPropertyName("msg_type")] public required string MessageType { get; set; }

    public void HandleAsMessageItem(Action<MessageItemDto> handle)
    {
        var ownerType = MessageType switch
        {
            "Send" => OwnerType.Send,
            "Receive" => OwnerType.Receive,
            "Log" => OwnerType.Log,
            _ => throw new ArgumentException("Invalid message type")
        };
        var messageItem = new MessageItemDto(Message, ownerType);
        handle.Invoke(messageItem);
    }
}