using System;
using System.IO;
using System.Runtime.InteropServices;
using Serilog;

namespace VideoCrop.App.Services;

/// <summary>
/// Creates a per-user .lnk under the Start Menu's Programs folder. Windows
/// Search indexes that location, so the entry becomes searchable. We use
/// <c>WScript.Shell</c> via late binding to avoid hand-rolled IShellLink
/// P/Invoke.
/// </summary>
internal static class StartMenuShortcut
{
    public static string LinkPath => Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.StartMenu),
        "Programs",
        "VideoCrop.lnk");

    public static bool Exists() => File.Exists(LinkPath);

    public static bool Delete()
    {
        try
        {
            if (File.Exists(LinkPath)) File.Delete(LinkPath);
            return true;
        }
        catch (Exception ex)
        {
            Log.Warning(ex, "Failed to delete Start menu shortcut");
            return false;
        }
    }

    public static bool Create(string targetExe, string? iconPath = null)
    {
        try
        {
            var shellType = Type.GetTypeFromProgID("WScript.Shell");
            if (shellType is null)
            {
                Log.Warning("WScript.Shell not available — can't create Start menu shortcut.");
                return false;
            }
            object? shell = Activator.CreateInstance(shellType);
            if (shell is null) return false;
            try
            {
                dynamic shellDyn = shell;
                dynamic shortcut = shellDyn.CreateShortcut(LinkPath);
                try
                {
                    shortcut.TargetPath = targetExe;
                    shortcut.WorkingDirectory = Path.GetDirectoryName(targetExe) ?? "";
                    shortcut.IconLocation = (iconPath ?? targetExe) + ",0";
                    shortcut.Description = "Trim, crop, resize, and compress videos";
                    shortcut.Save();
                    return true;
                }
                finally
                {
                    Marshal.FinalReleaseComObject(shortcut);
                }
            }
            finally
            {
                Marshal.FinalReleaseComObject(shell);
            }
        }
        catch (Exception ex)
        {
            Log.Warning(ex, "Failed to create Start menu shortcut");
            return false;
        }
    }
}
