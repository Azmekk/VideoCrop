namespace VideoCrop.Core.IO;

public sealed class TempFileManager : IAsyncDisposable, IDisposable
{
    private readonly List<string> _paths = new();
    private bool _disposed;

    public string Allocate(string extension)
    {
        if (!extension.StartsWith('.')) extension = "." + extension;
        var path = Path.Combine(Path.GetTempPath(), "videocrop_" + Guid.NewGuid().ToString("N") + extension);
        _paths.Add(path);
        return path;
    }

    public void Register(string path)
    {
        if (string.IsNullOrEmpty(path)) return;
        _paths.Add(path);
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        foreach (var path in _paths) TryDelete(path);
    }

    public ValueTask DisposeAsync()
    {
        Dispose();
        return ValueTask.CompletedTask;
    }

    private static void TryDelete(string path)
    {
        try { if (File.Exists(path)) File.Delete(path); }
        catch { /* best-effort cleanup */ }
    }
}
