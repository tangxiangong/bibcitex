using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Windows.System;

namespace BibCiTeX;

/// <summary>The independent global-hotkey search-and-paste panel.</summary>
internal sealed class HelperWindow : Window, IDisposable
{
    // Kept in step with the macOS helper metrics so both platforms read the same.
    private const double Header = 56;
    private const double LibraryRow = 56;
    private const double ReferenceRow = 80;
    private const double ListPadding = 16;
    private const double StatusRow = 40;
    private const double EmptyRow = 112;
    private const double MaxList = 460;
    private const double PanelWidth = 720;

    private readonly Grid root = new();
    private readonly TextBox search = new() { PlaceholderText = L10n.Text("搜索文献、作者、标题"), FontSize = 22, BorderThickness = new Thickness(0), Background = new SolidColorBrush(Microsoft.UI.Colors.Transparent), Padding = new Thickness(0, 8, 0, 8), VerticalAlignment = VerticalAlignment.Center };
    private readonly ListView results = new() { SelectionMode = ListViewSelectionMode.Single, IsItemClickEnabled = true, Padding = new Thickness(0), HorizontalContentAlignment = HorizontalAlignment.Stretch };
    private readonly TextBlock status = Views.Text("");
    private readonly Button chooseLibrary;
    private readonly TextBlock libraryName = new() { FontSize = 11, MaxLines = 1, TextTrimming = TextTrimming.CharacterEllipsis, MaxWidth = 108 };
    private readonly Grid errorBar = new() { ColumnSpacing = 7, Padding = new Thickness(12, 7, 12, 7), Visibility = Visibility.Collapsed };
    private readonly Grid listArea = new();
    private readonly StackPanel emptyState = new() { Spacing = 8, HorizontalAlignment = HorizontalAlignment.Center, VerticalAlignment = VerticalAlignment.Center, Visibility = Visibility.Collapsed };
    private readonly TextBlock emptyLabel = new() { FontSize = 12, TextWrapping = TextWrapping.Wrap, TextAlignment = TextAlignment.Center };
    private readonly SvgIcon emptyIcon = new("search", 24);
    private readonly ProgressRing listProgress = new() { Width = 24, Height = 24, IsActive = false, Visibility = Visibility.Collapsed };
    private bool searching;
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
    private int version;
    private int lifecycle;
    private string? failedKey;
    private ContentDialog? pasteFailureDialog;

