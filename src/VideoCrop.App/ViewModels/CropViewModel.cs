using System;
using CommunityToolkit.Mvvm.ComponentModel;
using VideoCrop.Core.Models;

namespace VideoCrop.App.ViewModels;

public sealed class CropViewModel : ObservableObject
{
    private int _sourceWidth;
    private int _sourceHeight;
    private CropSpec? _crop;

    public int SourceWidth { get => _sourceWidth; private set => SetProperty(ref _sourceWidth, value); }
    public int SourceHeight { get => _sourceHeight; private set => SetProperty(ref _sourceHeight, value); }

    public CropSpec? Crop
    {
        get => _crop;
        set
        {
            var sanitized = value?.Clamped(_sourceWidth, _sourceHeight);
            if (sanitized?.MatchesSource(_sourceWidth, _sourceHeight) == true)
                sanitized = null;
            if (SetProperty(ref _crop, sanitized))
            {
                OnPropertyChanged(nameof(IsActive));
                OnPropertyChanged(nameof(Summary));
                CropChanged?.Invoke(this, EventArgs.Empty);
            }
        }
    }

    public bool IsActive => _crop is not null;

    public string Summary
    {
        get
        {
            if (_crop is null) return _sourceWidth > 0 ? $"Full frame ({_sourceWidth}×{_sourceHeight})" : "(no source)";
            return $"{_crop.Width}×{_crop.Height} from {_sourceWidth}×{_sourceHeight} at ({_crop.X},{_crop.Y})";
        }
    }

    public event EventHandler? CropChanged;

    public void SetSource(int width, int height)
    {
        SourceWidth = width;
        SourceHeight = height;
        Crop = null;
    }
}
