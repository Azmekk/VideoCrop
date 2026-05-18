using System.Runtime.InteropServices;

namespace VideoCrop.App.Interop;

internal static class NativeMethods
{
    public const int ATTACH_PARENT_PROCESS = -1;

    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool AttachConsole(int dwProcessId);
}
