using System.ComponentModel;
using System.Runtime.InteropServices;
using Microsoft.UI.Xaml;

namespace BibCiTeX;

// Windows notification area and menus, attached to the WinUI HWND message loop.
internal sealed class TrayIcon : IDisposable
{
    private const uint Message = 0x8000 + 52;
    private readonly nint hwnd;
    private readonly nint icon;
    private readonly WindowInterop.SubclassProc callback;
    private Data data;
    private bool disposed;
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct Data
    {
        internal uint Size; internal nint Window; internal uint Id, Flags, CallbackMessage; internal nint Icon;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 128)] internal string Tip;
        internal uint State, StateMask;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 256)] internal string Info;
        internal uint Version;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 64)] internal string InfoTitle;
        internal uint InfoFlags; internal Guid Guid; internal nint BalloonIcon;
    }
    [DllImport("shell32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)] private static extern bool Shell_NotifyIconW(uint message, ref Data data);
    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)] private static extern nint LoadImageW(nint instance, string name, uint type, int width, int height, uint flags);
    [DllImport("user32.dll")] private static extern bool DestroyIcon(nint icon);
    [DllImport("user32.dll")] private static extern nint CreatePopupMenu();
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern bool AppendMenuW(nint menu, uint flags, nuint id, string? name);
    [DllImport("user32.dll")] private static extern uint TrackPopupMenuEx(nint menu, uint flags, int x, int y, nint hwnd, nint reserved);
    [DllImport("user32.dll")] private static extern bool DestroyMenu(nint menu);
    [DllImport("user32.dll")] private static extern bool SetForegroundWindow(nint hwnd);
    [DllImport("user32.dll")] private static extern bool PostMessageW(nint hwnd, uint message, nuint wparam, nint lparam);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern uint RegisterWindowMessageW(string name);

    internal TrayIcon(Window window, Action show, Action helper, Action update, Action quit)
    {
        hwnd = WinRT.Interop.WindowNative.GetWindowHandle(window);
        icon = LoadImageW(0, Path.Combine(AppContext.BaseDirectory, "Assets", "icon.ico"), 1, 16, 16, 0x10);
        if (icon == 0) throw new Win32Exception(Marshal.GetLastWin32Error());
        data = new Data { Size = (uint)Marshal.SizeOf<Data>(), Window = hwnd, Id = 1, Flags = 1 | 2 | 4, CallbackMessage = Message, Icon = icon, Tip = "BibCiTeX", Info = "", InfoTitle = "" };
        var taskbarCreated = RegisterWindowMessageW("TaskbarCreated");
        callback = (handle, message, wparam, lparam, _, _) =>
        {
            if (message == taskbarCreated) Shell_NotifyIconW(0, ref data);
            if (message == Message)
            {
                if ((uint)lparam == 0x0203) { show(); return 0; }
                if ((uint)lparam == 0x0205)
                {
                    var menu = CreatePopupMenu();
                    try
                    {
                        AppendMenuW(menu, 0, 1, "显示窗口"); AppendMenuW(menu, 0, 2, "快捷助手"); AppendMenuW(menu, 0, 3, "检查更新"); AppendMenuW(menu, 0x800, 0, null); AppendMenuW(menu, 0, 4, "退出 BibCiTeX");
                        WindowInterop.GetCursorPos(out var point); SetForegroundWindow(hwnd);
                        switch (TrackPopupMenuEx(menu, 0x100 | 2, point.X, point.Y, hwnd, 0))
                        { case 1: show(); break; case 2: helper(); break; case 3: update(); break; case 4: quit(); break; }
                    }
                    finally { DestroyMenu(menu); PostMessageW(hwnd, 0, 0, 0); }
                    return 0;
                }
            }
            return WindowInterop.DefSubclassProc(handle, message, wparam, lparam);
        };
        if (!WindowInterop.SetWindowSubclass(hwnd, callback, Message, 0)) { DestroyIcon(icon); throw new Win32Exception(Marshal.GetLastWin32Error()); }
        if (!Shell_NotifyIconW(0, ref data)) { WindowInterop.RemoveWindowSubclass(hwnd, callback, Message); DestroyIcon(icon); throw new Win32Exception(Marshal.GetLastWin32Error()); }
    }
    public void Dispose()
    {
        if (disposed) return; disposed = true;
        Shell_NotifyIconW(2, ref data); WindowInterop.RemoveWindowSubclass(hwnd, callback, Message); DestroyIcon(icon); GC.KeepAlive(callback);
    }
}
