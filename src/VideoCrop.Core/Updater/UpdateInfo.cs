namespace VideoCrop.Core.Updater;

public sealed record UpdateInfo(
    Version LatestVersion,
    string TagName,
    string AssetUrl,
    long AssetSize,
    string AssetName,
    string? ReleaseNotes,
    string ReleaseUrl);

public enum UpdateState
{
    Idle,
    Checking,
    UpToDate,
    UpdateAvailable,
    Downloading,
    Staged,
    Applying,
    Failed,
}
