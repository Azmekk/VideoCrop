using System;
using System.IO;
using System.Text.Json;

namespace VideoCrop.App.Services;

/// <summary>
/// Tiny JSON-backed key/value bag in %LOCALAPPDATA%\VideoCrop\settings.json.
/// Holds opt-in flags that the user has answered explicitly.
/// </summary>
public sealed class AppSettingsService
{
    private readonly string _path;
    private Data _data = new();

    public AppSettingsService()
    {
        var dir = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "VideoCrop");
        Directory.CreateDirectory(dir);
        _path = Path.Combine(dir, "settings.json");
        Load();
    }

    /// <summary>
    /// True when the user clicked "Not now" on the install-setup prompt. The
    /// prompt is suppressed until the shortcut is created some other way
    /// (e.g., by another VideoCrop install) or the file is wiped.
    /// </summary>
    public bool UserDeclinedInstall
    {
        get => _data.UserDeclinedInstall;
        set { _data.UserDeclinedInstall = value; Save(); }
    }

    private void Load()
    {
        try
        {
            if (!File.Exists(_path)) return;
            var json = File.ReadAllText(_path);
            _data = JsonSerializer.Deserialize<Data>(json) ?? new Data();
        }
        catch { /* corrupted — ignore, start fresh */ }
    }

    private void Save()
    {
        try { File.WriteAllText(_path, JsonSerializer.Serialize(_data)); }
        catch { /* not fatal */ }
    }

    private sealed class Data
    {
        public bool UserDeclinedInstall { get; set; }
    }
}
