using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;
using VideoCrop.Core.Encoding;
using VideoCrop.Core.IO;

namespace VideoCrop.Core.Processes;

public interface IFfmpegRunner
{
    Task<int> RunAsync(
        IReadOnlyList<string> arguments,
        Action<EncodeProgress>? onProgress,
        Action<string>? onStdErrLine,
        CancellationToken cancellationToken);
}

public sealed class FfmpegRunner(IToolLocator locator, ILogger<FfmpegRunner>? logger = null) : IFfmpegRunner
{
    private readonly ILogger<FfmpegRunner> _logger = logger ?? NullLogger<FfmpegRunner>.Instance;

    public async Task<int> RunAsync(
        IReadOnlyList<string> arguments,
        Action<EncodeProgress>? onProgress,
        Action<string>? onStdErrLine,
        CancellationToken cancellationToken)
    {
        if (!locator.TryResolve(ExternalTool.Ffmpeg, out var ffmpeg))
            throw new InvalidOperationException("ffmpeg not found.");

        var parser = new EncodeProgressParser();

        var result = await ExternalProcess.RunAsync(new ExternalProcessOptions
        {
            FileName = ffmpeg,
            Arguments = arguments,
            OnStdOut = line =>
            {
                var p = parser.OnLine(line);
                if (p is not null) onProgress?.Invoke(p);
            },
            OnStdErr = line =>
            {
                onStdErrLine?.Invoke(line);
                _logger.LogTrace("ffmpeg stderr: {Line}", line);
            },
        }, cancellationToken).ConfigureAwait(false);

        if (result.ExitCode != 0 && !cancellationToken.IsCancellationRequested)
        {
            _logger.LogWarning("ffmpeg exited with {Code}: {Stderr}", result.ExitCode, result.StdErr);
        }
        return result.ExitCode;
    }
}
