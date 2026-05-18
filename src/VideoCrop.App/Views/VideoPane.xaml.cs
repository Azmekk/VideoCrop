using System;
using System.ComponentModel;
using System.Globalization;
using System.Threading.Tasks;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Media;
using VideoCrop.App.Interop;
using VideoCrop.App.ViewModels;
using Windows.Foundation;
using WinRT.Interop;

namespace VideoCrop.App.Views;

public sealed partial class VideoPane : UserControl
{
    private PlayerViewModel? _player;
    private VideoHostWindow? _hostWindow;
    private bool _seekingByUser;
    private bool _suppressSliderEvent;
    private bool _suppressVolumeSliderEvent;
    private double _minBoundSec;
    private double _maxBoundSec;

    // Last bounds we pushed to the child window (in physical pixels). Used to
    // skip redundant SetWindowPos calls during the LayoutUpdated firehose.
    private Rect _lastBounds = Rect.Empty;
    private bool _videoVisible;

    public VideoPane()
    {
        InitializeComponent();
        Loaded += OnLoaded;
        Unloaded += OnUnloaded;
        SizeChanged += (_, _) => UpdateHostBounds();
        LayoutUpdated += (_, _) => UpdateHostBounds();
    }

    public PlayerViewModel? Player => _player;

    public void SetSeekBounds(double minSec, double maxSec)
    {
        if (maxSec <= minSec) maxSec = minSec + 0.01;
        _minBoundSec = minSec;
        _maxBoundSec = maxSec;
        _suppressSliderEvent = true;
        PositionSlider.Minimum = minSec;
        PositionSlider.Maximum = maxSec;
        _suppressSliderEvent = false;
        UpdatePositionLabel();
    }

    public async Task LoadVideoAsync(string path)
    {
        if (_player is null) return;
        _videoVisible = true;
        UpdateHostBounds(force: true);
        await _player.LoadFileAsync(path);
        UpdateEmptyState();
        TransportBar.Visibility = _player.IsReady ? Visibility.Visible : Visibility.Collapsed;
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        // Parent the child window to the WinUI main window's HWND. The HWND
        // lives as long as this UserControl does — mpv may come and go but
        // the host window persists, so respawned mpv re-attaches to the same
        // surface.
        try
        {
            var parentHwnd = WindowNative.GetWindowHandle(App.Current.MainWindow);
            _hostWindow = new VideoHostWindow(parentHwnd);
            // A freshly-shown WinUI 3 window doesn't have DWM fully wired up
            // for compositing a child swapchain — mpv renders into its
            // swapchain but DWM keeps the parent frame and never pulls those
            // pixels onto the desktop until the user moves the window. A
            // 1px parent move+back here primes DWM before mpv spawns, so
            // the first loaded frame is visible immediately.
            _hostWindow.NudgeParentPosition();
        }
        catch
        {
            // If embedding setup fails we still want the rest of the UI usable;
            // mpv will fall back to standalone mode when VideoHwnd is zero.
            _hostWindow = null;
        }

        _player = new PlayerViewModel(App.Current.ToolLocator, App.Current.LoggerFactory)
        {
            VideoHwnd = _hostWindow?.Handle ?? IntPtr.Zero,
        };
        _player.PropertyChanged += Player_PropertyChanged;
        UpdateEmptyState();
    }

