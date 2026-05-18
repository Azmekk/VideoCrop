using System;
using System.ComponentModel;

namespace VideoCrop.App.Interop;

/// <summary>
/// A bare child window parented to the WinUI main window's HWND. mpv attaches
/// to this via <c>--wid=&lt;hwnd&gt;</c>. We use the predefined STATIC class so
/// we don't need to register one (or manage a WndProc delegate) — STATIC has
/// its own DefWindowProc which is fine because mpv handles its own rendering
/// and input via the IPC pipe.
/// </summary>
internal sealed class VideoHostWindow : IDisposable
{
    private IntPtr _hwnd;
    private readonly IntPtr _parent;

    public IntPtr Handle => _hwnd;
    public IntPtr ParentHandle => _parent;

    public VideoHostWindow(IntPtr parent)
    {
        if (parent == IntPtr.Zero) throw new ArgumentException("Parent HWND required.", nameof(parent));
        _parent = parent;

        const uint style = NativeMethods.WS_CHILD
                         | NativeMethods.WS_CLIPCHILDREN
                         | NativeMethods.WS_CLIPSIBLINGS;
        // Created hidden (no WS_VISIBLE). Caller flips visibility via SetBounds.
        _hwnd = NativeMethods.CreateWindowExW(
            dwExStyle: 0,
            lpClassName: "STATIC",
            lpWindowName: null,
            dwStyle: style,
            x: 0, y: 0, nWidth: 1, nHeight: 1,
            hWndParent: parent,
            hMenu: IntPtr.Zero,
            hInstance: IntPtr.Zero,
            lpParam: IntPtr.Zero);

        if (_hwnd == IntPtr.Zero)
            throw new Win32Exception("CreateWindowExW failed for video host child window.");
    }

    /// <summary>Move/resize the child window and make it visible.</summary>
    public void SetBounds(int x, int y, int width, int height)
    {
        if (_hwnd == IntPtr.Zero) return;
        NativeMethods.SetWindowPos(
            _hwnd, IntPtr.Zero,
            x, y, Math.Max(1, width), Math.Max(1, height),
            NativeMethods.SWP_NOZORDER | NativeMethods.SWP_NOACTIVATE | NativeMethods.SWP_SHOWWINDOW);
    }

    public void Hide()
    {
        if (_hwnd == IntPtr.Zero) return;
        NativeMethods.SetWindowPos(
            _hwnd, IntPtr.Zero, 0, 0, 0, 0,
            NativeMethods.SWP_NOZORDER | NativeMethods.SWP_NOACTIVATE | NativeMethods.SWP_HIDEWINDOW);
    }

    public uint GetParentDpi() => NativeMethods.GetDpiForWindow(_parent);

    /// <summary>
    /// Physically move the parent window by 1px and back. A freshly-shown
    /// WinUI 3 window doesn't have DWM fully wired up for child-swapchain
    /// compositing — mpv presents frames into its swapchain but DWM keeps
    /// the stale parent frame until the parent is actually moved. SWP_NOMOVE
    /// pokes and SWP_FRAMECHANGED don't count as "real" moves; only an
    /// actual coordinate change does. Doing this once after window creation
    /// primes DWM for the rest of the session.
    /// </summary>
    public bool NudgeParentPosition()
    {
        if (_parent == IntPtr.Zero) return false;
        if (!NativeMethods.GetWindowRect(_parent, out var r)) return false;
        var ok1 = NativeMethods.SetWindowPos(
            _parent, IntPtr.Zero,
            r.Left + 1, r.Top, 0, 0,
            NativeMethods.SWP_NOSIZE | NativeMethods.SWP_NOZORDER | NativeMethods.SWP_NOACTIVATE);
        var ok2 = NativeMethods.SetWindowPos(
            _parent, IntPtr.Zero,
            r.Left, r.Top, 0, 0,
            NativeMethods.SWP_NOSIZE | NativeMethods.SWP_NOZORDER | NativeMethods.SWP_NOACTIVATE);
        return ok1 && ok2;
    }

    public void Dispose()
    {
        if (_hwnd != IntPtr.Zero)
        {
            NativeMethods.DestroyWindow(_hwnd);
            _hwnd = IntPtr.Zero;
        }
    }
}
