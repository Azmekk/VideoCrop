using System.Diagnostics;
using System.Text;

namespace VideoCrop.Core.Processes;

public sealed class ExternalProcessOptions
{
    public string FileName { get; init; } = "";
    public IReadOnlyList<string> Arguments { get; init; } = Array.Empty<string>();
    public string? WorkingDirectory { get; init; }
    public Action<string>? OnStdOut { get; init; }
    public Action<string>? OnStdErr { get; init; }
    public bool CaptureStdErr { get; init; } = true;
    public bool RedirectStdIn { get; init; }
}

public sealed class ExternalProcessResult
{
    public required int ExitCode { get; init; }
    public required string StdErr { get; init; }
}

public static class ExternalProcess
{
    public static async Task<ExternalProcessResult> RunAsync(
        ExternalProcessOptions options,
        CancellationToken cancellationToken)
    {
        ArgumentException.ThrowIfNullOrEmpty(options.FileName);

        var psi = new ProcessStartInfo
        {
            FileName = options.FileName,
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            RedirectStandardInput = options.RedirectStdIn,
            WorkingDirectory = options.WorkingDirectory ?? Path.GetDirectoryName(options.FileName) ?? Environment.CurrentDirectory,
        };
        foreach (var a in options.Arguments) psi.ArgumentList.Add(a);

        using var process = new Process { StartInfo = psi, EnableRaisingEvents = true };

        var stderr = new StringBuilder();

        process.OutputDataReceived += (_, e) =>
        {
            if (e.Data is null) return;
            options.OnStdOut?.Invoke(e.Data);
        };
        process.ErrorDataReceived += (_, e) =>
        {
            if (e.Data is null) return;
            if (options.CaptureStdErr) stderr.AppendLine(e.Data);
            options.OnStdErr?.Invoke(e.Data);
        };

        if (!process.Start())
            throw new InvalidOperationException($"Failed to start: {options.FileName}");

        process.BeginOutputReadLine();
        process.BeginErrorReadLine();

        using var registration = cancellationToken.Register(() =>
        {
            try { if (!process.HasExited) process.Kill(entireProcessTree: true); }
            catch { /* race with natural exit */ }
        });

        try
        {
            await process.WaitForExitAsync(CancellationToken.None).ConfigureAwait(false);
        }
        catch (OperationCanceledException)
        {
            try { if (!process.HasExited) process.Kill(entireProcessTree: true); } catch { }
            throw;
        }

        return new ExternalProcessResult
        {
            ExitCode = process.ExitCode,
            StdErr = stderr.ToString(),
        };
    }

    public static async Task<string> RunCaptureStdOutAsync(
        string fileName,
        IReadOnlyList<string> arguments,
        CancellationToken cancellationToken)
    {
        var sb = new StringBuilder();
        var result = await RunAsync(new ExternalProcessOptions
        {
            FileName = fileName,
            Arguments = arguments,
            OnStdOut = line => sb.AppendLine(line),
        }, cancellationToken).ConfigureAwait(false);

        if (result.ExitCode != 0)
            throw new ExternalProcessException(fileName, result.ExitCode, result.StdErr);

        return sb.ToString();
    }
}

public sealed class ExternalProcessException(string tool, int exitCode, string stderr)
    : Exception($"{Path.GetFileName(tool)} exited with code {exitCode}: {Truncate(stderr)}")
{
    public string Tool { get; } = tool;
    public int ExitCode { get; } = exitCode;
    public string StdErr { get; } = stderr;

    private static string Truncate(string s) => s.Length <= 500 ? s : s[..500] + "…";
}
