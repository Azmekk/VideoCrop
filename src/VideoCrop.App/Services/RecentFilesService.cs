using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;

namespace VideoCrop.App.Services;

public sealed class RecentFilesService
{
    private const int MaxItems = 10;
    private readonly string _path;
    private List<string> _items = new();

    public RecentFilesService()
    {
        var dir = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "VideoCrop");
        Directory.CreateDirectory(dir);
        _path = Path.Combine(dir, "recent.json");
        Load();
    }

    public IReadOnlyList<string> Items => _items;

    public void Add(string fullPath)
    {
        _items.RemoveAll(p => string.Equals(p, fullPath, StringComparison.OrdinalIgnoreCase));
        _items.Insert(0, fullPath);
        if (_items.Count > MaxItems) _items = _items.Take(MaxItems).ToList();
        Save();
    }

    public void Remove(string fullPath)
    {
        _items.RemoveAll(p => string.Equals(p, fullPath, StringComparison.OrdinalIgnoreCase));
        Save();
    }

    private void Load()
    {
        try
        {
            if (!File.Exists(_path)) return;
            var json = File.ReadAllText(_path);
            _items = JsonSerializer.Deserialize<List<string>>(json) ?? new List<string>();
        }
        catch { /* corrupted file — ignore */ }
    }

    private void Save()
    {
        try
        {
            File.WriteAllText(_path, JsonSerializer.Serialize(_items));
        }
        catch { /* not fatal */ }
    }
}
