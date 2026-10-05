using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Windows.System;

namespace BibCiTeX;

/// <summary>
/// The workbench collapsed to panel scale: a search field over the reference
/// list, with the active library on the header. It is the second presentation of
/// the same application state as <see cref="MainWindow"/> — the same Rust search
/// and paste path — and deliberately has no detail inspector.
///
/// Two entry points share this one window, model and paste path: the global
/// hotkey shows it centred, and the notification-area icon shows it as the tray
/// window, hanging above the taskbar on the primary work area.
/// </summary>
internal sealed class HelperWindow : Window, IDisposable
{
    // Kept in step with the macOS helper metrics so both platforms read the same.
    private const double Header = 56;
    private const double LibraryRow = 56;
    private const double ReferenceRow = 80;
    private const double ListPadding = 16;
    private const double StatusRow = 40;
    private const double FallbackRow = 40;
    private const double MaxList = 460;
    private const double TrayWidth = 460;
    private const double PanelWidth = 720;

    private readonly Grid root = new();
    private readonly TextBox search = new() { PlaceholderText = "搜索文献、作者、标题", FontSize = 22, BorderThickness = new Thickness(0), Background = new SolidColorBrush(Microsoft.UI.Colors.Transparent), Padding = new Thickness(0, 8, 0, 8), VerticalAlignment = VerticalAlignment.Center };
    private readonly ListView results = new() { SelectionMode = ListViewSelectionMode.Single, IsItemClickEnabled = true, Padding = new Thickness(8, 0, 8, 8) };
    private readonly TextBlock status = Views.Text("");
    private readonly Button chooseLibrary;
    private readonly ProgressRing progress = new() { Width = 18, Height = 18, IsActive = false, Visibility = Visibility.Collapsed };
    private readonly Button fallback;
    private List<Library> libraries = [];
    private Library? current;
    private bool selecting;
    private bool composing;
    private bool visible;
    private bool active;
    private bool opening;
    private bool pasting;
    private bool disposed;
    private bool busy;
    private bool trayAnchored;
    private int version;
    private int lifecycle;
    private string? failedKey;

    internal HelperWindow()
    {
        Title = "快捷助手"; SystemBackdrop = new DesktopAcrylicBackdrop();
        var presenter = OverlappedPresenter.CreateForToolWindow(); presenter.SetBorderAndTitleBar(false, false); presenter.IsAlwaysOnTop = true; presenter.IsResizable = false; presenter.IsMinimizable = false; presenter.IsMaximizable = false; AppWindow.SetPresenter(presenter);
        AppWindow.IsShownInSwitchers = false;
        root.RequestedTheme = App.Theme;
        root.RowDefinitions.Add(new() { Height = new GridLength(Header) });
        root.RowDefinitions.Add(new() { Height = GridLength.Auto });
        root.RowDefinitions.Add(new());
        root.RowDefinitions.Add(new() { Height = GridLength.Auto });

        var header = new Grid { Padding = new Thickness(20, 0, 16, 0), ColumnSpacing = 14 };
        header.ColumnDefinitions.Add(new() { Width = GridLength.Auto }); header.ColumnDefinitions.Add(new()); header.ColumnDefinitions.Add(new() { Width = GridLength.Auto }); header.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        header.Children.Add(new SvgIcon("search", 24)); Grid.SetColumn(search, 1); header.Children.Add(search);
        AutomationProperties.SetName(search, "搜索");
        Grid.SetColumn(progress, 2); header.Children.Add(progress);
        chooseLibrary = Views.Button("library", "文献库", () => { if (busy) return; selecting = !selecting; search.Text = ""; _ = Refresh(); });
        Grid.SetColumn(chooseLibrary, 3); header.Children.Add(chooseLibrary); root.Children.Add(header);

        status.Margin = new Thickness(20, 8, 20, 8); status.Visibility = Visibility.Collapsed; Grid.SetRow(status, 1); root.Children.Add(status);
        Grid.SetRow(results, 2); root.Children.Add(results);
        fallback = new Button { Content = "复制引用键", Margin = new Thickness(20, 0, 20, 8), Visibility = Visibility.Collapsed };
        fallback.Click += async (_, _) => { if (failedKey is { } key) { try { await RustCore.Copy(key); } catch (Exception error) { SetStatus(error.Message); } } };
        Grid.SetRow(fallback, 3); root.Children.Add(fallback);
        Content = root;
        search.TextCompositionStarted += (_, _) => composing = true;
        search.TextCompositionEnded += (_, _) => { composing = false; _ = Refresh(); };
        search.TextChanged += (_, _) => { if (!composing) _ = Refresh(true); };
        search.PreviewKeyDown += KeyDown;
        results.PreviewKeyDown += KeyDown;
        results.ItemClick += async (_, e) => await Execute(e.ClickedItem as ListViewItem);
        Activated += (_, args) => { active = args.WindowActivationState != WindowActivationState.Deactivated; if (!active && !opening && !pasting) Hide(); };
        AppWindow.Closing += (_, args) => { if (!disposed) { args.Cancel = true; Hide(); } };
        Resize(Header);
    }