    private async void OnUnloaded(object sender, RoutedEventArgs e)
    {
        if (_player is not null) await _player.DisposeAsync();
        _player = null;
        _hostWindow?.Dispose();
        _hostWindow = null;
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
                if (_maxBoundSec <= 0)
                {
                    SetSeekBounds(0, Math.Max(0.01, _player.DurationSeconds));
                }
                UpdatePositionLabel();
                break;
            case nameof(PlayerViewModel.PositionSeconds):
                if (!_seekingByUser)
                {
                    _suppressSliderEvent = true;
                    var clamped = Math.Clamp(_player.PositionSeconds, PositionSlider.Minimum, PositionSlider.Maximum);
                    PositionSlider.Value = clamped;
                    _suppressSliderEvent = false;
                    UpdatePositionLabel();
                }
                break;
            case nameof(PlayerViewModel.IsReady):
                TransportBar.Visibility = _player.IsReady ? Visibility.Visible : Visibility.Collapsed;
                if (!_player.IsReady)
                {
                    // mpv died — hide the host so the empty-state overlay is
                    // visible. The HWND itself stays alive for the next spawn.
                    _videoVisible = false;
                    _hostWindow?.Hide();
                }
                else if (!string.IsNullOrEmpty(_player.CurrentFile))
                {
                    _videoVisible = true;
                    UpdateHostBounds(force: true);
                }
                UpdateEmptyState();
                break;
            case nameof(PlayerViewModel.CurrentFile):
                UpdateEmptyState();
                break;
            case nameof(PlayerViewModel.Volume):
                _suppressVolumeSliderEvent = true;
                VolumeSlider.Value = _player.Volume;
                _suppressVolumeSliderEvent = false;
                UpdateVolumeIcon();
                break;
        }
    }

    private void OnVolumeSliderChanged(object sender, RangeBaseValueChangedEventArgs e)
    {
        if (_suppressVolumeSliderEvent) return;
        if (_player is null) return;
        _ = _player.SetVolumeAsync(e.NewValue);
        UpdateVolumeIcon();
    }

    private void UpdateVolumeIcon()
    {
        // Segoe Fluent Icons: E74F mute, E993 low, E994 medium, E995 high.
        var v = VolumeSlider.Value;
        VolumeIcon.Glyph = v <= 0     ? ""
                         : v <  33    ? ""
                         : v <  66    ? ""
                         :              "";
    }

    private void UpdateEmptyState()
    {
        if (_player is null) return;
        var hasFile = !string.IsNullOrEmpty(_player.CurrentFile);
        if (!hasFile)
        {
            EmptyOverlay.Text = _player.HasMpv
                ? "Drop a video here, or click Open File."
                : "mpv not found. Restart the app to download the required tools.";
            EmptyOverlayPanel.Visibility = Visibility.Visible;
            ReopenPlayerButton.Visibility = Visibility.Collapsed;
            return;
        }

        if (_player.IsReady)
        {
            // mpv child window covers the placeholder; XAML overlay would only
            // peek through at the edges, so hide it entirely.
            EmptyOverlayPanel.Visibility = Visibility.Collapsed;
            ReopenPlayerButton.Visibility = Visibility.Collapsed;
        }
        else
        {
            EmptyOverlay.Text = "Player crashed.";
            EmptyOverlayPanel.Visibility = Visibility.Visible;
            ReopenPlayerButton.Visibility = Visibility.Visible;
        }
    }

    private async void OnReopenPlayerClick(object sender, RoutedEventArgs e)
    {
        if (_player is null) return;
        ReopenPlayerButton.IsEnabled = false;
        try { await _player.ReopenAsync(); }
        finally { ReopenPlayerButton.IsEnabled = true; }
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

    /// <summary>
    /// Reposition the child HWND over the XAML placeholder. Converts XAML
    /// logical coords (relative to the root visual) to physical pixels using
    /// the parent window's current DPI, then SetWindowPos. Skips the call
    /// when bounds haven't changed since LayoutUpdated fires constantly.
    /// </summary>
    private void UpdateHostBounds(bool force = false)
    {
        if (_hostWindow is null) return;
        if (!_videoVisible) return;
        if (VideoPlaceholder.ActualWidth <= 0 || VideoPlaceholder.ActualHeight <= 0) return;

        Rect logical;
        try
        {
            var transform = VideoPlaceholder.TransformToVisual(null);
            logical = transform.TransformBounds(
                new Rect(0, 0, VideoPlaceholder.ActualWidth, VideoPlaceholder.ActualHeight));
        }
        catch
        {
            return; // Not yet attached to the visual tree.
        }

        var dpi = _hostWindow.GetParentDpi();
        if (dpi == 0) dpi = 96;
        var scale = dpi / 96.0;

        var physical = new Rect(
            Math.Round(logical.X * scale),
            Math.Round(logical.Y * scale),
            Math.Round(logical.Width * scale),
            Math.Round(logical.Height * scale));

        if (!force && physical.Equals(_lastBounds)) return;
        _lastBounds = physical;

        _hostWindow.SetBounds(
            (int)physical.X, (int)physical.Y,
            (int)physical.Width, (int)physical.Height);
    }
}
