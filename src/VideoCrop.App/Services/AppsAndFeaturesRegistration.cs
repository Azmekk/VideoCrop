using System;
using System.IO;
using System.Reflection;
using Microsoft.Win32;
using Serilog;

namespace VideoCrop.App.Services;

/// <summary>
/// Per-user Apps & Features (formerly "Programs and Features") registration
/// via the HKCU uninstall key. Lets the OS list VideoCrop in Settings → Apps
/// and provide a working Uninstall button without an MSI/MSIX installer.
/// </summary>
internal static class AppsAndFeaturesRegistration
{
    private const string KeyPath =
        @"Software\Microsoft\Windows\CurrentVersion\Uninstall\VideoCrop";

    public static bool IsRegistered()
    {
        using var key = Registry.CurrentUser.OpenSubKey(KeyPath);
        return key is not null;
    }

    public static bool Register(string exePath)
    {
        try
        {
            var installDir = Path.GetDirectoryName(exePath) ?? "";
            var version = Assembly.GetExecutingAssembly()
                .GetCustomAttribute<AssemblyInformationalVersionAttribute>()
                ?.InformationalVersion
                ?? Assembly.GetExecutingAssembly().GetName().Version?.ToString()
                ?? "2.0.0";
            // Strip CI build metadata after '+' if present (1.2.3+abc → 1.2.3).
            var plusIdx = version.IndexOf('+');
            if (plusIdx > 0) version = version[..plusIdx];

            using var key = Registry.CurrentUser.CreateSubKey(KeyPath)
                ?? throw new InvalidOperationException("Could not create uninstall key.");
            key.SetValue("DisplayName", "VideoCrop", RegistryValueKind.String);
            key.SetValue("DisplayVersion", version, RegistryValueKind.String);
            key.SetValue("Publisher", "Azmek", RegistryValueKind.String);
            key.SetValue("InstallLocation", installDir, RegistryValueKind.String);
            key.SetValue("DisplayIcon", exePath + ",0", RegistryValueKind.String);
            key.SetValue("UninstallString", $"\"{exePath}\" --uninstall", RegistryValueKind.String);
            key.SetValue("URLInfoAbout", "https://github.com/Azmekk/VideoCrop", RegistryValueKind.String);
            key.SetValue("NoModify", 1, RegistryValueKind.DWord);
            key.SetValue("NoRepair", 1, RegistryValueKind.DWord);
            key.SetValue("EstimatedSize", EstimateSizeKB(installDir), RegistryValueKind.DWord);
            Log.Information("Registered VideoCrop in Apps & Features at {Path}", exePath);
            return true;
        }
        catch (Exception ex)
        {
            Log.Warning(ex, "Failed to register in Apps & Features");
            return false;
        }
    }

    public static bool Unregister()
    {
        try
        {
            Registry.CurrentUser.DeleteSubKeyTree(KeyPath, throwOnMissingSubKey: false);
            return true;
        }
        catch (Exception ex)
        {
            Log.Warning(ex, "Failed to remove Apps & Features registration");
            return false;
        }
    }

    private static int EstimateSizeKB(string installDir)
    {
        try
        {
            if (!Directory.Exists(installDir)) return 0;
            long bytes = 0;
            foreach (var f in Directory.EnumerateFiles(installDir, "*", SearchOption.AllDirectories))
            {
                try { bytes += new FileInfo(f).Length; } catch { }
            }
            return (int)Math.Min(int.MaxValue, bytes / 1024);
        }
        catch { return 0; }
    }
}
