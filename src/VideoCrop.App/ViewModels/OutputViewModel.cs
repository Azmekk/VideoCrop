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
        if (_sourcePath is null || OutputDirectory is null || _codec is null)
        {
            FilenamePreview = "";
            return;
        }
        try
        {
            var path = OutputNamer.GetNextAvailable(_sourcePath, OutputDirectory, _codec.Value.DefaultContainerExtension());
            FilenamePreview = Path.GetFileName(path);
        }
        catch
        {
            FilenamePreview = "";
        }
    }

    public string? BuildOutputPath()
    {
        if (_sourcePath is null || OutputDirectory is null || _codec is null) return null;
        return OutputNamer.GetNextAvailable(_sourcePath, OutputDirectory, _codec.Value.DefaultContainerExtension());
    }
}
