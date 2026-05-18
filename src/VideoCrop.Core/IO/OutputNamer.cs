namespace VideoCrop.Core.IO;

public static class OutputNamer
{
    private const string Suffix = "_VideoCrop";

    public static string GetNextAvailable(string sourcePath, string outputDir, string extension)
    {
        if (string.IsNullOrWhiteSpace(sourcePath)) throw new ArgumentException("Source path required.", nameof(sourcePath));
        if (string.IsNullOrWhiteSpace(outputDir)) throw new ArgumentException("Output directory required.", nameof(outputDir));
        if (string.IsNullOrWhiteSpace(extension)) throw new ArgumentException("Extension required.", nameof(extension));

        if (!extension.StartsWith('.')) extension = "." + extension;

        var rawBase = Path.GetFileNameWithoutExtension(sourcePath);
        if (string.IsNullOrEmpty(rawBase)) rawBase = "output";
        var baseName = StripExistingSuffix(rawBase);

        var first = Path.Combine(outputDir, baseName + Suffix + extension);
        if (!File.Exists(first)) return first;

        for (var i = 2; i < int.MaxValue; i++)
        {
            var candidate = Path.Combine(outputDir, baseName + Suffix + i + extension);
            if (!File.Exists(candidate)) return candidate;
        }

        throw new IOException("Could not allocate unique output filename.");
    }

    private static string StripExistingSuffix(string name)
    {
        var idx = name.LastIndexOf(Suffix, StringComparison.Ordinal);
        if (idx < 0) return name;
        var tail = name[(idx + Suffix.Length)..];
        if (tail.Length == 0 || tail.All(char.IsDigit)) return name[..idx];
        return name;
    }
}
