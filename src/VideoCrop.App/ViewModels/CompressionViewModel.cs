using System;
using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using VideoCrop.Core.Encoding;
using VideoCrop.Core.Models;

namespace VideoCrop.App.ViewModels;

public enum CompressionRateMode { Crf, TargetBitrate, TargetSize }

public sealed class CompressionViewModel : ObservableObject
{
    public ObservableCollection<PresetDefinition> Presets { get; } = new();

    private PresetDefinition _selectedPreset;
    private bool _enabled = true;
    private bool _advancedEnabled;
    private VideoCodec _codec = VideoCodec.H264;
    private SpeedPreset _speed = SpeedPreset.Medium;
    private CompressionRateMode _rateMode = CompressionRateMode.Crf;
    private int _crf = 23;
    private long _targetBitrateBps = 2_500_000;
    private long _targetSizeMb = 25;
    private AudioCodec _audioCodec = AudioCodec.Aac;
    private int _audioBitrateKbps = 128;
    private PixelFormat _pixelFormat = PixelFormat.Yuv420p;

    public CompressionViewModel()
    {
        foreach (var p in PresetLibrary.All) Presets.Add(p);
        _selectedPreset = PresetLibrary.WebMedium;
    }

    public PresetDefinition SelectedPreset
    {
        get => _selectedPreset;
        set
        {
            if (SetProperty(ref _selectedPreset, value))
            {
                _codec = value.Spec.Codec;
                _speed = value.Spec.Speed;
                _crf = value.Spec.Crf;
                _audioCodec = value.Spec.AudioCodec;
                _audioBitrateKbps = value.Spec.AudioBitrateKbps;
                _pixelFormat = value.Spec.PixelFormat;
                _rateMode = CompressionRateMode.Crf;
                OnAllChanged();
                OnPropertyChanged(nameof(AudioBitrateKbps));
            }
        }
    }

    /// <summary>
    /// Master toggle. When false, the user has opted out of re-encoding and
    /// the encoder will stream-copy both tracks. <see cref="SpecOrNull"/>
    /// reflects this by returning null.
    /// </summary>
    public bool Enabled
    {
        get => _enabled;
        set { if (SetProperty(ref _enabled, value)) OnAllChanged(); }
    }

    public bool AdvancedEnabled
    {
        get => _advancedEnabled;
        set
        {
            if (SetProperty(ref _advancedEnabled, value)) OnAllChanged();
        }
    }

    public VideoCodec Codec { get => _codec; set { if (SetProperty(ref _codec, value)) OnAllChanged(); } }
    public SpeedPreset Speed { get => _speed; set { if (SetProperty(ref _speed, value)) OnAllChanged(); } }
    public CompressionRateMode RateMode { get => _rateMode; set { if (SetProperty(ref _rateMode, value)) OnAllChanged(); } }
    public int Crf { get => _crf; set { if (SetProperty(ref _crf, value)) OnAllChanged(); } }
    public long TargetBitrateBps { get => _targetBitrateBps; set { if (SetProperty(ref _targetBitrateBps, value)) OnAllChanged(); } }
    public long TargetSizeMb { get => _targetSizeMb; set { if (SetProperty(ref _targetSizeMb, value)) OnAllChanged(); } }
    public AudioCodec AudioCodec { get => _audioCodec; set { if (SetProperty(ref _audioCodec, value)) OnAllChanged(); } }
    public int AudioBitrateKbps { get => _audioBitrateKbps; set { if (SetProperty(ref _audioBitrateKbps, value)) OnAllChanged(); } }
    public PixelFormat PixelFormat { get => _pixelFormat; set { if (SetProperty(ref _pixelFormat, value)) OnAllChanged(); } }

    public bool IsTwoPass => _enabled && _advancedEnabled && _rateMode == CompressionRateMode.TargetSize;

    /// <summary>Spec to feed into the encoder, or null when re-encoding is disabled.</summary>
    public CompressionSpec? SpecOrNull => _enabled ? Spec : null;

    public CompressionSpec Spec
    {
        get
        {
            if (!_advancedEnabled) return _selectedPreset.Spec;
            var (rateControl, targetBps, targetBytes) = _rateMode switch
            {
                CompressionRateMode.Crf => (RateControl.Crf, (long?)null, (long?)null),
                CompressionRateMode.TargetBitrate => (RateControl.Vbr, (long?)_targetBitrateBps, (long?)null),
                CompressionRateMode.TargetSize => (RateControl.TwoPassTargetSize, (long?)null, (long?)(_targetSizeMb * 1024L * 1024L)),
                _ => (RateControl.Crf, (long?)null, (long?)null),
            };
            return new CompressionSpec(
                Codec: _codec,
                RateControl: rateControl,
                Crf: _crf,
                TargetBitrateBps: targetBps,
                TargetSizeBytes: targetBytes,
                Speed: _speed,
                AudioCodec: _audioCodec,
                AudioBitrateKbps: _audioBitrateKbps,
                PixelFormat: _pixelFormat);
        }
    }

    public string Description =>
        !_enabled ? "Re-encoding off — video and audio will be stream-copied."
        : _advancedEnabled ? "Custom (advanced)"
        : _selectedPreset.Description;
    public string? Warning => !_enabled || _advancedEnabled ? null : _selectedPreset.CompatibilityWarning;
    public bool HasWarning => _enabled && !_advancedEnabled && !string.IsNullOrEmpty(_selectedPreset.CompatibilityWarning);

    private void OnAllChanged()
    {
        OnPropertyChanged(nameof(Spec));
        OnPropertyChanged(nameof(SpecOrNull));
        OnPropertyChanged(nameof(Description));
        OnPropertyChanged(nameof(Warning));
        OnPropertyChanged(nameof(HasWarning));
        OnPropertyChanged(nameof(IsTwoPass));
    }
}
