using System;
using System.ComponentModel;
using System.Globalization;
using System.Threading.Tasks;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using VideoCrop.App.ViewModels;

namespace VideoCrop.App.Views;

public sealed partial class VideoPane : UserControl
{
    private PlayerViewModel? _player;
    private bool _seekingByUser;
    private bool _suppressSliderEvent;

    public VideoPane()
    {
        InitializeComponent();
        Loaded += OnLoaded;
        Unloaded += OnUnloaded;
    }

    public PlayerViewModel? Player => _player;

    public async Task LoadVideoAsync(string path)
    {
        if (_player is null) return;
        await _player.LoadFileAsync(path);
        EmptyOverlay.Visibility = Visibility.Collapsed;
        TransportBar.Visibility = _player.IsReady ? Visibility.Visible : Visibility.Collapsed;
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        _player = new PlayerViewModel(App.Current.ToolLocator, App.Current.LoggerFactory);
        _player.PropertyChanged += Player_PropertyChanged;

        if (!_player.HasMpv)
        {
            EmptyOverlay.Text = "mpv not found. Restart the app to download the required tools.";
        }
    }

    private async void OnUnloaded(object sender, RoutedEventArgs e)
    {
        if (_player is not null) await _player.DisposeAsync();
        _player = null;
    }

    private void Player_PropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (_player is null) return;

        switch (e.PropertyName)
        {
            case nameof(PlayerViewModel.IsPaused):
                PlayPauseLabel.Text = _player.IsPaused ? "Play" : "Pause";
                break;
            case nameof(PlayerViewModel.DurationSeconds):
                PositionSlider.Maximum = Math.Max(0.01, _player.DurationSeconds);
                UpdatePositionLabel();
                break;
            case nameof(PlayerViewModel.PositionSeconds):
                if (!_seekingByUser)
                {
                    _suppressSliderEvent = true;
                    PositionSlider.Value = Math.Min(_player.PositionSeconds, PositionSlider.Maximum);
                    _suppressSliderEvent = false;
                    UpdatePositionLabel();
                }
                break;
            case nameof(PlayerViewModel.IsReady):
                TransportBar.Visibility = _player.IsReady ? Visibility.Visible : Visibility.Collapsed;
                break;
        }
    }

    private async void OnPlayPauseClick(object sender, RoutedEventArgs e)
    {
        if (_player is null) return;
        await _player.TogglePauseAsync();
    }

    private void OnSliderValueChanged(object sender, RangeBaseValueChangedEventArgs e)
    {
        if (_suppressSliderEvent) return;
        _seekingByUser = true;
        UpdatePositionLabel();
    }

    private async void OnSliderPointerCaptureLost(object sender, Microsoft.UI.Xaml.Input.PointerRoutedEventArgs e)
    {
        if (_player is null) return;
        if (!_seekingByUser) return;
        var target = PositionSlider.Value;
        _seekingByUser = false;
        await _player.SeekAsync(target);
    }

    private void UpdatePositionLabel()
    {
        if (_player is null) return;
        PositionLabel.Text = $"{FormatTime(PositionSlider.Value)} / {FormatTime(_player.DurationSeconds)}";
    }

    private static string FormatTime(double seconds)
    {
        if (double.IsNaN(seconds) || seconds < 0) seconds = 0;
        var ts = TimeSpan.FromSeconds(seconds);
        if (ts.TotalHours >= 1) return ts.ToString(@"h\:mm\:ss", CultureInfo.InvariantCulture);
        return ts.ToString(@"m\:ss", CultureInfo.InvariantCulture);
    }
}
