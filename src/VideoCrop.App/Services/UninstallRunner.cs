using System;
using System.Diagnostics;
using System.IO;
using Serilog;

namespace VideoCrop.App.Services;

/// <summary>
/// Runs the cleanup flow when VideoCrop.App.exe is launched with
/// <c>--uninstall</c> (the command Apps &amp; Features fires). Tears down
/// the side-effects we installed via <see cref="AppsAndFeaturesRegistration"/>
/// and <see cref="StartMenuShortcut"/>, then schedules removal of the
/// install directory and user data via a detached PowerShell helper that
/// waits for our PID to exit (Windows holds a lock on our exe while we
/// run, so we can't delete it ourselves).
/// </summary>
internal static class UninstallRunner
{
    public static void Run()
    {
        Log.Information("Uninstall flow starting");

        try { StartMenuShortcut.Delete(); } catch (Exception ex) { Log.Warning(ex, "shortcut delete failed"); }
        try { AppsAndFeaturesRegistration.Unregister(); } catch (Exception ex) { Log.Warning(ex, "registry delete failed"); }

        var exe = Environment.ProcessPath;
        if (string.IsNullOrEmpty(exe))
        {
            Log.Warning("No ProcessPath; cannot schedule install-dir cleanup");
            return;
        }
        var installDir = Path.GetDirectoryName(exe);
        if (string.IsNullOrEmpty(installDir))
        {
            Log.Warning("Could not resolve install dir for cleanup");
            return;
        }
        var userData = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "VideoCrop");

        ScheduleDeferredCleanup(installDir, userData);
    }

    private static void ScheduleDeferredCleanup(string installDir, string userData)
    {
        try
        {
            var pid = Environment.ProcessId;
            // Use a temp script — running powershell -Command "..." inline
            // is fragile with embedded quotes and paths.
            var scriptPath = Path.Combine(Path.GetTempPath(), $"videocrop-uninstall-{Guid.NewGuid():N}.ps1");
            var script = $@"
$ErrorActionPreference = 'SilentlyContinue'
$pidToWatch = {pid}
try {{ Wait-Process -Id $pidToWatch -Timeout 30 }} catch {{}}
Start-Sleep -Milliseconds 500
Remove-Item -LiteralPath '{userData.Replace("'", "''")}'   -Recurse -Force
Remove-Item -LiteralPath '{installDir.Replace("'", "''")}' -Recurse -Force
Remove-Item -LiteralPath $MyInvocation.MyCommand.Path -Force
";
            File.WriteAllText(scriptPath, script);

            var psi = new ProcessStartInfo
            {
                FileName = "powershell.exe",
                UseShellExecute = false,
                CreateNoWindow = true,
                WindowStyle = ProcessWindowStyle.Hidden,
            };
            psi.ArgumentList.Add("-NoProfile");
            psi.ArgumentList.Add("-ExecutionPolicy");
            psi.ArgumentList.Add("Bypass");
            psi.ArgumentList.Add("-WindowStyle");
            psi.ArgumentList.Add("Hidden");
            psi.ArgumentList.Add("-File");
            psi.ArgumentList.Add(scriptPath);
            Process.Start(psi);
            Log.Information("Deferred cleanup helper spawned ({Script})", scriptPath);
        }
        catch (Exception ex)
        {
            Log.Error(ex, "Failed to schedule deferred uninstall cleanup");
        }
    }
}
