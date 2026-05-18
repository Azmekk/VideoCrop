using System;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media.Imaging;
using VideoCrop.App.Controls;
using VideoCrop.Core.Models;

namespace VideoCrop.App.Views;

public sealed partial class CropDialog : ContentDialog
{
    public CropSpec? Result { get; private set; }
    public bool ResetRequested { get; private set; }

    public CropDialog(int sourceWidth, int sourceHeight, CropSpec? initial, string? screenshotPath)
    {
        InitializeComponent();

        // Give the overlay an explicit pixel size matching the source; the
        // Viewbox parent scales it uniformly to fit the dialog while preserving
        // aspect, so a 16:9 video doesn't get crushed into a square.
        Overlay.Width = sourceWidth;
        Overlay.Height = sourceHeight;
        Overlay.SetSource(sourceWidth, sourceHeight);
        Overlay.Crop = initial ?? new CropSpec(0, 0, sourceWidth, sourceHeight);

        // Size the dialog content to fit the source aspect inside a safe content
        // box. We must cap BOTH axes — the ContentDialog adds ~150px of chrome
        // (title + action buttons + padding), and the dialog still has to fit
        // inside the host window. For a tall (e.g. 9:16) or square source,
        // capping only the width would blow up the height and clip the bottom.
        var aspect = sourceHeight > 0 ? (double)sourceWidth / sourceHeight : 16.0 / 9.0;
        const double maxContentWidth = 1080;
        const double maxContentHeight = 540;
        const double chromeWithinGrid = 70; // aspect picker + info text + row spacing

        double viewboxW, viewboxH;
        if (maxContentWidth / aspect <= maxContentHeight)
        {
            viewboxW = maxContentWidth;
            viewboxH = maxContentWidth / aspect;
        }
        else
        {
            viewboxH = maxContentHeight;
            viewboxW = maxContentHeight * aspect;
        }

        RootGrid.Width = viewboxW;
        RootGrid.Height = viewboxH + chromeWithinGrid;

        if (screenshotPath is not null)
        {
            try { Overlay.BackgroundImage = new BitmapImage(new Uri(screenshotPath)); }
            catch { /* ignore — overlay still functional */ }
        }

        AspectCombo.SelectedIndex = 0;

        UpdateInfo(Overlay.Crop);
        Overlay.CropChanged += (_, spec) => UpdateInfo(spec);

        PrimaryButtonClick += (_, _) => { Result = Overlay.Crop; };
        SecondaryButtonClick += (_, args) =>
        {
            ResetRequested = true;
            Result = null;
            // Keep dialog open? No — secondary closes by default. Set Cancel = false to close.
            args.Cancel = false;
        };
    }

    private void UpdateInfo(CropSpec spec)
    {
        CropInfo.Text = $"Crop: {spec.Width}×{spec.Height} from {Overlay.SourceWidth}×{Overlay.SourceHeight} at ({spec.X},{spec.Y})";
    }

    private void OnAspectChanged(object sender, SelectionChangedEventArgs e)
    {
        if (AspectCombo.SelectedItem is ComboBoxItem item && item.Tag is string tag &&
            Enum.TryParse<CropOverlay.AspectMode>(tag, out var mode))
        {
            Overlay.Aspect = mode;
        }
    }
}
