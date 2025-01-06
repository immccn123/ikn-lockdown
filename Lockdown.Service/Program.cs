using Lockdown.Service;

var builder = Host.CreateApplicationBuilder(args);
builder.Services.AddWindowsService(options => {
    options.ServiceName = "Lockdown Service";
});
builder.Services.AddHostedService<WindowsWorker>();

var host = builder.Build();
host.Run();
