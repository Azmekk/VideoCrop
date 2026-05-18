using System.IO.Compression;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Net.Http.Json;
using System.Runtime.InteropServices;
using System.Text.Json.Serialization;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;
using SharpCompress.Archives;
using SharpCompress.Archives.SevenZip;

namespace VideoCrop.Core.IO;

public sealed record ToolDownloadProgress(string Tool, double Fraction, string Status);

public interface IToolDownloadService
{
    /// <summary>
    /// Downloads ffmpeg+ffprobe (BtbN GPL build) and mpv (shinchiro build) into
    /// the locator's tools directory. Returns true if all three landed.
    /// Existing files are overwritten.
    /// </summary>
    Task<bool> DownloadAllAsync(IProgress<ToolDownloadProgress>? progress, CancellationToken ct);
}

public sealed class ToolDownloadService(
    IToolLocator locator,
    HttpClient http,
    ILogger<ToolDownloadService>? logger = null) : IToolDownloadService
{
    private readonly ILogger<ToolDownloadService> _logger = logger ?? NullLogger<ToolDownloadService>.Instance;

    public async Task<bool> DownloadAllAsync(IProgress<ToolDownloadProgress>? progress, CancellationToken ct)
    {
        Directory.CreateDirectory(locator.ToolsDirectory);

        var (ffmpegArch, mpvArch) = ResolveArchitectures();

        try
        {
            await DownloadFfmpegAsync(ffmpegArch, progress, ct);
            await DownloadMpvAsync(mpvArch, progress, ct);
            return true;
        }
        catch (OperationCanceledException)
        {
            throw;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Tool download failed");
            return false;
        }
    }

    private async Task DownloadFfmpegAsync(string arch, IProgress<ToolDownloadProgress>? progress, CancellationToken ct)
    {
        // BtbN GPL build — needed for libx264/libx265 (H.264/H.265 encoding).
        var url = $"https://github.com/BtbN/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-{arch}-gpl.zip";
        var tempZip = Path.Combine(Path.GetTempPath(), $"videocrop-ffmpeg-{Guid.NewGuid():N}.zip");
        var tempExtract = Path.Combine(Path.GetTempPath(), $"videocrop-ffmpeg-extract-{Guid.NewGuid():N}");

        try
        {
            await DownloadFileAsync(url, tempZip, "ffmpeg", progress, ct);

            Report(progress, "ffmpeg", 0.9, "Extracting…");
            Directory.CreateDirectory(tempExtract);
            ZipFile.ExtractToDirectory(tempZip, tempExtract);

            var ffmpegExe  = FindFile(tempExtract, "ffmpeg.exe")  ?? throw new FileNotFoundException("ffmpeg.exe not found in archive");
            var ffprobeExe = FindFile(tempExtract, "ffprobe.exe") ?? throw new FileNotFoundException("ffprobe.exe not found in archive");

            File.Copy(ffmpegExe,  locator.GetExpectedPath(ExternalTool.Ffmpeg),  overwrite: true);
            File.Copy(ffprobeExe, locator.GetExpectedPath(ExternalTool.Ffprobe), overwrite: true);

            // Best-effort copy of the LICENSE that ships in BtbN's zip.
            var license = FindFileByPredicate(tempExtract, n => n.StartsWith("LICENSE", StringComparison.OrdinalIgnoreCase));
            if (license is not null)
            {
                File.Copy(license, Path.Combine(locator.ToolsDirectory, "ffmpeg-LICENSE.txt"), overwrite: true);
            }
            Report(progress, "ffmpeg", 1.0, "Done");
        }
        finally
        {
            TryDelete(tempZip);
            TryDeleteDir(tempExtract);
        }
    }

    private async Task DownloadMpvAsync(string arch, IProgress<ToolDownloadProgress>? progress, CancellationToken ct)
    {
        // mpv distributes only .7z, so we have to resolve the latest asset
        // URL via the GitHub API and extract with SharpCompress.
        Report(progress, "mpv", 0.0, "Resolving latest build…");
        var assetUrl = await ResolveMpvAssetUrlAsync(arch, ct);

        var temp7z = Path.Combine(Path.GetTempPath(), $"videocrop-mpv-{Guid.NewGuid():N}.7z");
        var tempExtract = Path.Combine(Path.GetTempPath(), $"videocrop-mpv-extract-{Guid.NewGuid():N}");

        try
        {
            await DownloadFileAsync(assetUrl, temp7z, "mpv", progress, ct);

            Report(progress, "mpv", 0.9, "Extracting…");
            Directory.CreateDirectory(tempExtract);
            ExtractSevenZip(temp7z, tempExtract);

            var mpvExe = FindFile(tempExtract, "mpv.exe") ?? throw new FileNotFoundException("mpv.exe not found in archive");
            File.Copy(mpvExe, locator.GetExpectedPath(ExternalTool.Mpv), overwrite: true);

            var license = FindFileByPredicate(tempExtract, n => n.StartsWith("LICENSE", StringComparison.OrdinalIgnoreCase));
            if (license is not null)
            {
                File.Copy(license, Path.Combine(locator.ToolsDirectory, "mpv-LICENSE.txt"), overwrite: true);
            }
            Report(progress, "mpv", 1.0, "Done");
        }
        finally
        {
            TryDelete(temp7z);
            TryDeleteDir(tempExtract);
        }
    }

    private async Task<string> ResolveMpvAssetUrlAsync(string arch, CancellationToken ct)
    {
        using var request = new HttpRequestMessage(HttpMethod.Get, "https://api.github.com/repos/shinchiro/mpv-winbuild-cmake/releases/latest");
        request.Headers.UserAgent.ParseAdd("VideoCrop/1.0");
        request.Headers.Accept.Add(new MediaTypeWithQualityHeaderValue("application/vnd.github+json"));

        using var response = await http.SendAsync(request, HttpCompletionOption.ResponseContentRead, ct);
        response.EnsureSuccessStatusCode();
        var release = await response.Content.ReadFromJsonAsync<MpvRelease>(cancellationToken: ct)
            ?? throw new InvalidOperationException("mpv release info missing");

        // Asset names look like  mpv-x86_64-YYYYMMDD-git-HASH.7z .
        // Skip "dev" variants (dev libraries, not the player binary).
        var asset = release.Assets.FirstOrDefault(a =>
            a.Name is { } name
            && name.StartsWith($"mpv-{arch}-", StringComparison.OrdinalIgnoreCase)
            && name.EndsWith(".7z", StringComparison.OrdinalIgnoreCase)
            && !name.Contains("dev", StringComparison.OrdinalIgnoreCase));

        if (asset?.BrowserDownloadUrl is null)
            throw new InvalidOperationException($"No mpv asset for arch '{arch}' in release {release.TagName}");

        _logger.LogInformation("mpv asset: {Name}", asset.Name);
        return asset.BrowserDownloadUrl;
    }

    private async Task DownloadFileAsync(string url, string destination, string tool, IProgress<ToolDownloadProgress>? progress, CancellationToken ct)
    {
        _logger.LogInformation("Downloading {Tool} from {Url}", tool, url);
        Report(progress, tool, 0.0, "Connecting…");

        using var request = new HttpRequestMessage(HttpMethod.Get, url);
        request.Headers.UserAgent.ParseAdd("VideoCrop/1.0");
        using var response = await http.SendAsync(request, HttpCompletionOption.ResponseHeadersRead, ct);
        response.EnsureSuccessStatusCode();
        var total = response.Content.Headers.ContentLength ?? 0;

        await using var dest = File.Create(destination);
        await using var src = await response.Content.ReadAsStreamAsync(ct);

        var buffer = new byte[81920];
        long copied = 0;
        int read;
        var lastReport = 0.0;
        while ((read = await src.ReadAsync(buffer, ct)) > 0)
        {
            await dest.WriteAsync(buffer.AsMemory(0, read), ct);
            copied += read;
            if (total > 0)
            {
                // 0..0.9 reserved for download, 0.9..1.0 for extract.
                var fraction = Math.Clamp((double)copied / total * 0.9, 0, 0.9);
                if (fraction - lastReport > 0.01)
                {
                    Report(progress, tool, fraction, FormatStatus(copied, total));
                    lastReport = fraction;
                }
            }
        }
    }

    private static void ExtractSevenZip(string archivePath, string destination)
    {
        using var archive = SevenZipArchive.OpenArchive(archivePath, new SharpCompress.Readers.ReaderOptions());
        foreach (var entry in archive.Entries.Where(e => !e.IsDirectory))
        {
            if (entry.Key is null) continue;
            var outPath = Path.Combine(destination, entry.Key);
            Directory.CreateDirectory(Path.GetDirectoryName(outPath)!);
            using var stream = entry.OpenEntryStream();
            using var outStream = File.Create(outPath);
            stream.CopyTo(outStream);
        }
    }

    private static (string Ffmpeg, string Mpv) ResolveArchitectures() => RuntimeInformation.ProcessArchitecture switch
    {
        Architecture.X64   => ("win64",    "x86_64"),
        Architecture.Arm64 => ("winarm64", "aarch64"),
        _ => throw new PlatformNotSupportedException(
            $"Unsupported architecture {RuntimeInformation.ProcessArchitecture}; VideoCrop runs on x64 or arm64."),
    };

    private static string? FindFile(string root, string filename) =>
        Directory.EnumerateFiles(root, filename, SearchOption.AllDirectories).FirstOrDefault();

    private static string? FindFileByPredicate(string root, Func<string, bool> predicate) =>
        Directory.EnumerateFiles(root, "*", SearchOption.AllDirectories)
            .FirstOrDefault(p => predicate(Path.GetFileName(p)));

    private static void Report(IProgress<ToolDownloadProgress>? progress, string tool, double fraction, string status) =>
        progress?.Report(new ToolDownloadProgress(tool, fraction, status));

    private static string FormatStatus(long copied, long total)
    {
        var mb = copied / 1024.0 / 1024.0;
        var totalMb = total / 1024.0 / 1024.0;
        return $"{mb:0.#} / {totalMb:0.#} MB";
    }

    private static void TryDelete(string path) { try { if (File.Exists(path)) File.Delete(path); } catch { } }
    private static void TryDeleteDir(string path) { try { if (Directory.Exists(path)) Directory.Delete(path, recursive: true); } catch { } }

    private sealed class MpvRelease
    {
        [JsonPropertyName("tag_name")] public string? TagName { get; set; }
        [JsonPropertyName("assets")] public List<MpvAsset> Assets { get; set; } = new();
    }

    private sealed class MpvAsset
    {
        [JsonPropertyName("name")] public string? Name { get; set; }
        [JsonPropertyName("browser_download_url")] public string? BrowserDownloadUrl { get; set; }
    }
}
