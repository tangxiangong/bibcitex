using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.UI.Xaml;

namespace BibCiTeX;

internal static partial class WindowInterop
{
    internal static void ConfigureTitleBar(Window window)
    {
        window.AppWindow.SetIcon(System.IO.Path.Combine(AppContext.BaseDirectory, "Assets", "icon.ico"));
        var root = (FrameworkElement)window.Content;
        void Refresh() => window.AppWindow.TitleBar.PreferredTheme = root.ActualTheme == ElementTheme.Dark
            ? Microsoft.UI.Windowing.TitleBarTheme.Dark
            : Microsoft.UI.Windowing.TitleBarTheme.Light;
        void ThemeChanged(FrameworkElement sender, object args) => Refresh();
        void Loaded(object sender, RoutedEventArgs args) => Refresh();
        root.ActualThemeChanged += ThemeChanged;
        root.Loaded += Loaded;
        window.Closed += (_, _) =>
        {
            root.ActualThemeChanged -= ThemeChanged;
            root.Loaded -= Loaded;
        };
        Refresh();
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct MenuItemInfo
    {
        internal uint Size, Mask, Type, State, Id;
        internal nint Submenu, CheckedBitmap, UncheckedBitmap;
        internal nuint Data;
        [MarshalAs(UnmanagedType.LPWStr)] internal string Text;
        internal uint TextLength;
        internal nint Bitmap;
    }
    [LibraryImport("user32.dll")]
    private static partial nint GetSystemMenu(nint window, [MarshalAs(UnmanagedType.Bool)] bool revert);
    [DllImport("user32.dll", CharSet = CharSet.Unicode, EntryPoint = "GetMenuStringW")]
    private static extern int GetMenuString(nint menu, uint item, StringBuilder text, int count, uint flags);
    [DllImport("user32.dll", CharSet = CharSet.Unicode, EntryPoint = "SetMenuItemInfoW")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool SetMenuItemInfo(nint menu, uint item, [MarshalAs(UnmanagedType.Bool)] bool byPosition, ref MenuItemInfo info);
    internal static void LocalizeSystemMenu(Window window)
    {
        var menu = GetSystemMenu(WinRT.Interop.WindowNative.GetWindowHandle(window), false);
        if (menu == 0) return;
        foreach (var (command, key) in new (uint Command, string Key)[]
        {
            (0xF120, "windowsMenu.restore"), (0xF010, "windowsMenu.move"), (0xF000, "windowsMenu.size"),
            (0xF020, "windowsMenu.minimize"), (0xF030, "windowsMenu.maximize"), (0xF060, "windowsMenu.close")
        })
        {
            var original = new StringBuilder(256);
            if (GetMenuString(menu, command, original, original.Capacity, 0) == 0) continue;
            var previous = original.ToString(); var shortcut = previous.IndexOf('\t');
            var title = L10n.Text(key) + (shortcut < 0 ? "" : previous[shortcut..]);
            var info = new MenuItemInfo { Size = (uint)Marshal.SizeOf<MenuItemInfo>(), Mask = 0x40, Text = title };
            // MIIM_STRING changes only the title: preserve commands, flags and disabled states.
            SetMenuItemInfo(menu, command, false, ref info);
        }
    }
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
