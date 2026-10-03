using System.ComponentModel;
using System.Runtime.InteropServices;
using Microsoft.UI.Xaml;

namespace BibCiTeX;

internal static partial class WindowInterop
{
    [StructLayout(LayoutKind.Sequential)] internal struct Point { internal int X, Y; }
    [LibraryImport("user32.dll")] internal static partial uint GetDpiForWindow(nint hwnd);
    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)] internal static partial bool GetCursorPos(out Point point);
    [LibraryImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)] internal static partial bool RegisterHotKey(nint hwnd, int id, uint modifiers, uint key);
    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)] internal static partial bool UnregisterHotKey(nint hwnd, int id);
    internal delegate nint SubclassProc(nint hwnd, uint message, nuint wparam, nint lparam, nuint id, nuint data);
    [DllImport("comctl32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)] internal static extern bool SetWindowSubclass(nint hwnd, SubclassProc callback, nuint id, nuint data);
    [DllImport("comctl32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)] internal static extern bool RemoveWindowSubclass(nint hwnd, SubclassProc callback, nuint id);
    [LibraryImport("comctl32.dll")] internal static partial nint DefSubclassProc(nint hwnd, uint message, nuint wparam, nint lparam);
}

internal sealed class GlobalShortcut : IDisposable
{
    private const int Id = 0xB1B;
    private readonly nint hwnd;
    private readonly WindowInterop.SubclassProc callback;
    private bool disposed;
    internal GlobalShortcut(Window window, Action action)
    {
        hwnd = WinRT.Interop.WindowNative.GetWindowHandle(window);
        callback = (handle, message, wparam, lparam, _, _) =>
        {
            if (message == 0x0312 && wparam == Id) { action(); return 0; }
            return WindowInterop.DefSubclassProc(handle, message, wparam, lparam);
        };
        if (!WindowInterop.SetWindowSubclass(hwnd, callback, Id, 0)) throw new Win32Exception(Marshal.GetLastWin32Error());
        if (!WindowInterop.RegisterHotKey(hwnd, Id, 0x0008 | 0x0004 | 0x4000, 0x4B))
        { WindowInterop.RemoveWindowSubclass(hwnd, callback, Id); throw new Win32Exception(Marshal.GetLastWin32Error()); }
    }
    public void Dispose()
    {
        if (disposed) return; disposed = true;
        WindowInterop.UnregisterHotKey(hwnd, Id); WindowInterop.RemoveWindowSubclass(hwnd, callback, Id);
        GC.KeepAlive(callback);
    }
}
