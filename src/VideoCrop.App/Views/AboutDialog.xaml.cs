using System.Reflection;
using Microsoft.UI.Xaml.Controls;

namespace VideoCrop.App.Views;

public sealed partial class AboutDialog : ContentDialog
{
    public AboutDialog()
    {
        InitializeComponent();
        var version = Assembly.GetExecutingAssembly().GetName().Version?.ToString() ?? "dev";
        VersionText.Text = $"Version {version}";
    }
}
