using Microsoft.AspNetCore.Components;
using Socketor.DataModel;

namespace Socketor.Utilities;

public interface IMessageBackend
{
    public RenderFragment PropertyArea { get; }
    public Task SendMessageAsync(string message);
    public event Action<MessageItemDto> ShowMessage;

}