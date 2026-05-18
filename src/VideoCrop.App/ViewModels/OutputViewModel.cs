using System;
using System.IO;
using CommunityToolkit.Mvvm.ComponentModel;
using VideoCrop.Core.IO;
using VideoCrop.Core.Models;

namespace VideoCrop.App.ViewModels;

public sealed class OutputViewModel : ObservableObject
{
    private string? _outputDirectory;
    private string _filenamePreview = "";

    public string? OutputDirectory
    {
        get => _outputDirectory;
        set
        {
            if (SetProperty(ref _outputDirectory, value)) RefreshFilenamePreview();
        }
    }

    public string FilenamePreview { get => _filenamePreview; private set => SetProperty(ref _filenamePreview, value); }

    private string? _sourcePath;
    private VideoCodec? _codec;

    public void Bind(string? sourcePath, VideoCodec? codec)
    {
        _sourcePath = sourcePath;
        _codec = codec;
        if (sourcePath is not null && OutputDirectory is null)
        {
            OutputDirectory = Path.GetDirectoryName(sourcePath);
        }
        RefreshFilenamePreview();
    }

    private void RefreshFilenamePreview()
    {
        var ext = ResolveExtension();
        if (_sourcePath is null || OutputDirectory is null || ext is null)
        {
            FilenamePreview = "";
            return;
        }
        try
        {
            var path = OutputNamer.GetNextAvailable(_sourcePath, OutputDirectory, ext);
            FilenamePreview = Path.GetFileName(path);
        }
        catch
        {
            FilenamePreview = "";
        }
    }

    public string? BuildOutputPath()
    {
        var ext = ResolveExtension();
        if (_sourcePath is null || OutputDirectory is null || ext is null) return null;
        return OutputNamer.GetNextAvailable(_sourcePath, OutputDirectory, ext);
    }

    /// <summary>
    /// Pick a container extension: when a codec is known, use its canonical
    /// container; otherwise (stream-copy) preserve the source's extension.
    /// </summary>
    private string? ResolveExtension()
    {
        if (_codec is { } c) return c.DefaultContainerExtension();
        if (_sourcePath is null) return null;
        var srcExt = Path.GetExtension(_sourcePath);
        return string.IsNullOrEmpty(srcExt) ? null : srcExt.TrimStart('.');
    }
}
