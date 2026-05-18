using System.ComponentModel;
using Microsoft.UI.Xaml.Controls;
using VideoCrop.App.ViewModels;

namespace VideoCrop.App.Views;

public sealed partial class ToolSetupDialog : ContentDialog
{
    public ToolSetupViewModel ViewModel { get; }

    public bool DownloadSucceeded { get; private set; }

    public ToolSetupDialog(ToolSetupViewModel viewModel)
    {
        InitializeComponent();
        ViewModel = viewModel;
        ViewModel.PropertyChanged += OnVmPropertyChanged;
        UpdateUi();

        PrimaryButtonClick += async (_, args) =>
        {
            var def = args.GetDeferral();
            try
            {
                // Block the dialog from auto-closing while we download; we'll
                // close it ourselves when done.
                args.Cancel = true;
                IsPrimaryButtonEnabled = false;
                CloseButtonText = "Cancel";
                DownloadSucceeded = await ViewModel.DownloadAsync();
                if (DownloadSucceeded)
                {
                    Hide();
                }
                else
                {
                    IsPrimaryButtonEnabled = true;
                    PrimaryButtonText = "Retry";
                }
            }
            finally
            {
                def.Complete();
            }
        };
    }

    private void OnVmPropertyChanged(object? sender, PropertyChangedEventArgs e) => UpdateUi();

    private void UpdateUi()
    {
        FfmpegBar.Value = ViewModel.FfmpegProgress;
        MpvBar.Value = ViewModel.MpvProgress;
        FfmpegStatus.Text = ViewModel.FfmpegStatus;
        MpvStatus.Text = ViewModel.MpvStatus;
        ErrorBar.IsOpen = !string.IsNullOrEmpty(ViewModel.ErrorMessage);
        ErrorBar.Message = ViewModel.ErrorMessage ?? "";
    }
}
