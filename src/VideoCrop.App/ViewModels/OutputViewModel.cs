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
    private OutputMode _mode = OutputMode.Everything;

    public string? OutputDirectory
    {
        get => _outputDirectory;
        set
        {
            if (SetProperty(ref _outputDirectory, value)) RefreshFilenamePreview();
        }
    }

    public string FilenamePreview { get => _filenamePreview; private set => SetProperty(ref _filenamePreview, value); }

    /// <summary>What the user wants in the output: both tracks, audio only, or video only.</summary>
    public OutputMode Mode
    {
        get => _mode;
        set { if (SetProperty(ref _mode, value)) RefreshFilenamePreview(); }
    }

    private string? _sourcePath;
    private VideoCodec? _codec;
    private AudioCodec _audioCodec = AudioCodec.Aac;

    public void Bind(string? sourcePath, VideoCodec? codec, AudioCodec audioCodec)
    {
        _sourcePath = sourcePath;
        _codec = codec;
        _audioCodec = audioCodec;
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
    /// Pick a container extension based on the active output mode:
    /// <list type="bullet">
    ///   <item>Audio only → audio-codec container (.m4a/.opus/.mp3, .mka for copy).</item>
    ///   <item>Video / Everything → video-codec container, or source ext when stream-copy.</item>
    /// </list>
    /// </summary>
    private string? ResolveExtension()
    {
        if (_mode == OutputMode.AudioOnly)
        {
            return _audioCodec switch
            {
                AudioCodec.Mp3  => "mp3",
                AudioCodec.Opus => "opus",
                AudioCodec.Copy => "mka",
                _               => "m4a",
            };
        }
        if (_codec is { } c) return c.DefaultContainerExtension();
        if (_sourcePath is null) return null;
        var srcExt = Path.GetExtension(_sourcePath);
        return string.IsNullOrEmpty(srcExt) ? null : srcExt.TrimStart('.');
    }
}
