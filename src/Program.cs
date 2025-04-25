using Blazored.LocalStorage;
using BootstrapBlazor.Components;
using Fluxor;
using Microsoft.AspNetCore.Components.Web;
using Microsoft.AspNetCore.Components.WebAssembly.Hosting;
using Socketor;
using Socketor.Utilities;
using TauriApi;

var builder = WebAssemblyHostBuilder.CreateDefault(args);
builder.RootComponents.Add<App>("#app");
builder.RootComponents.Add<HeadOutlet>("head::after");

var httpClient = new HttpClient { BaseAddress = new Uri(builder.HostEnvironment.BaseAddress) };
builder.Services.AddScoped(_ => httpClient);
builder.Services.AddTauriApi();
builder.Services.AddBootstrapBlazor();
// 增加本地化
builder.Services.AddCultureService();
List<string> supportedCultures = ["zh-CN", "en-US"];
builder.Services.Configure<BootstrapBlazorOptions>(op =>
{
    op.ToastDelay = 4000;
    op.SupportedCultures = supportedCultures;
});

builder.Services.AddBlazoredLocalStorageAsSingleton();
builder.Services.AddFluxor(options => options.ScanAssemblies(typeof(Program).Assembly));

var host = builder.Build();
await host.InitCultureAsync();
await host.RunAsync();