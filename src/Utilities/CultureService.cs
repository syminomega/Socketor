using System.Globalization;
using Microsoft.AspNetCore.Components.WebAssembly.Hosting;
using Microsoft.JSInterop;

namespace Socketor.Utilities;

public class CultureService(IJSRuntime jsRuntime)
{
    private readonly Lazy<Task<IJSObjectReference>> _cultureStorageJs = new(jsRuntime
        .InvokeAsync<IJSObjectReference>("import", "./js/Utilities/CultureService.js").AsTask);

    public async Task<string?> GetCurrentCulture()
    {
        var module = await _cultureStorageJs.Value;
        return await module.InvokeAsync<string?>("getStoredCulture");
    }

    public async Task SetCurrentCulture(string cultureName)
    {
        var module = await _cultureStorageJs.Value;
        await module.InvokeVoidAsync("setStoredCulture", cultureName);
        ChangeCulture(cultureName);
    }

    public void ChangeCulture(string cultureName)
    {
        var culture = new CultureInfo(cultureName);
        CultureInfo.DefaultThreadCurrentCulture = culture;
        CultureInfo.DefaultThreadCurrentUICulture = culture;
    }
}

public static class CultureServiceExtensions
{
    public static void AddCultureService(this IServiceCollection services)
    {
        services.AddScoped<CultureService>();
    }

    public static async Task InitCultureAsync(this WebAssemblyHost host)
    {
        var cultureService = host.Services.GetRequiredService<CultureService>();
        var cultureName = await cultureService.GetCurrentCulture() ?? "en-US";
        cultureService.ChangeCulture(cultureName);
    }
}