    internal HelperWindow()
    {
        Title = L10n.Text("快捷助手"); SystemBackdrop = new DesktopAcrylicBackdrop();
        var presenter = OverlappedPresenter.CreateForToolWindow(); presenter.SetBorderAndTitleBar(false, false); presenter.IsAlwaysOnTop = true; presenter.IsResizable = false; presenter.IsMinimizable = false; presenter.IsMaximizable = false; AppWindow.SetPresenter(presenter);
        AppWindow.IsShownInSwitchers = false;
        root.RequestedTheme = App.Theme;
        Localized.BindValue(root, FrameworkElement.LanguageProperty, () => L10n.Language);
        Views.LocalizeTextBox(search);
        root.RowDefinitions.Add(new() { Height = new GridLength(Header) });
        root.RowDefinitions.Add(new() { Height = GridLength.Auto });
        root.RowDefinitions.Add(new());
        root.RowDefinitions.Add(new() { Height = GridLength.Auto });

        var header = new Grid { Padding = new Thickness(22, 0, 22, 0), ColumnSpacing = 14 };
        header.ColumnDefinitions.Add(new() { Width = GridLength.Auto }); header.ColumnDefinitions.Add(new()); header.ColumnDefinitions.Add(new() { Width = GridLength.Auto }); header.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        header.Children.Add(new SvgIcon("search", 22)); Grid.SetColumn(search, 1); header.Children.Add(search);
        Localized.Name(search, "搜索");
        Grid.SetColumn(progress, 2); header.Children.Add(progress);
        chooseLibrary = new Button { Content = Views.Row(new SvgIcon("library", 14), libraryName, new SvgIcon("chevronDown", 9)), CornerRadius = new CornerRadius(16), Padding = new Thickness(10, 6, 10, 6), MaxWidth = 160, BorderThickness = new Thickness(0) };
        Localized.Tooltip(chooseLibrary, "切换文献库 (Tab)"); Localized.Name(chooseLibrary, "切换文献库");
        chooseLibrary.Click += (_, _) => { if (busy || pasting) return; selecting = true; search.Text = ""; _ = Refresh(); search.Focus(FocusState.Programmatic); };
        Grid.SetColumn(chooseLibrary, 3); header.Children.Add(chooseLibrary); root.Children.Add(header);

        errorBar.ColumnDefinitions.Add(new() { Width = GridLength.Auto }); errorBar.ColumnDefinitions.Add(new()); errorBar.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        errorBar.Children.Add(new SvgIcon("alert", 14) { VerticalAlignment = VerticalAlignment.Center });
        status.FontSize = 11; status.MaxLines = 2; Views.ThemeForeground(status, "SystemFillColorCriticalBrush");
        Grid.SetColumn(status, 1); errorBar.Children.Add(status); Grid.SetRow(errorBar, 1); root.Children.Add(errorBar);
        // Keep equal gutters outside the scrolling template, so a template's
        // scrollbar/content padding cannot add a second right-hand gutter.
        listArea.Children.Add(new Border { Padding = new Thickness(8, 0, 8, 8), Child = results });
        emptyIcon.HorizontalAlignment = HorizontalAlignment.Center; emptyState.Children.Add(emptyIcon); emptyState.Children.Add(emptyLabel); listArea.Children.Add(emptyState); listArea.Children.Add(listProgress);
        var rule = Views.Divider(); rule.VerticalAlignment = VerticalAlignment.Top; listArea.Children.Add(rule);
        Grid.SetRow(listArea, 2); root.Children.Add(listArea);
        fallback = new Button { Content = L10n.Text("复制引用键"), Visibility = Visibility.Collapsed, FontSize = 11 };
        fallback.Click += async (_, _) => { if (failedKey is { } key) { try { await RustCore.Copy(key); } catch (Exception error) { SetErrorStatus(error); } } };
        Grid.SetColumn(fallback, 2); errorBar.Children.Add(fallback);
        Localized.Bind(search, TextBox.PlaceholderTextProperty, "搜索文献、作者、标题");
        Localized.Bind(fallback, ContentControl.ContentProperty, "复制引用键");
        L10n.Changed += LocalizeTitle;
        WindowInterop.LocalizeSystemMenu(this);
        ScrollViewer.SetVerticalScrollBarVisibility(results, ScrollBarVisibility.Hidden);
        ScrollViewer.SetHorizontalScrollBarVisibility(results, ScrollBarVisibility.Hidden);
        ScrollViewer.SetHorizontalScrollMode(results, ScrollMode.Disabled);
        Views.HideScrollbars(results);
        Views.HideScrollbars(search);
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

    private void LocalizeTitle() => Title = L10n.Text("快捷助手");

    internal async Task Toggle()
    {
        if (pasting) return;
        if (visible) { Hide(); return; }
        if (opening || disposed || busy) return;
        App.Tray?.Hide();
        opening = true;
        busy = true;
        var showVersion = ++lifecycle;
        try
        {
            // Capture before activation, including when launched by the global hotkey.
            await RustCore.CapturePasteTarget();
            if (showVersion != lifecycle) return;
            failedKey = null; fallback.Visibility = Visibility.Collapsed; SetStatus("");
            search.Text = ""; results.Items.Clear(); chooseLibrary.Visibility = Visibility.Collapsed; Resize(Header); Position(); visible = true;
            Activate(); search.Focus(FocusState.Programmatic); SetSearching(true);
            var rows = await RustCore.Libraries();
            var saved = await RustCore.HelperCurrent();
            if (showVersion != lifecycle) return;
            libraries = rows; current = saved;
            if (current is not null && !libraries.Any(x => x.Name == current.Name && x.Path == current.Path)) current = null;
            selecting = current is null;
            busy = false; SetSearching(false);
            await Refresh();
        }
        catch (Exception error)
        {
            if (showVersion == lifecycle) { visible = true; Activate(); SetSearching(false); SetErrorStatus(error); ResizeContent(); }
        }
        finally { opening = false; busy = false; if (visible && !active) Hide(); }
    }
    internal void Hide()
    {
        visible = false; version++; lifecycle++; composing = false; SetSearching(false); pasteFailureDialog?.Hide(); AppWindow.Hide();
    }
    private static ListViewItem FullWidthRow(ListViewItem item)
    {
        item.HorizontalAlignment = HorizontalAlignment.Stretch;
        item.HorizontalContentAlignment = HorizontalAlignment.Stretch;
        item.Margin = new Thickness(0);
        return item;
    }

    private async Task Refresh(bool debounce = false)
    {
        if (!visible || busy || pasting) return;
        var request = ++version;
        failedKey = null; fallback.Visibility = Visibility.Collapsed; SetStatus("");
        Localized.Bind(search, TextBox.PlaceholderTextProperty, selecting ? "搜索或选择文献库" : "搜索文献、作者、标题");
        chooseLibrary.Visibility = !selecting && current is not null ? Visibility.Visible : Visibility.Collapsed;
        libraryName.Text = current?.Name ?? "";
        emptyState.Visibility = Visibility.Collapsed;
        var query = search.Text; var selectedPath = current?.Path;
        SetSearching(!selecting && query.Trim().Length > 0);
        try
        {
            if (debounce) await Task.Delay(90);
            if (request != version) return;
            results.Items.Clear();
            if (selecting)
            {
                foreach (var library in libraries.Where(x => x.Name.Contains(query, StringComparison.CurrentCultureIgnoreCase) || x.Path.Contains(query, StringComparison.CurrentCultureIgnoreCase) || PathDisplay.Format(x.Path).Contains(query, StringComparison.CurrentCultureIgnoreCase) || x.Description.Contains(query, StringComparison.CurrentCultureIgnoreCase) || x.UpdatedAt.Contains(query, StringComparison.CurrentCultureIgnoreCase))) results.Items.Add(FullWidthRow(Views.LibraryItem(library, current?.Name == library.Name && current?.Path == library.Path)));
                if (results.Items.Count == 0) SetEmpty("未找到文献库，请先到主窗口添加文献库", "library");
            }
            else if (query.Trim().Length > 0 && selectedPath is not null)
            {
                progress.Visibility = Visibility.Visible; progress.IsActive = true;
                var references = await RustCore.Search(selectedPath, query);
                if (request != version) return;
                foreach (var reference in references) results.Items.Add(FullWidthRow(Views.ReferenceItem(reference)));
                if (references.Count == 0) SetEmpty("未找到匹配记录", "search");
            }
            if (results.Items.Count > 0) results.SelectedIndex = 0;
            ResizeContent();
        }
        catch (Exception error) { if (request == version) { SetErrorStatus(error); ResizeContent(); } }
        finally { if (request == version) SetSearching(false); }
    }
    private async void KeyDown(object sender, KeyRoutedEventArgs args)
    {
        if (pasting || pasteFailureDialog is not null || composing || (busy && args.Key != VirtualKey.Escape)) return;
        var control = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(VirtualKey.Control);
        var alt = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(VirtualKey.Menu);
        var shift = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(VirtualKey.Shift);
        var winLeft = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(VirtualKey.LeftWindows);
        var winRight = Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(VirtualKey.RightWindows);
        if (((control | alt | shift | winLeft | winRight) & Windows.UI.Core.CoreVirtualKeyStates.Down) != 0) return;
        switch (args.Key)
        {
            case VirtualKey.Escape: args.Handled = true; Hide(); break;
            case VirtualKey.Tab: args.Handled = true; selecting = true; search.Text = ""; await Refresh(); search.Focus(FocusState.Programmatic); break;
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
        if (pasting || busy || searching || item is null) return;
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
                pasting = true;
                // Release keyboard focus without invalidating this paste session.
                visible = false;
                AppWindow.Hide();
                await RustCore.Paste(reference.Key);
                if (session == lifecycle) Hide();
            }
        }
        catch (Exception error)
        {
            if (request != version) return;
            if (item.Tag is Reference reference)
            {
                // Copy automatically, even when key injection failed before writing
                // the clipboard. The dialog reports only the original paste error.
                try
                {
                    await RustCore.Copy(reference.Key);
                    if (session != lifecycle || disposed) return;
                    failedKey = null; fallback.Visibility = Visibility.Collapsed;
                }
                catch
                {
                    if (session != lifecycle || disposed) return;
                    failedKey = reference.Key; fallback.Visibility = Visibility.Visible;
                }
                visible = true;
                Activate(); SetErrorStatus(error);
                // A compact helper needs enough room for a native ContentDialog.
                Resize(360);
                var dialog = Localized.Dialog(new ContentDialog { XamlRoot = root.XamlRoot }, "错误");
                Localized.Error(dialog, ContentControl.ContentProperty, error);
                pasteFailureDialog = dialog;
                try { await Views.ShowDialog(dialog); }
                catch (Exception dialogError)
                {
                    // The window can close while WinUI prepares the dialog. Keep
                    // the original inline paste error rather than escaping an
                    // async event handler or replacing it with a dialog failure.
                    System.Diagnostics.Debug.WriteLine(dialogError);
                }
                finally { pasteFailureDialog = null; }
                if (session == lifecycle && !disposed && visible)
                {
                    ResizeContent(); search.Focus(FocusState.Programmatic);
                }
            }
            else { Activate(); search.Focus(FocusState.Programmatic); SetErrorStatus(error); ResizeContent(); }
        }
        finally { pasting = false; busy = false; }
    }
    private void SetErrorStatus(Exception value) { Localized.Error(status, TextBlock.TextProperty, value); errorBar.Visibility = Visibility.Visible; ResizeContent(); }
    private void SetEmpty(string key, string icon)
    {
        Localized.Bind(emptyLabel, TextBlock.TextProperty, key); emptyIcon.Icon = icon; emptyState.Visibility = Visibility.Visible;
    }
    private void SetStatus(string text) { Localized.BindValue(status, TextBlock.TextProperty, () => text); errorBar.Visibility = text.Length == 0 ? Visibility.Collapsed : Visibility.Visible; }
    private void SetSearching(bool value)
    {
        searching = value;
        progress.IsActive = listProgress.IsActive = value;
        progress.Visibility = listProgress.Visibility = value ? Visibility.Visible : Visibility.Collapsed;
        results.Visibility = value ? Visibility.Collapsed : Visibility.Visible;
        if (value) emptyState.Visibility = Visibility.Collapsed;
        ResizeContent();
    }
    private void ResizeContent()
    {
        if (!visible) return;
        var rows = results.Items.Count;
        var list = searching || emptyState.Visibility == Visibility.Visible ? EmptyRow : rows == 0 ? 0 : Math.Min(MaxList, rows * ((selecting ? LibraryRow : ReferenceRow) + 2) + ListPadding);
        listArea.Visibility = list > 0 ? Visibility.Visible : Visibility.Collapsed;
        Resize(Header + list + (errorBar.Visibility == Visibility.Visible ? StatusRow : 0));
    }
    private void Resize(double height)
    {
        var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this); var scale = WindowInterop.GetDpiForWindow(hwnd) / 96.0;
        var area = Area();
        var width = Math.Min(PanelWidth * scale, area.Width - 40 * scale);
        AppWindow.ResizeClient(new Windows.Graphics.SizeInt32((int)Math.Max(320, width), (int)Math.Min(height * scale, area.Height - 120 * scale)));
        // Keep the helper centred on its display after its content height changes.
        Place(area);
    }
    private Windows.Graphics.RectInt32 Area() => DisplayArea.GetFromPoint(Cursor(), DisplayAreaFallback.Primary).WorkArea;
    private static Windows.Graphics.PointInt32 Cursor()
    {
        WindowInterop.GetCursorPos(out var point); return new Windows.Graphics.PointInt32(point.X, point.Y);
    }
    private void Place(Windows.Graphics.RectInt32 area)
    {
        var size = AppWindow.Size;
        var x = area.X + (area.Width - size.Width) / 2;
        var y = area.Y + area.Height / 6;
        AppWindow.Move(new Windows.Graphics.PointInt32(x, y));
    }
    private void Position()
    {
        Resize(Header);
        // Moving across monitors can change DPI and therefore the final pixel width.
        Place(Area());
    }
    public void Dispose() { disposed = true; L10n.Changed -= LocalizeTitle; Hide(); }
}
