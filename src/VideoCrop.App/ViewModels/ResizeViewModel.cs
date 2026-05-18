using System;
using CommunityToolkit.Mvvm.ComponentModel;
using VideoCrop.Core.Models;

namespace VideoCrop.App.ViewModels;

public sealed class ResizeViewModel : ObservableObject
{
    public static readonly int[] HeightPresets = { 2160, 1440, 1080, 720, 480 };

    private int _width;
    private int _height;
    private bool _enabled;
    private bool _aspectLocked = true;
    private int _inputWidth = 1920;
    private int _inputHeight = 1080;
    private int _sourceWidth;
    private int _sourceHeight;
    private bool _suppressLockMath;

    public bool Enabled
    {
        get => _enabled;
        set => SetProperty(ref _enabled, value);
    }

    public bool AspectLocked
    {
        get => _aspectLocked;
        set
        {
            if (SetProperty(ref _aspectLocked, value))
            {
                if (value) ApplyLock(fromWidth: true);
                OnPropertyChanged(nameof(MismatchWarning));
            }
        }
    }

    public int Width
    {
        get => _width;
        set
        {
            value = Math.Max(2, value);
            if (SetProperty(ref _width, ResizeSpec.RoundEven(value)))
            {
                if (_aspectLocked && !_suppressLockMath) ApplyLock(fromWidth: true);
                OnPropertyChanged(nameof(MismatchWarning));
            }
        }
    }

    public int Height
    {
        get => _height;
        set
        {
            value = Math.Max(2, value);
            if (SetProperty(ref _height, ResizeSpec.RoundEven(value)))
            {
                if (_aspectLocked && !_suppressLockMath) ApplyLock(fromWidth: false);
                OnPropertyChanged(nameof(MismatchWarning));
            }
        }
    }

    public int InputWidth { get => _inputWidth; private set => SetProperty(ref _inputWidth, value); }
    public int InputHeight { get => _inputHeight; private set => SetProperty(ref _inputHeight, value); }
    public bool CropActive { get; private set; }
    public string InputDisplay
    {
        get
        {
            if (_sourceWidth == 0) return "(no source)";
            var aspect = _inputHeight > 0 ? (double)_inputWidth / _inputHeight : 1.0;
            var label = $"{_inputWidth}×{_inputHeight} ({aspect:0.##}:1)";
            if (CropActive) label += $" — cropped from {_sourceWidth}×{_sourceHeight}";
            return label;
        }
    }

    public string? MismatchWarning
    {
        get
        {
            if (!_enabled || _aspectLocked) return null;
            if (_inputHeight == 0 || _height == 0) return null;
            var inputAspect = (double)_inputWidth / _inputHeight;
            var outputAspect = (double)_width / _height;
            if (Math.Abs(inputAspect - outputAspect) / inputAspect > 0.01)
                return "Aspect mismatch — output will be stretched.";
            return null;
        }
    }

    public ResizeSpec? AsSpecOrNull()
    {
        if (!_enabled) return null;
        return new ResizeSpec(_width, _height, _aspectLocked);
    }

    public string? ApplyPresetHeight(int height)
    {
        if (_inputHeight == 0) return null;
        var aspect = (double)_inputWidth / _inputHeight;
        var w = ResizeSpec.RoundEven((int)Math.Round(height * aspect));
        _suppressLockMath = true;
        Width = w;
        Height = ResizeSpec.RoundEven(height);
        _suppressLockMath = false;
        return $"Resize set to {Width}×{Height}.";
    }

    public string? OnCropChanged(CropSpec? crop, int sourceWidth, int sourceHeight)
    {
        _sourceWidth = sourceWidth;
        _sourceHeight = sourceHeight;
        if (crop is null)
        {
            InputWidth = sourceWidth;
            InputHeight = sourceHeight;
            CropActive = false;
        }
        else
        {
            InputWidth = crop.Width;
            InputHeight = crop.Height;
            CropActive = true;
        }
        OnPropertyChanged(nameof(InputDisplay));
        OnPropertyChanged(nameof(CropActive));

        if (_enabled && _aspectLocked && _inputHeight > 0)
        {
            // Maintain locked axis (width) — recompute height to new aspect.
            var newHeight = ResizeSpec.RoundEven((int)Math.Round(_width * (double)_inputHeight / _inputWidth));
            if (newHeight != _height)
            {
                _suppressLockMath = true;
                Height = newHeight;
                _suppressLockMath = false;
                return $"Resize updated to {_width}×{_height} to match new aspect.";
            }
        }
        OnPropertyChanged(nameof(MismatchWarning));
        return null;
    }

    public void SetSource(int width, int height)
    {
        _sourceWidth = width;
        _sourceHeight = height;
        InputWidth = width;
        InputHeight = height;
        CropActive = false;
        if (_width == 0 || _height == 0)
        {
            _suppressLockMath = true;
            Width = width;
            Height = height;
            _suppressLockMath = false;
        }
        OnPropertyChanged(nameof(InputDisplay));
        OnPropertyChanged(nameof(CropActive));
    }

    private void ApplyLock(bool fromWidth)
    {
        if (_inputHeight == 0 || _inputWidth == 0) return;
        _suppressLockMath = true;
        if (fromWidth)
        {
            Height = ResizeSpec.RoundEven((int)Math.Round(_width * (double)_inputHeight / _inputWidth));
        }
        else
        {
            Width = ResizeSpec.RoundEven((int)Math.Round(_height * (double)_inputWidth / _inputHeight));
        }
        _suppressLockMath = false;
    }
}
