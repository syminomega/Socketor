using System.Text.Json.Serialization;

namespace Socketor.DataModel;

public class ActionResultDto
{
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    [JsonPropertyName("error_message")]
    public string ErrorMessage { get; set; } = "";
}