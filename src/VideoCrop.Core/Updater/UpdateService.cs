using System.IO.Compression;
using System.Net.Http.Headers;
using System.Net.Http.Json;
using System.Runtime.InteropServices;
using System.Text.Json;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;

namespace VideoCrop.Core.Updater;

public interface IUpdateService
{
    Task<UpdateInfo?> CheckForUpdateAsync(Version currentVersion, CancellationToken ct);
    Task<string?> DownloadAndStageAsync(UpdateInfo info, IProgress<double>? progress, CancellationToken ct);
}

public sealed class UpdateService(
    HttpClient http,
    string repoOwnerSlashName,
    string stagingRoot,
    ILogger<UpdateService>? logger = null) : IUpdateService
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNameCaseInsensitive = true,
    };

    private readonly ILogger<UpdateService> _logger = logger ?? NullLogger<UpdateService>.Instance;

    public async Task<UpdateInfo?> CheckForUpdateAsync(Version currentVersion, CancellationToken ct)
    {
        if (string.IsNullOrWhiteSpace(repoOwnerSlashName) || repoOwnerSlashName.Contains("YOUR-REPO", StringComparison.OrdinalIgnoreCase))
        {
            _logger.LogInformation("Updater: repository not configured, skipping check.");
            return null;
        }

        var url = $"https://api.github.com/repos/{repoOwnerSlashName}/releases/latest";
        _logger.LogInformation("Updater: checking {Url}", url);

        using var request = new HttpRequestMessage(HttpMethod.Get, url);
        request.Headers.UserAgent.ParseAdd("VideoCrop-Updater/1.0");
        request.Headers.Accept.Add(new MediaTypeWithQualityHeaderValue("application/vnd.github+json"));

        GitHubRelease? release;
        try
        {
            using var response = await http.SendAsync(request, HttpCompletionOption.ResponseContentRead, ct).ConfigureAwait(false);
            if (response.StatusCode == System.Net.HttpStatusCode.NotFound)
            {
                _logger.LogInformation("Updater: no releases published yet.");
                return null;
            }
            response.EnsureSuccessStatusCode();
            release = await response.Content.ReadFromJsonAsync<GitHubRelease>(JsonOptions, ct).ConfigureAwait(false);
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Updater: release check failed.");
            return null;
        }

        if (release is null || release.Draft) return null;

        var latestVersion = ParseVersion(release.TagName);
        if (latestVersion is null)
        {
            _logger.LogWarning("Updater: unparseable tag '{Tag}'.", release.TagName);
            return null;
        }
        if (latestVersion <= currentVersion)
        {
            _logger.LogInformation("Updater: already up to date (current {Current}, latest {Latest}).", currentVersion, latestVersion);
            return null;
        }

        var asset = SelectAsset(release.Assets);
        if (asset is null)
        {
            _logger.LogWarning("Updater: latest release {Tag} has no matching asset.", release.TagName);
            return null;
        }

        return new UpdateInfo(
            LatestVersion: latestVersion,
            TagName: release.TagName ?? "",
            AssetUrl: asset.BrowserDownloadUrl ?? "",
            AssetSize: asset.Size,
            AssetName: asset.Name ?? "",
            ReleaseNotes: release.Body,
            ReleaseUrl: release.HtmlUrl ?? "");
    }

    public async Task<string?> DownloadAndStageAsync(UpdateInfo info, IProgress<double>? progress, CancellationToken ct)
    {
        Directory.CreateDirectory(stagingRoot);
        var zipPath = Path.Combine(stagingRoot, info.AssetName);
        var stagedDir = Path.Combine(stagingRoot, "staged");

        // Best-effort clean of previous staging artifacts.
        TryClearDirectory(stagedDir);
        TryDeleteFile(zipPath);

        _logger.LogInformation("Updater: downloading {Url} -> {Path}", info.AssetUrl, zipPath);
        try
        {
            using var request = new HttpRequestMessage(HttpMethod.Get, info.AssetUrl);
            request.Headers.UserAgent.ParseAdd("VideoCrop-Updater/1.0");
            using var response = await http.SendAsync(request, HttpCompletionOption.ResponseHeadersRead, ct).ConfigureAwait(false);
            response.EnsureSuccessStatusCode();

            var total = response.Content.Headers.ContentLength ?? info.AssetSize;
            await using var dest = File.Create(zipPath);
            await using var src = await response.Content.ReadAsStreamAsync(ct).ConfigureAwait(false);

            var buffer = new byte[81920];
            long copied = 0;
            int read;
            while ((read = await src.ReadAsync(buffer, ct).ConfigureAwait(false)) > 0)
            {
                await dest.WriteAsync(buffer.AsMemory(0, read), ct).ConfigureAwait(false);
                copied += read;
                if (total > 0) progress?.Report(Math.Clamp((double)copied / total, 0, 1));
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Updater: download failed.");
            TryDeleteFile(zipPath);
            return null;
        }

        try
        {
            Directory.CreateDirectory(stagedDir);
            ZipFile.ExtractToDirectory(zipPath, stagedDir, overwriteFiles: true);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Updater: extraction failed.");
            return null;
        }

        // GitHub release zips often contain a top-level folder. Flatten it.
        var flattened = FlattenSingleTopLevelFolder(stagedDir);
        _logger.LogInformation("Updater: staged update at {Path}", flattened);
        return flattened;
    }

    internal static Version? ParseVersion(string? tag)
    {
        if (string.IsNullOrWhiteSpace(tag)) return null;
        var clean = tag.TrimStart('v', 'V').Trim();
        // Strip any pre-release suffix after a hyphen.
        var dashIdx = clean.IndexOf('-');
        if (dashIdx >= 0) clean = clean[..dashIdx];
        return Version.TryParse(clean, out var v) ? v : null;
    }

    internal static GitHubAsset? SelectAsset(IReadOnlyList<GitHubAsset> assets)
    {
        var arch = RuntimeInformation.ProcessArchitecture.ToString().ToLowerInvariant();
        // 1. Exact arch + zip match
        foreach (var a in assets)
        {
            var name = a.Name ?? "";
            if (name.EndsWith(".zip", StringComparison.OrdinalIgnoreCase)
                && name.Contains($"win-{arch}", StringComparison.OrdinalIgnoreCase))
                return a;
        }
        // 2. Any win-* zip
        foreach (var a in assets)
        {
            var name = a.Name ?? "";
            if (name.EndsWith(".zip", StringComparison.OrdinalIgnoreCase)
                && name.Contains("win", StringComparison.OrdinalIgnoreCase))
                return a;
        }
        // 3. First zip
        foreach (var a in assets)
        {
            if ((a.Name ?? "").EndsWith(".zip", StringComparison.OrdinalIgnoreCase))
                return a;
        }
        return null;
    }

    private static string FlattenSingleTopLevelFolder(string dir)
    {
        var entries = Directory.GetFileSystemEntries(dir);
        if (entries.Length == 1 && Directory.Exists(entries[0]))
            return entries[0];
        return dir;
    }

    private static void TryClearDirectory(string path)
    {
        try { if (Directory.Exists(path)) Directory.Delete(path, recursive: true); }
        catch { /* best-effort */ }
    }

    private static void TryDeleteFile(string path)
    {
        try { if (File.Exists(path)) File.Delete(path); }
        catch { /* best-effort */ }
    }
}
