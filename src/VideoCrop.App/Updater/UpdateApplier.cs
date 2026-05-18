using System;
using System.Diagnostics;
using System.IO;
using System.Text;

namespace VideoCrop.App.Updater;

/// <summary>
/// Spawns a detached PowerShell helper that waits for the current process to
/// exit, mirrors the staged files over the install directory (including the
/// updater binary itself — that's safe because we're no longer running), then
/// relaunches the app. The script lives in a temp file and self-deletes.
/// </summary>
public static class UpdateApplier
{
    public static void ApplyAndRestart(string installDir, string stagedDir, int currentPid, string relaunchExePath)
    {
        if (!Directory.Exists(stagedDir))
            throw new DirectoryNotFoundException($"Staged directory not found: {stagedDir}");

        var scriptPath = Path.Combine(Path.GetTempPath(), $"videocrop-update-{Guid.NewGuid():N}.ps1");
        var logPath = Path.Combine(Path.GetTempPath(), $"videocrop-update-{Guid.NewGuid():N}.log");

        var script = BuildScript(installDir, stagedDir, currentPid, relaunchExePath, logPath);
        File.WriteAllText(scriptPath, script, new UTF8Encoding(encoderShouldEmitUTF8Identifier: false));

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

        var proc = Process.Start(psi)
            ?? throw new InvalidOperationException("Failed to start update helper.");
        // Don't wait — let it run in the background, our process is about to exit.
        _ = proc;
    }

    private static string BuildScript(string installDir, string stagedDir, int currentPid, string relaunchExePath, string logPath)
    {
        // Use a single-quoted here-string so PowerShell does not expand anything
        // in the embedded paths; quote each path explicitly below.
        var sb = new StringBuilder();
        sb.AppendLine("$ErrorActionPreference = 'Stop'");
        sb.AppendLine($"$InstallDir = '{Escape(installDir)}'");
        sb.AppendLine($"$StagedDir  = '{Escape(stagedDir)}'");
        sb.AppendLine($"$Relaunch   = '{Escape(relaunchExePath)}'");
        sb.AppendLine($"$LogPath    = '{Escape(logPath)}'");
        // $PID is a PowerShell automatic variable holding the running shell's
        // own PID — it's read-only and assigning to it doesn't take effect.
        // Use $AppPid for the VideoCrop pid so Wait-Process gates on the
        // right process.
        sb.AppendLine($"$AppPid     = {currentPid}");
        sb.AppendLine();
        sb.AppendLine("function Log([string]$msg) { Add-Content -Path $LogPath -Value (\"[{0:HH:mm:ss}] {1}\" -f (Get-Date), $msg) }");
        sb.AppendLine();
        sb.AppendLine("Log \"Waiting for pid $AppPid to exit...\"");
        sb.AppendLine("try {");
        sb.AppendLine("    Wait-Process -Id $AppPid -Timeout 30 -ErrorAction SilentlyContinue");
        sb.AppendLine("} catch { Log \"Wait error: $_\" }");
        sb.AppendLine();
        sb.AppendLine("# Brief settle delay for file handles to release.");
        sb.AppendLine("Start-Sleep -Milliseconds 500");
        sb.AppendLine();
        sb.AppendLine("Log \"Mirroring $StagedDir -> $InstallDir\"");
        sb.AppendLine("try {");
        sb.AppendLine("    # robocopy returns 0-7 for success; >=8 is failure. /MIR mirrors directories, /XJ skips junctions, /NFL /NDL /NJH /NJS keeps output small.");
        sb.AppendLine("    $proc = Start-Process -FilePath robocopy.exe -ArgumentList @($StagedDir, $InstallDir, '/E', '/XJ', '/R:3', '/W:1', '/NFL', '/NDL', '/NJH', '/NJS', '/NP') -NoNewWindow -Wait -PassThru");
        sb.AppendLine("    Log \"robocopy exit: $($proc.ExitCode)\"");
        sb.AppendLine("    if ($proc.ExitCode -ge 8) { Log 'Update failed during file copy.'; exit 1 }");
        sb.AppendLine("} catch { Log \"Copy error: $_\"; exit 1 }");
        sb.AppendLine();
        sb.AppendLine("Log \"Launching $Relaunch\"");
        sb.AppendLine("try {");
        sb.AppendLine("    Start-Process -FilePath $Relaunch -WorkingDirectory $InstallDir");
        sb.AppendLine("} catch { Log \"Launch error: $_\" }");
        sb.AppendLine();
        sb.AppendLine("# Self-delete this script. The helper PowerShell process exits shortly after.");
        sb.AppendLine("try { Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue } catch { }");
        return sb.ToString();
    }

    private static string Escape(string path) => path.Replace("'", "''");
}
