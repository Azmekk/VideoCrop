using System;
using Microsoft.Extensions.Logging;
using Microsoft.UI.Xaml;
using Serilog;
using VideoCrop.App.Services;
using VideoCrop.Core.IO;

namespace VideoCrop.App;

public partial class App : Application
{
    public static new App Current => (App)Application.Current;

    public IToolLocator ToolLocator { get; } = new ToolLocator();
    public ILoggerFactory LoggerFactory { get; }
    public AppServices Services { get; }

    public Window? MainWindow => _window;
    private Window? _window;

    public App()
    {
        // Attach to the parent terminal if one exists (e.g. `dotnet run` or a
        // user launching from PowerShell) so logs are visible there. End users
        // who double-click the exe get nothing — no stray console window —
        // and Serilog's file sink still captures the full log either way.
        if (VideoCrop.App.Interop.NativeMethods.AttachConsole(VideoCrop.App.Interop.NativeMethods.ATTACH_PARENT_PROCESS))
        {
            Console.OutputEncoding = System.Text.Encoding.UTF8;
        }

        var logFile = System.IO.Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "VideoCrop", "logs", "videocrop-.log");

        Log.Logger = new LoggerConfiguration()
            .MinimumLevel.Debug()
            .WriteTo.Console(outputTemplate: "{Timestamp:HH:mm:ss.fff} [{Level:u3}] {SourceContext} {Message:lj}{NewLine}{Exception}")
            .WriteTo.File(logFile, rollingInterval: RollingInterval.Day, retainedFileCountLimit: 7)
            .CreateLogger();

        LoggerFactory = Microsoft.Extensions.Logging.LoggerFactory.Create(builder =>
        {
            builder.AddSerilog(dispose: false);
        });

        Services = new AppServices(ToolLocator, LoggerFactory);

        InitializeComponent();

        UnhandledException += (_, e) =>
        {
            Log.Fatal(e.Exception, "Unhandled exception");
        };
    }


    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        _window = new MainWindow();

        // WinUI 3 doesn't install a SynchronizationContext on the UI thread by default,
        // so `await` continuations resume on the thread pool and any captured
        // SynchronizationContext.Current is null. Install one based on the window's
        // DispatcherQueue so view-model awaits and posted callbacks marshal correctly.
        var syncContext = new Microsoft.UI.Dispatching.DispatcherQueueSynchronizationContext(
            _window.DispatcherQueue);
        System.Threading.SynchronizationContext.SetSynchronizationContext(syncContext);

        _window.Activate();
    }
}