    /// <summary>
    /// Shows the search-and-paste surface. <paramref name="tray"/> anchors it above
    /// the notification area (the tray window); otherwise it is centred, as the
    /// global hotkey presents it. Both share this one window, model and paste path.
    /// </summary>
    internal async Task Toggle(bool tray = false)
    {
        if (pasting) return;
        if (visible) { Hide(); return; }
        if (opening || disposed || busy) return;
        trayAnchored = tray;
        opening = true;
        busy = true;
        var showVersion = ++lifecycle;
        try
        {
            // Capture before activation, including when launched by the global hotkey.
            await RustCore.CapturePasteTarget();
            if (showVersion != lifecycle) return;
            failedKey = null; fallback.Visibility = Visibility.Collapsed; SetStatus("");
            search.Text = ""; results.Items.Clear(); Resize(Header); Position(); visible = true;
            Activate(); search.Focus(FocusState.Programmatic);
            var rows = await RustCore.Libraries();
            var saved = await RustCore.HelperCurrent();
            if (showVersion != lifecycle) return;
            libraries = rows; current = saved;
            if (current is not null && !libraries.Any(x => x.Name == current.Name && x.Path == current.Path)) current = null;
            selecting = current is null;
            busy = false;
            await Refresh();
        }
        catch (Exception error)
        {
            if (showVersion == lifecycle) { visible = true; Activate(); SetStatus(error.Message); Resize(Header + ReferenceRow); }
        }
        finally { opening = false; busy = false; if (visible && !active) Hide(); }
    }
    private void Hide()
    {
        visible = false; version++; lifecycle++; composing = false; trayAnchored = false; AppWindow.Hide();
    }
    private async Task Refresh(bool debounce = false)
    {
        if (!visible || busy) return;
        var request = ++version;
        failedKey = null; fallback.Visibility = Visibility.Collapsed; SetStatus("");
        search.PlaceholderText = selecting ? "搜索或选择文献库" : "搜索文献、作者、标题";
        ToolTipService.SetToolTip(chooseLibrary, current?.Name ?? "文献库");
        var query = search.Text; var selectedPath = current?.Path;
        try
        {
            if (debounce) await Task.Delay(90);
            if (request != version) return;
            results.Items.Clear();
            if (selecting)
            {
                foreach (var library in libraries.Where(x => x.Name.Contains(query, StringComparison.CurrentCultureIgnoreCase) || x.Path.Contains(query, StringComparison.CurrentCultureIgnoreCase) || x.Description.Contains(query, StringComparison.CurrentCultureIgnoreCase) || x.UpdatedAt.Contains(query, StringComparison.CurrentCultureIgnoreCase))) results.Items.Add(Views.LibraryItem(library, current?.Name == library.Name && current?.Path == library.Path));
                if (results.Items.Count == 0) SetStatus(libraries.Count == 0 ? "未找到文献库，请先到主窗口添加文献库" : "未找到匹配的文献");
            }
            else if (query.Trim().Length > 0 && selectedPath is not null)
            {
                progress.Visibility = Visibility.Visible; progress.IsActive = true;
                var references = await RustCore.Search(selectedPath, query);
                if (request != version) return;
                // The hotkey panel pastes the row, so a second copy target there
                // would only compete with it; the tray window collects keys, so its
                // rows make the key itself clickable.
                foreach (var reference in references) results.Items.Add(Views.ReferenceItem(reference, citeKeyCopies: trayAnchored));
                if (references.Count == 0) SetStatus("未找到匹配的文献");
            }
            if (results.Items.Count > 0) results.SelectedIndex = 0;
            ResizeContent();
        }
        catch (Exception error) { if (request == version) { SetStatus(error.Message); ResizeContent(); } }
        finally { if (request == version) { progress.IsActive = false; progress.Visibility = Visibility.Collapsed; } }
    }
    private async void KeyDown(object sender, KeyRoutedEventArgs args)
    {
        if (composing || (busy && args.Key != VirtualKey.Escape)) return;
        var control = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(VirtualKey.Control);
        var alt = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(VirtualKey.Menu);
        var shift = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(VirtualKey.Shift);
        var winLeft = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(VirtualKey.LeftWindows);
        var winRight = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(VirtualKey.RightWindows);
        if (((control | alt | shift | winLeft | winRight) & Windows.UI.Core.CoreVirtualKeyStates.Down) != 0) return;
        switch (args.Key)
        {
            case VirtualKey.Escape: args.Handled = true; Hide(); break;
            case VirtualKey.Tab: args.Handled = true; selecting = !selecting || current is null; search.Text = ""; await Refresh(); search.Focus(FocusState.Programmatic); break;
            case VirtualKey.Down:
            case VirtualKey.Up:
                args.Handled = true;
                if (results.Items.Count > 0) { var direction = args.Key == VirtualKey.Down ? 1 : -1; results.SelectedIndex = (results.SelectedIndex + direction + results.Items.Count) % results.Items.Count; results.ScrollIntoView(results.SelectedItem); }
                break;
            case VirtualKey.Enter: args.Handled = true; await Execute(results.SelectedItem as ListViewItem); break;
        }
    }
    private async Task Execute(ListViewItem? item)
    {
        if (pasting || busy || item is null) return;
        var session = lifecycle;
        var request = ++version;
        try
        {
            if (item.Tag is Library library)
            {
                busy = true;
                var initialQuery = search.Text;
                var selected = await RustCore.HelperSelect(library.Name, library.Path);
                if (request != version) return;
                current = selected; selecting = false; if (search.Text == initialQuery) search.Text = ""; busy = false; await Refresh(); search.Focus(FocusState.Programmatic);
            }
            else if (item.Tag is Reference reference)
            {
                if (trayAnchored)
                {
                    // Collecting keys: copy and stay open, so several records can be
                    // taken in one visit without the window closing each time. A
                    // failure still has to be visible, or the click looks ignored.
                    pasting = true;
                    // A failure has to be visible and the window has to grow to show
                    // it, or the click looks ignored.
                    if (!await Views.CopyKey(reference.Key)) { SetStatus("复制引用键失败"); ResizeContent(); }
                    pasting = false;
                }
                else
                {
                    pasting = true;
                    await RustCore.Paste(reference.Key);
                    if (session == lifecycle) Hide();
                }
            }
        }
        catch (Exception error)
        {
            if (request != version) return;
            if (item.Tag is Reference reference && !trayAnchored) { failedKey = reference.Key; fallback.Visibility = Visibility.Visible; }
            Activate(); search.Focus(FocusState.Programmatic); SetStatus(error.Message); ResizeContent();
        }
        finally { pasting = false; busy = false; }
    }
    private void SetStatus(string text) { status.Text = text; status.Visibility = text.Length == 0 ? Visibility.Collapsed : Visibility.Visible; }
    private void ResizeContent()
    {
        var rows = results.Items.Count;
        var list = rows == 0 ? 0 : Math.Min(MaxList, rows * (selecting ? LibraryRow : ReferenceRow) + ListPadding);
        Resize(Header + list + (status.Visibility == Visibility.Visible ? StatusRow : 0) + (fallback.Visibility == Visibility.Visible ? FallbackRow : 0));
    }
    private void Resize(double height)
    {
        var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this); var scale = WindowInterop.GetDpiForWindow(hwnd) / 96.0;
        var area = Area();
        // The tray window stays panel-width and bottom-anchored; the hotkey panel
        // gets the workbench's centre-column width.
        var width = Math.Min((trayAnchored ? TrayWidth : PanelWidth) * scale, area.Width - 40 * scale);
        AppWindow.ResizeClient(new Windows.Graphics.SizeInt32((int)Math.Max(320, width), (int)Math.Min(height * scale, area.Height - 80 * scale)));
        // Anchored, so it must be re-placed after every height change to grow upward.
        Place(area);
    }
    // The tray window hangs from the notification area on the primary work area;
    // the hotkey panel follows the display the pointer is on, like the workbench.
    private Windows.Graphics.RectInt32 Area() => trayAnchored
        ? DisplayArea.GetFromWindowId(AppWindow.Id, DisplayAreaFallback.Primary).WorkArea
        : DisplayArea.GetFromPoint(Cursor(), DisplayAreaFallback.Primary).WorkArea;
    private static Windows.Graphics.PointInt32 Cursor()
    {
        WindowInterop.GetCursorPos(out var point); return new Windows.Graphics.PointInt32(point.X, point.Y);
    }
    private void Place(Windows.Graphics.RectInt32 area)
    {
        var size = AppWindow.Size;
        var x = trayAnchored ? area.X + area.Width - size.Width - 12 : area.X + (area.Width - size.Width) / 2;
        var y = trayAnchored ? area.Y + area.Height - size.Height - 12 : area.Y + area.Height / 6;
        AppWindow.Move(new Windows.Graphics.PointInt32(x, y));
    }
    private void Position()
    {
        Resize(Header);
        // Moving across monitors can change DPI and therefore the final pixel width.
        Place(Area());
    }
    public void Dispose() { disposed = true; Hide(); }
}
