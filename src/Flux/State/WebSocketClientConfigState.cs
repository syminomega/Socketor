using Fluxor;
using Socketor.DataModel.Configs;

namespace Socketor.Flux.State;

[FeatureState]
public class WebSocketClientConfigState(WebSocketClientConfig config)
{
    private WebSocketClientConfigState() : this(new WebSocketClientConfig())
    {
    }

    public WebSocketClientConfig Config { get; } = config;
}