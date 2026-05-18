using System;
using System.ComponentModel;
using System.IO;
using System.Linq;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Windows.ApplicationModel.DataTransfer;
using Windows.Storage;
using Windows.Storage.Pickers;
using VideoCrop.App.Services;
using VideoCrop.App.ViewModels;
using VideoCrop.Core.Encoding;
using VideoCrop.Core.Models;
using VideoCrop.Core.Updater;
using WinRT.Interop;

namespace VideoCrop.App.Views;

public sealed partial class MainView : UserControl
{
    private static readonly string[] AcceptedExtensions =
    {
        ".mp4", ".mkv", ".webm", ".mov", ".avi", ".m4v", ".wmv", ".flv",
        ".ts", ".mts", ".m2ts", ".mpg", ".mpeg", ".3gp", ".ogv",
    };

    public MainViewModel ViewModel { get; }

    public MainView()
    {
        InitializeComponent();
        ViewModel = new MainViewModel(App.Current.Services);
        ViewModel.ToolStatus.PropertyChanged += ToolStatus_PropertyChanged;
        ViewModel.Source.PropertyChanged += Source_PropertyChanged;
        ViewModel.Compression.PropertyChanged += Compression_PropertyChanged;
        ViewModel.Output.PropertyChanged += Output_PropertyChanged;
        ViewModel.Encode.PropertyChanged += Encode_PropertyChanged;

        ViewModel.Cut.PropertyChanged += Cut_PropertyChanged;
        ViewModel.Cut.BoundsChanged += (_, _) => UpdateCutFromVm();
        ViewModel.Crop.PropertyChanged += (_, _) => UpdateCropSummary();
        ViewModel.Crop.CropChanged += (_, _) => ApplyCropToPlayer();
        ViewModel.Update.PropertyChanged += (_, _) => UpdateUpdateBar();
        ViewModel.Resize.PropertyChanged += (_, _) => UpdateResize();
        ViewModel.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(MainViewModel.PipelineSummary))
                PipelineSummaryText.Text = ViewModel.PipelineSummary;
        };
        ViewModel.ResizeToast += (_, msg) => UpdateResize();
        CutTimelineCtrl.StartChanged += (_, t) =>
        {
            ViewModel.Cut.Start = t;
            ThrottledSeek(t);
        };
        CutTimelineCtrl.EndChanged += (_, t) =>
        {
            ViewModel.Cut.End = t;
            ThrottledSeek(t);
        };
        // On drag release, force one definitive exact seek so the frame
        // matches the final handle position even if the trailing live seek
        // was throttled out.
        CutTimelineCtrl.StartCommitted += async (_, t) =>
        {
            if (VideoPaneView.Player is { } player) await player.SeekAsync(t.TotalSeconds);
        };
        CutTimelineCtrl.EndCommitted += async (_, t) =>
        {
            if (VideoPaneView.Player is { } player) await player.SeekAsync(t.TotalSeconds);
        };
        CutTimelineCtrl.PositionRequested += async (_, t) =>
        {
            if (VideoPaneView.Player is { } player) await player.SeekAsync(t.TotalSeconds);
        };

        BindPresets();
        UpdateToolStatus();
        UpdateSource();
        UpdatePresetDescription();
        UpdateOutput();
        UpdateEncode();
        UpdateCutFromVm();
        UpdateCropSummary();
        UpdateResize();

        Loaded += async (_, _) =>
        {
            await ViewModel.InitializeAsync();
            await EnsureToolsAvailableAsync();
            await OfferInstallSetupAsync();
        };
    }

    private async System.Threading.Tasks.Task OfferInstallSetupAsync()
    {
        var settings = App.Current.Services.Settings;
        if (settings.HasPromptedInstallSetup) return;
        // If a prior install already wired up both, just record that and move on.
        if (StartMenuShortcut.Exists() && AppsAndFeaturesRegistration.IsRegistered())
        {
            settings.HasPromptedInstallSetup = true;
            return;
        }

        var dialog = new ContentDialog
        {
            XamlRoot = XamlRoot,
            Title = "Set up VideoCrop on this PC?",
            Content =
                "Adds a Start menu shortcut (Windows search) and registers VideoCrop in Settings → Apps & Features " +
                "for a clean Uninstall later. Decline if this is a temporary or test install.",
            PrimaryButtonText = "Set up",
            CloseButtonText = "Not now",
            DefaultButton = ContentDialogButton.Primary,
        };
        ContentDialogResult result;
        try { result = await dialog.ShowAsync(); }
        catch { return; }

        settings.HasPromptedInstallSetup = true;
        if (result != ContentDialogResult.Primary) return;

        var exe = Environment.ProcessPath;
        if (string.IsNullOrEmpty(exe)) return;
        var icon = Path.Combine(AppContext.BaseDirectory, "Assets", "AppIcon.ico");
        StartMenuShortcut.Create(exe, File.Exists(icon) ? icon : null);
        AppsAndFeaturesRegistration.Register(exe);
    }

    private void BindPresets()
    {
        PresetCombo.ItemsSource = ViewModel.Compression.Presets;
        PresetCombo.DisplayMemberPath = nameof(PresetDefinition.DisplayName);
        PresetCombo.SelectedItem = ViewModel.Compression.SelectedPreset;
    }

    private void ToolStatus_PropertyChanged(object? sender, PropertyChangedEventArgs e) => UpdateToolStatus();
    private void Source_PropertyChanged(object? sender, PropertyChangedEventArgs e) => UpdateSource();
    private void Compression_PropertyChanged(object? sender, PropertyChangedEventArgs e) => UpdatePresetDescription();
    private void Output_PropertyChanged(object? sender, PropertyChangedEventArgs e) => UpdateOutput();
    private void Encode_PropertyChanged(object? sender, PropertyChangedEventArgs e) => UpdateEncode();
    private void Cut_PropertyChanged(object? sender, PropertyChangedEventArgs e) => UpdateCutFromVm();

    private bool _suppressCutBoxUpdate;

    // Throttle live seeks while dragging cut handles — exact seeks decode
    // from the previous keyframe (~50-150 ms each), so firing one per pointer
    // move (60+/sec) backs mpv up and the preview visibly lags. ~10/sec lets
    // mpv keep up while still showing the actual frame at the handle.
    private DateTime _lastCutSeek = DateTime.MinValue;
    private static readonly TimeSpan CutSeekInterval = TimeSpan.FromMilliseconds(100);

    private void ThrottledSeek(TimeSpan t)
    {
        if (VideoPaneView.Player is not { } player) return;
        var now = DateTime.UtcNow;
        if (now - _lastCutSeek < CutSeekInterval) return;
        _lastCutSeek = now;
        _ = player.SeekAsync(t.TotalSeconds);
    }

    private void UpdateCutFromVm()
    {
        var cut = ViewModel.Cut;
        CutTimelineCtrl.Start = cut.Start;
        CutTimelineCtrl.End = cut.End;
        CutTimelineCtrl.Duration = cut.SourceDuration;
        CutDurationText.Text = FormatTimeSpan(cut.Duration) + " (of " + FormatTimeSpan(cut.SourceDuration) + ")";

        if (!_suppressCutBoxUpdate)
        {
            CutStartBox.Text = FormatTimeSpan(cut.Start);
            CutEndBox.Text = FormatTimeSpan(cut.End);
        }

        // Constrain the playback slider so the user cannot scrub outside
        // the active cut range. When cut is full-range, this is just [0..duration].
        VideoPaneView.SetSeekBounds(cut.Start.TotalSeconds, cut.End.TotalSeconds);
    }

    private static string FormatTimeSpan(TimeSpan ts)
    {
        if (ts.TotalHours >= 1)
            return ts.ToString(@"h\:mm\:ss\.fff", System.Globalization.CultureInfo.InvariantCulture);
        return ts.ToString(@"m\:ss\.fff", System.Globalization.CultureInfo.InvariantCulture);
    }

    private static bool TryParseTimeSpan(string text, out TimeSpan result)
    {
        result = TimeSpan.Zero;
        if (string.IsNullOrWhiteSpace(text)) return false;
        var s = text.Trim();
        // Accept h:mm:ss.fff, m:ss.fff, ss.fff
        if (TimeSpan.TryParseExact(s, new[] { @"h\:mm\:ss\.fff", @"h\:mm\:ss", @"m\:ss\.fff", @"m\:ss", @"s\.fff", @"s" },
            System.Globalization.CultureInfo.InvariantCulture, out result))
            return true;
        if (double.TryParse(s, System.Globalization.NumberStyles.Float, System.Globalization.CultureInfo.InvariantCulture, out var seconds))
        {
            result = TimeSpan.FromSeconds(seconds);
            return true;
        }
        return false;
    }

    private void OnCutStartLostFocus(object sender, RoutedEventArgs e)
    {
        if (!TryParseTimeSpan(CutStartBox.Text, out var ts)) return;
        _suppressCutBoxUpdate = true;
        ViewModel.Cut.Start = ts;
        _suppressCutBoxUpdate = false;
        UpdateCutFromVm();
    }

    private void OnCutEndLostFocus(object sender, RoutedEventArgs e)
    {
        if (!TryParseTimeSpan(CutEndBox.Text, out var ts)) return;
        _suppressCutBoxUpdate = true;
        ViewModel.Cut.End = ts;
        _suppressCutBoxUpdate = false;
        UpdateCutFromVm();
    }

    private async void OnCutSetStart(object sender, RoutedEventArgs e)
    {
        ViewModel.Cut.Player = VideoPaneView.Player;
        await ViewModel.Cut.SetStartFromPlayheadAsync();
    }

    private async void OnCutSetEnd(object sender, RoutedEventArgs e)
    {
        ViewModel.Cut.Player = VideoPaneView.Player;
        await ViewModel.Cut.SetEndFromPlayheadAsync();
    }

    private void OnCutAccurateModeChanged(object sender, SelectionChangedEventArgs e)
    {
        // Index 0 = Fast (keyframe-snapped), 1 = Accurate (frame-exact).
        ViewModel.Cut.Accurate = CutAccurateButtons.SelectedIndex == 1;
    }

    private void OnCutReset(object sender, RoutedEventArgs e)
    {
        ViewModel.Cut.Reset();
    }

    private void UpdateCropSummary()
    {
        CropSummary.Text = ViewModel.Crop.Summary;
    }

    private void ApplyCropToPlayer()
    {
        if (VideoPaneView.Player is not { } player) return;
        var crop = ViewModel.Crop.Crop;
        var mpvCrop = crop is null
            ? null
            : $"{crop.Width}x{crop.Height}+{crop.X}+{crop.Y}";
        _ = player.SetVideoCropAsync(mpvCrop);
    }

    private async void OnEditCropClicked(object sender, RoutedEventArgs e)
    {
        var info = ViewModel.Source.VideoInfo;
        if (info is null) return;

        string? screenshotPath = null;
        if (VideoPaneView.Player is { } player)
        {
            screenshotPath = System.IO.Path.Combine(System.IO.Path.GetTempPath(),
                "videocrop_crop_" + Guid.NewGuid().ToString("N") + ".png");
            var ok = await player.ScreenshotToFileAsync(screenshotPath);
            if (!ok) screenshotPath = null;
        }

        var dialog = new CropDialog(info.Width, info.Height, ViewModel.Crop.Crop, screenshotPath)
        {
            XamlRoot = this.XamlRoot,
        };
        var result = await dialog.ShowAsync();

        if (screenshotPath is not null)
        {
            try { System.IO.File.Delete(screenshotPath); } catch { }
        }

        if (result == ContentDialogResult.Primary && dialog.Result is not null)
        {
            ViewModel.Crop.Crop = dialog.Result;
        }
        else if (dialog.ResetRequested)
        {
            ViewModel.Crop.Crop = null;
        }
    }

    private void OnResetCropClicked(object sender, RoutedEventArgs e)
    {
        ViewModel.Crop.Crop = null;
    }

    private void UpdateUpdateBar()
    {
        var u = ViewModel.Update;
        UpdateBar.IsOpen = u.IsActionable;
        UpdateBar.Title = u.Info is null
            ? "Update"
            : $"Update {u.Info.TagName}";
        UpdateBar.Message = u.StatusMessage;
        UpdateBar.Severity = u.State switch
        {
            UpdateState.Failed => Microsoft.UI.Xaml.Controls.InfoBarSeverity.Warning,
            UpdateState.Staged => Microsoft.UI.Xaml.Controls.InfoBarSeverity.Success,
            _ => Microsoft.UI.Xaml.Controls.InfoBarSeverity.Informational,
        };
        UpdateRestartButton.Visibility = u.State == UpdateState.Staged
            ? Visibility.Visible
            : Visibility.Collapsed;
    }

    private void OnUpdateRestartClicked(object sender, RoutedEventArgs e)
    {
        ViewModel.Update.TryApplyAndRestart();
    }

    private async System.Threading.Tasks.Task EnsureToolsAvailableAsync()
    {
        if (!ViewModel.ToolSetup.ToolsMissing) return;

        var dialog = new ToolSetupDialog(ViewModel.ToolSetup) { XamlRoot = XamlRoot };
        var result = await dialog.ShowAsync();

        // Close = user picked "Quit" without downloading. Refresh status; the
        // main UI will stay in a degraded state with the Tools section warning
        // and Process disabled until they restart and complete setup.
        await ViewModel.ToolStatus.RefreshAsync();

        if (!dialog.DownloadSucceeded && result == ContentDialogResult.None)
        {
            // User cancelled — leave the app in degraded state.
        }
    }

    private bool _suppressResizeBoxes;

    private void UpdateResize()
    {
        var r = ViewModel.Resize;
        ResizeInputDisplay.Text = "Input: " + r.InputDisplay;
        ResizeEnabledSwitch.IsOn = r.Enabled;
        AspectLockToggle.IsChecked = r.AspectLocked;
        _suppressResizeBoxes = true;
        ResizeWidthBox.Value = r.Width;
        ResizeHeightBox.Value = r.Height;
        _suppressResizeBoxes = false;
        ResizeMismatchBar.IsOpen = r.MismatchWarning is not null;
        ResizeMismatchBar.Message = r.MismatchWarning ?? "";
        PipelineSummaryText.Text = ViewModel.PipelineSummary;
    }

    private void OnResizeEnabledToggled(object sender, RoutedEventArgs e)
    {
        ViewModel.Resize.Enabled = ResizeEnabledSwitch.IsOn;
    }

    private void OnAspectLockToggled(object sender, RoutedEventArgs e)
    {
        ViewModel.Resize.AspectLocked = AspectLockToggle.IsChecked == true;
    }

    private void OnResizeWidthChanged(NumberBox sender, NumberBoxValueChangedEventArgs args)
    {
        if (_suppressResizeBoxes) return;
        if (double.IsNaN(args.NewValue)) return;
        ViewModel.Resize.Width = (int)args.NewValue;
    }

    private void OnResizeHeightChanged(NumberBox sender, NumberBoxValueChangedEventArgs args)
    {
        if (_suppressResizeBoxes) return;
        if (double.IsNaN(args.NewValue)) return;
        ViewModel.Resize.Height = (int)args.NewValue;
    }

    private void OnResizePreset2160(object sender, RoutedEventArgs e) => ApplyResizePreset(2160);
    private void OnResizePreset1440(object sender, RoutedEventArgs e) => ApplyResizePreset(1440);
    private void OnResizePreset1080(object sender, RoutedEventArgs e) => ApplyResizePreset(1080);
    private void OnResizePreset720(object sender, RoutedEventArgs e) => ApplyResizePreset(720);
    private void OnResizePreset480(object sender, RoutedEventArgs e) => ApplyResizePreset(480);

    private void ApplyResizePreset(int height)
    {
        ViewModel.Resize.Enabled = true;
        ViewModel.Resize.AspectLocked = true;
        ViewModel.Resize.ApplyPresetHeight(height);
    }

    private bool _suppressAdvancedSync;

    private void OnCompressionEnabledToggled(object sender, RoutedEventArgs e)
    {
        // Fires during InitializeComponent (when IsOn="True" in XAML changes
        // from the default false), before ViewModel is constructed. Skip until
        // construction is complete; the constructor's UpdatePresetDescription
        // call will sync state explicitly.
        if (ViewModel is null) return;
        var on = CompressionEnabledToggle.IsOn;
        ViewModel.Compression.Enabled = on;
        CompressionDetailsPanel.Visibility = on ? Visibility.Visible : Visibility.Collapsed;
        StreamCopyNote.Visibility = on ? Visibility.Collapsed : Visibility.Visible;
        UpdatePresetDescription();
    }

    private void OnAdvancedToggled(object sender, RoutedEventArgs e)
    {
        ViewModel.Compression.AdvancedEnabled = AdvancedToggle.IsOn;
        AdvancedPanel.Visibility = AdvancedToggle.IsOn ? Visibility.Visible : Visibility.Collapsed;
        EasyPanel.Visibility = AdvancedToggle.IsOn ? Visibility.Collapsed : Visibility.Visible;
        if (AdvancedToggle.IsOn) SyncAdvancedUiFromVm();
    }

    private void SyncAdvancedUiFromVm()
    {
        _suppressAdvancedSync = true;
        var c = ViewModel.Compression;
        SelectByTag(AdvCodecCombo, c.Codec.ToString());
        RebuildSpeedCombo(c.Codec);
        SelectByTag(AdvSpeedCombo, c.Speed.ToString());
        SelectByTag(AdvRateModeCombo, c.RateMode.ToString());
        SelectByTag(AdvAudioCodecCombo, c.AudioCodec.ToString());
        SelectByTag(AdvPixelFormatCombo, c.PixelFormat.ToString());
        AdvCrfBox.Value = c.Crf;
        AdvBitrateBox.Value = c.TargetBitrateBps / 1000.0;
        AdvTargetSizeBox.Value = c.TargetSizeMb;
        SelectAudioBitrate(AdvAudioBitrateCombo, c.AudioBitrateKbps);
        AdvBitrateBox.Visibility = c.RateMode == CompressionRateMode.TargetBitrate ? Visibility.Visible : Visibility.Collapsed;
        AdvTargetSizeBox.Visibility = c.RateMode == CompressionRateMode.TargetSize ? Visibility.Visible : Visibility.Collapsed;
        AdvCrfBox.Visibility = c.RateMode == CompressionRateMode.Crf ? Visibility.Visible : Visibility.Collapsed;
        _suppressAdvancedSync = false;
    }

    private static void SelectByTag(ComboBox combo, string tag)
    {
        foreach (var item in combo.Items)
        {
            if (item is ComboBoxItem cbi && (cbi.Tag as string) == tag)
            {
                combo.SelectedItem = cbi;
                return;
            }
        }
    }

    private static void SelectAudioBitrate(ComboBox combo, int kbps)
    {
        var s = kbps.ToString(System.Globalization.CultureInfo.InvariantCulture);
        SelectByTag(combo, s);
    }

    private static T? GetTagEnum<T>(ComboBox combo) where T : struct, Enum
    {
        if (combo.SelectedItem is ComboBoxItem cbi && cbi.Tag is string tag && Enum.TryParse<T>(tag, out var value))
            return value;
        return null;
    }

    private void OnAdvCodecChanged(object sender, SelectionChangedEventArgs e)
    {
        if (_suppressAdvancedSync) return;
        if (GetTagEnum<VideoCodec>(AdvCodecCombo) is { } c)
        {
            ViewModel.Compression.Codec = c;
            // libsvtav1 uses numeric presets, libx264/x265 use named ones —
            // swap the dropdown items so the labels match what ffmpeg will
            // actually emit.
            _suppressAdvancedSync = true;
            RebuildSpeedCombo(c);
            SelectByTag(AdvSpeedCombo, ViewModel.Compression.Speed.ToString());
            _suppressAdvancedSync = false;
        }
    }

    private static readonly (string Label, SpeedPreset Value)[] X264SpeedItems =
    {
        ("ultrafast", SpeedPreset.UltraFast),
        ("superfast", SpeedPreset.SuperFast),
        ("veryfast",  SpeedPreset.VeryFast),
        ("faster",    SpeedPreset.Faster),
        ("fast",      SpeedPreset.Fast),
        ("medium",    SpeedPreset.Medium),
        ("slow",      SpeedPreset.Slow),
        ("slower",    SpeedPreset.Slower),
        ("veryslow",  SpeedPreset.VerySlow),
    };

    // libsvtav1 presets: 0 = slowest/best, 13 = fastest/lowest quality. The
    // numbers shown here are what ffmpeg actually receives via -preset.
    private static readonly (string Label, SpeedPreset Value)[] Av1SpeedItems =
    {
        ("3 — slowest",  SpeedPreset.VerySlow),
        ("4",            SpeedPreset.Slower),
        ("5",            SpeedPreset.Slow),
        ("6 — default",  SpeedPreset.Medium),
        ("8",            SpeedPreset.Fast),
        ("9",            SpeedPreset.Faster),
        ("10",           SpeedPreset.VeryFast),
        ("11",           SpeedPreset.SuperFast),
        ("12 — fastest", SpeedPreset.UltraFast),
    };

    private void RebuildSpeedCombo(VideoCodec codec)
    {
        var items = codec == VideoCodec.Av1 ? Av1SpeedItems : X264SpeedItems;
        AdvSpeedCombo.Header = codec == VideoCodec.Av1 ? "Preset (libsvtav1)" : "Speed";
        AdvSpeedCombo.Items.Clear();
        foreach (var (label, value) in items)
        {
            AdvSpeedCombo.Items.Add(new ComboBoxItem { Content = label, Tag = value.ToString() });
        }
    }

    private void OnAdvSpeedChanged(object sender, SelectionChangedEventArgs e)
    {
        if (_suppressAdvancedSync) return;
        if (GetTagEnum<SpeedPreset>(AdvSpeedCombo) is { } s) ViewModel.Compression.Speed = s;
    }

    private void OnAdvRateModeChanged(object sender, SelectionChangedEventArgs e)
    {
        if (_suppressAdvancedSync) return;
        if (GetTagEnum<CompressionRateMode>(AdvRateModeCombo) is { } m)
        {
            ViewModel.Compression.RateMode = m;
            AdvCrfBox.Visibility = m == CompressionRateMode.Crf ? Visibility.Visible : Visibility.Collapsed;
            AdvBitrateBox.Visibility = m == CompressionRateMode.TargetBitrate ? Visibility.Visible : Visibility.Collapsed;
            AdvTargetSizeBox.Visibility = m == CompressionRateMode.TargetSize ? Visibility.Visible : Visibility.Collapsed;
        }
    }

    private void OnAdvCrfChanged(NumberBox sender, NumberBoxValueChangedEventArgs args)
    {
        if (_suppressAdvancedSync || double.IsNaN(args.NewValue)) return;
        ViewModel.Compression.Crf = (int)args.NewValue;
    }

    private void OnAdvBitrateChanged(NumberBox sender, NumberBoxValueChangedEventArgs args)
    {
        if (_suppressAdvancedSync || double.IsNaN(args.NewValue)) return;
        ViewModel.Compression.TargetBitrateBps = (long)(args.NewValue * 1000);
    }

    private void OnAdvTargetSizeChanged(NumberBox sender, NumberBoxValueChangedEventArgs args)
    {
        if (_suppressAdvancedSync || double.IsNaN(args.NewValue)) return;
        ViewModel.Compression.TargetSizeMb = (long)args.NewValue;
    }

    private void OnAdvAudioCodecChanged(object sender, SelectionChangedEventArgs e)
    {
        if (_suppressAdvancedSync) return;
        if (GetTagEnum<AudioCodec>(AdvAudioCodecCombo) is { } a) ViewModel.Compression.AudioCodec = a;
    }

    private void OnAdvAudioBitrateChanged(object sender, SelectionChangedEventArgs e)
    {
        if (_suppressAdvancedSync) return;
        if (AdvAudioBitrateCombo.SelectedItem is ComboBoxItem cbi && cbi.Tag is string tag && int.TryParse(tag, out var bps))
        {
            ViewModel.Compression.AudioBitrateKbps = bps;
        }
    }


    private void OnAdvPixelFormatChanged(object sender, SelectionChangedEventArgs e)
    {
        if (_suppressAdvancedSync) return;
        if (GetTagEnum<PixelFormat>(AdvPixelFormatCombo) is { } p) ViewModel.Compression.PixelFormat = p;
    }

    private void OnAcceleratorOpen(Microsoft.UI.Xaml.Input.KeyboardAccelerator sender, Microsoft.UI.Xaml.Input.KeyboardAcceleratorInvokedEventArgs args)
    {
        OnOpenFileClicked(this, new RoutedEventArgs());
        args.Handled = true;
    }

    private void OnAcceleratorProcess(Microsoft.UI.Xaml.Input.KeyboardAccelerator sender, Microsoft.UI.Xaml.Input.KeyboardAcceleratorInvokedEventArgs args)
    {
        if (ViewModel.Source.HasVideo && !ViewModel.Encode.IsRunning)
        {
            OnProcessClicked(this, new RoutedEventArgs());
            args.Handled = true;
        }
    }

    private async void OnAcceleratorPlayPause(Microsoft.UI.Xaml.Input.KeyboardAccelerator sender, Microsoft.UI.Xaml.Input.KeyboardAcceleratorInvokedEventArgs args)
    {
        if (VideoPaneView.Player is { } player) await player.TogglePauseAsync();
        args.Handled = true;
    }

    private async void OnAcceleratorSetStart(Microsoft.UI.Xaml.Input.KeyboardAccelerator sender, Microsoft.UI.Xaml.Input.KeyboardAcceleratorInvokedEventArgs args)
    {
        ViewModel.Cut.Player = VideoPaneView.Player;
        await ViewModel.Cut.SetStartFromPlayheadAsync();
        args.Handled = true;
    }

    private async void OnAcceleratorSetEnd(Microsoft.UI.Xaml.Input.KeyboardAccelerator sender, Microsoft.UI.Xaml.Input.KeyboardAcceleratorInvokedEventArgs args)
    {
        ViewModel.Cut.Player = VideoPaneView.Player;
        await ViewModel.Cut.SetEndFromPlayheadAsync();
        args.Handled = true;
    }

    private void UpdateToolStatus()
    {
        FfmpegStatusText.Text = ViewModel.ToolStatus.FfmpegStatus;
        FfprobeStatusText.Text = ViewModel.ToolStatus.FfprobeStatus;
        MpvStatusText.Text = ViewModel.ToolStatus.MpvStatus;
        ToolsMissingBar.IsOpen = !ViewModel.ToolStatus.AllFound;
    }

    private void UpdateSource()
    {
        var src = ViewModel.Source;
        SourceLoading.IsActive = src.IsLoading;
        var hasError = !src.HasVideo && !string.IsNullOrEmpty(src.StatusMessage) && !src.IsLoading
            && src.StatusMessage.StartsWith("Failed", StringComparison.Ordinal);
        SourceErrorBar.IsOpen = hasError;
        SourceErrorBar.Message = hasError ? src.StatusMessage : "";

        if (src.HasVideo)
        {
            SourceEmpty.Visibility = Visibility.Collapsed;
            SourceDetails.Visibility = Visibility.Visible;
            SourceFileName.Text = src.FileName;
            SourceDuration.Text = src.DurationDisplay;
            SourceResolution.Text = src.ResolutionDisplay;
            SourceCodec.Text = src.CodecDisplay;
            SourceFps.Text = src.FpsDisplay;
            SourceBitrate.Text = src.BitrateDisplay;
            SourceAudio.Text = src.AudioDisplay;
        }
        else
        {
            SourceEmpty.Visibility = Visibility.Visible;
            SourceDetails.Visibility = Visibility.Collapsed;
            SourceEmpty.Text = src.IsLoading ? src.StatusMessage : "No video loaded.";
        }

        ProcessButton.IsEnabled = src.HasVideo && !ViewModel.Encode.IsRunning;
    }

    private void UpdatePresetDescription()
    {
        var comp = ViewModel.Compression;
        PresetDescription.Text = comp.Description;
        PresetWarningBar.IsOpen = comp.HasWarning;
        PresetWarningBar.Message = comp.Warning ?? "";
    }

    private void UpdateOutput()
    {
        OutputDirectoryBox.Text = ViewModel.Output.OutputDirectory ?? "";
        FilenamePreview.Text = string.IsNullOrEmpty(ViewModel.Output.FilenamePreview)
            ? "(will compute on Process)"
            : ViewModel.Output.FilenamePreview;
    }

    private void UpdateEncode()
    {
        var enc = ViewModel.Encode;
        EncodeProgressBar.Value = enc.Progress;
        ProcessButton.IsEnabled = ViewModel.Source.HasVideo && !enc.IsRunning;
        CancelButton.IsEnabled = enc.IsRunning;
        if (enc.IsRunning)
        {
            var eta = enc.Eta;
            EncodeStatusText.Text = eta is null
                ? $"frame {enc.Frame} @ {enc.Fps:0.#} fps — {enc.Speed:0.##}×"
                : $"frame {enc.Frame} @ {enc.Fps:0.#} fps — {enc.Speed:0.##}× — ETA {eta:mm\\:ss}";
        }
        else
        {
            EncodeStatusText.Text = enc.StatusMessage;
        }

        if (!string.IsNullOrEmpty(enc.ErrorTail))
        {
            EncodeErrorExpander.Visibility = Visibility.Visible;
            EncodeErrorText.Text = enc.ErrorTail;
        }
    }

    private async System.Threading.Tasks.Task LoadFileAsync(string path)
    {
        await ViewModel.Source.LoadAsync(path);
        if (ViewModel.Source.HasVideo)
        {
            App.Current.Services.RecentFiles.Add(path);
            await VideoPaneView.LoadVideoAsync(path);
            if (VideoPaneView.Player is { } player)
            {
                ViewModel.Cut.Player = player;
                player.PropertyChanged -= Player_PositionChanged;
                player.PropertyChanged += Player_PositionChanged;
            }
        }
    }

    private void Player_PositionChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (e.PropertyName != nameof(PlayerViewModel.PositionSeconds)) return;
        if (VideoPaneView.Player is { } player)
        {
            CutTimelineCtrl.Position = TimeSpan.FromSeconds(player.PositionSeconds);
        }
    }

    private void OnDragOver(object sender, DragEventArgs e)
    {
        if (e.DataView.Contains(StandardDataFormats.StorageItems))
        {
            e.AcceptedOperation = DataPackageOperation.Copy;
            e.DragUIOverride.Caption = "Open video";
            e.DragUIOverride.IsCaptionVisible = true;
            e.DragUIOverride.IsContentVisible = true;
            e.Handled = true;
        }
    }

    private async void OnDrop(object sender, DragEventArgs e)
    {
        if (!e.DataView.Contains(StandardDataFormats.StorageItems)) return;
        var def = e.GetDeferral();
        try
        {
            var items = await e.DataView.GetStorageItemsAsync();
            var file = items.OfType<StorageFile>().FirstOrDefault(f => IsAccepted(f.Path));
            if (file is not null)
            {
                await LoadFileAsync(file.Path);
            }
        }
        finally
        {
            def.Complete();
        }
    }

    private async void OnOpenFileClicked(object sender, RoutedEventArgs e)
    {
        var picker = new FileOpenPicker
        {
            ViewMode = PickerViewMode.List,
            SuggestedStartLocation = PickerLocationId.VideosLibrary,
        };
        foreach (var ext in AcceptedExtensions) picker.FileTypeFilter.Add(ext);

        var hwnd = WindowNative.GetWindowHandle(App.Current.MainWindow);
        InitializeWithWindow.Initialize(picker, hwnd);

        var file = await picker.PickSingleFileAsync();
        if (file is null) return;

        await LoadFileAsync(file.Path);
    }

    private void OnPresetSelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (PresetCombo.SelectedItem is PresetDefinition p)
        {
            ViewModel.Compression.SelectedPreset = p;
        }
    }

    private void OnOutputModeChanged(object sender, SelectionChangedEventArgs e)
    {
        ViewModel.Output.Mode = OutputModeButtons.SelectedIndex switch
        {
            1 => OutputMode.AudioOnly,
            2 => OutputMode.VideoOnly,
            _ => OutputMode.Everything,
        };
    }

    private async void OnBrowseOutputClicked(object sender, RoutedEventArgs e)
    {
        var picker = new FolderPicker { SuggestedStartLocation = PickerLocationId.VideosLibrary };
        picker.FileTypeFilter.Add("*");

        var hwnd = WindowNative.GetWindowHandle(App.Current.MainWindow);
        InitializeWithWindow.Initialize(picker, hwnd);

        var folder = await picker.PickSingleFolderAsync();
        if (folder is null) return;

        ViewModel.Output.OutputDirectory = folder.Path;
    }

    private async void OnProcessClicked(object sender, RoutedEventArgs e)
    {
        var job = ViewModel.BuildEncodeJob();
        if (job is null) return;

        await ViewModel.Encode.RunAsync(job);
    }

    private void OnCancelClicked(object sender, RoutedEventArgs e)
    {
        ViewModel.Encode.Cancel();
    }

    private static bool IsAccepted(string path)
    {
        var ext = Path.GetExtension(path);
        if (string.IsNullOrEmpty(ext)) return false;
        return AcceptedExtensions.Contains(ext, StringComparer.OrdinalIgnoreCase);
    }
}
