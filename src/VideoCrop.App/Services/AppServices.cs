using Microsoft.Extensions.Logging;
using VideoCrop.Core.IO;

namespace VideoCrop.App.Services;

public sealed class AppServices(IToolLocator toolLocator, ILoggerFactory loggerFactory)
{
    public IToolLocator ToolLocator { get; } = toolLocator;
    public ILoggerFactory LoggerFactory { get; } = loggerFactory;
    public RecentFilesService RecentFiles { get; } = new();
    public AppSettingsService Settings { get; } = new();
}
