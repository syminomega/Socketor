using Blazorise;
using Blazorise.AntDesign;
using Blazorise.Icons.FontAwesome;
using Microsoft.AspNetCore.Components.Web;
using Microsoft.AspNetCore.Components.WebAssembly.Hosting;
using Socketor;

var builder = WebAssemblyHostBuilder.CreateDefault(args);
builder.RootComponents.Add<App>("#app");
builder.RootComponents.Add<HeadOutlet>("head::after");

builder.Services.AddScoped(sp => new HttpClient
{
    BaseAddress = new Uri(builder.HostEnvironment.BaseAddress)
});
builder.Services
    .AddBlazorise(options => { options.Immediate = true; })
    .AddAntDesignProviders()
    .AddFontAwesomeIcons();

await builder.Build().RunAsync();
