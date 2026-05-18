using System;
using System.IO;
using System.Text.Json;

namespace VideoCrop.App.Services;

/// <summary>
/// Tiny JSON-backed key/value bag in %LOCALAPPDATA%\VideoCrop\settings.json.
/// Holds opt-in flags that the user has answered explicitly (e.g., "do not
/// ask me about the Start menu shortcut again").
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

    public bool HasPromptedInstallSetup
    {
        get => _data.HasPromptedInstallSetup;
        set { _data.HasPromptedInstallSetup = value; Save(); }
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
        public bool HasPromptedInstallSetup { get; set; }
    }
}
