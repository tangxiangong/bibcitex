using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Windows.System;

namespace BibCiTeX;

/// <summary>Independent tray workbench; shares only data and service operations.</summary>
internal sealed class TrayWindow : Window
{
    private readonly Grid root = new() { Padding = new Thickness(16), RowSpacing = 12 };
    private readonly ComboBox library = new() { Width = 144, FontSize = 12, Padding = new Thickness(6, 0, 24, 0), MinHeight = 28, PlaceholderText = L10n.Text("文献库"), BorderThickness = new Thickness(0), Background = new SolidColorBrush(Microsoft.UI.Colors.Transparent), VerticalAlignment = VerticalAlignment.Center };
    private readonly TextBox search = new() { PlaceholderText = L10n.Text("搜索文献"), MinHeight = 34 };
    private readonly ComboBox type = new() { MinWidth = 110 };
    private readonly ComboBox field = new() { MinWidth = 110 };
    private readonly ListView references = new() { SelectionMode = ListViewSelectionMode.Single };
    private readonly StackPanel detail = new() { Spacing = 12, Padding = new Thickness(14, 0, 0, 0) };
    private readonly StackPanel actions = new() { Orientation = Orientation.Horizontal, Spacing = 8, Padding = new Thickness(14, 10, 0, 0) };
    private readonly TextBlock count = Views.Text(L10n.References(0), 12);
    private readonly TextBlock empty = Views.LocalizedText("暂无可显示的文献");
    private readonly TextBlock error = new() { TextWrapping = TextWrapping.Wrap, MaxLines = 3, Visibility = Visibility.Collapsed };
    private readonly ProgressRing progress = new() { Width = 16, Height = 16, IsActive = false, Visibility = Visibility.Collapsed };
    private int version, reloadVersion, session, loadedVersion = -1;
    private bool visible, disposed, composing, updating, loading;

    internal TrayWindow(Action showMain)
    {
        Title = "BibCiTeX";
        SystemBackdrop = new DesktopAcrylicBackdrop();
        var presenter = OverlappedPresenter.CreateForToolWindow();
        presenter.SetBorderAndTitleBar(false, false);
        presenter.IsAlwaysOnTop = true; presenter.IsResizable = false;
        presenter.IsMinimizable = false; presenter.IsMaximizable = false;
        AppWindow.SetPresenter(presenter); AppWindow.IsShownInSwitchers = false;
        root.RequestedTheme = App.Theme;
        Localized.BindValue(root, FrameworkElement.LanguageProperty, () => L10n.Language);
        Views.LocalizeTextBox(search);
        for (var i = 0; i < 3; i++) root.RowDefinitions.Add(new() { Height = GridLength.Auto });
        root.RowDefinitions.Add(new()); root.RowDefinitions.Add(new() { Height = GridLength.Auto });
        var header = new Grid { ColumnSpacing = 10 };
        header.ColumnDefinitions.Add(new() { Width = GridLength.Auto }); header.ColumnDefinitions.Add(new());
        header.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        header.Children.Add(new TextBlock { Text = "BibCiTeX", FontSize = 14, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold, VerticalAlignment = VerticalAlignment.Center });
        var buttons = Views.Row(Views.Button("refresh", "刷新", () => _ = Reload()),
            Views.Button("externalLink", "显示窗口", () => { Hide(); showMain(); }), Views.Button("x", "关闭", Hide));
        Grid.SetColumn(buttons, 2); header.Children.Add(buttons); root.Children.Add(header);
        // One outer search field, with a separately focusable library picker inside its trailing edge.
        var searchContainer = new Grid();
        searchContainer.ColumnDefinitions.Add(new());
        searchContainer.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        search.BorderThickness = new Thickness(0);
        search.MinHeight = 32;
        search.Background = new SolidColorBrush(Microsoft.UI.Colors.Transparent);
        searchContainer.Children.Add(search);
        var libraryArea = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 6, Margin = new Thickness(0, 2, 4, 2), VerticalAlignment = VerticalAlignment.Center };
        var separator = (Border)Microsoft.UI.Xaml.Markup.XamlReader.Load("""
            <Border xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                    Width="1" Height="16" Margin="0,0,4,0"
                    Background="{ThemeResource ControlStrokeColorDefaultBrush}" />
            """);
        libraryArea.Children.Add(separator);
        libraryArea.Children.Add(new SvgIcon("library", 13) { VerticalAlignment = VerticalAlignment.Center, Opacity = .7 });
        libraryArea.Children.Add(library);
        Localized.Tooltip(library, "文献库");
        Grid.SetColumn(libraryArea, 1); searchContainer.Children.Add(libraryArea);
        var searchBorder = (Border)Microsoft.UI.Xaml.Markup.XamlReader.Load("""
            <Border xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                    MinHeight="34" BorderThickness="1" CornerRadius="8"
                    BorderBrush="{ThemeResource ControlStrokeColorDefaultBrush}"
                    Background="{ThemeResource ControlFillColorDefaultBrush}" />
            """);
        searchBorder.Child = searchContainer;
        Grid.SetRow(searchBorder, 1); root.Children.Add(searchBorder);
        foreach (var (label, value) in Reference.Types) type.Items.Add(LocalizedCombo(label, value));
        foreach (var (label, value) in new[] { ("全部字段", "all"), ("作者", "author"), ("标题", "title"), ("期刊", "journal"), ("年份", "year") }) field.Items.Add(LocalizedCombo(label, value));
        type.SelectedIndex = field.SelectedIndex = 0;
        var filters = new Grid { ColumnSpacing = 8 };
        filters.ColumnDefinitions.Add(new() { Width = GridLength.Auto }); filters.ColumnDefinitions.Add(new() { Width = GridLength.Auto }); filters.ColumnDefinitions.Add(new());
        filters.Children.Add(type); Grid.SetColumn(field, 1); filters.Children.Add(field);
        var counter = Views.Row(progress, count); counter.HorizontalAlignment = HorizontalAlignment.Right; counter.VerticalAlignment = VerticalAlignment.Center;
        Grid.SetColumn(counter, 2); filters.Children.Add(counter); Grid.SetRow(filters, 2); root.Children.Add(filters);
        var body = new Grid { ColumnSpacing = 12 };
        body.ColumnDefinitions.Add(new()); body.ColumnDefinitions.Add(new() { Width = new GridLength(270) });
        body.Children.Add(references); empty.HorizontalAlignment = HorizontalAlignment.Center; empty.VerticalAlignment = VerticalAlignment.Center; body.Children.Add(empty);
        var reading = new Grid(); reading.RowDefinitions.Add(new()); reading.RowDefinitions.Add(new() { Height = GridLength.Auto });
        reading.Children.Add(new ScrollViewer { Content = detail, HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled });
        Grid.SetRow(actions, 1); reading.Children.Add(actions); Grid.SetColumn(reading, 1); body.Children.Add(reading);
        Grid.SetRow(body, 3); root.Children.Add(body); Grid.SetRow(error, 4); root.Children.Add(error);
        Localized.Name(library, "文献库"); Localized.Name(search, "搜索文献");
        Localized.Name(type, "类型筛选"); Localized.Name(field, "字段筛选");
        Localized.Bind(search, TextBox.PlaceholderTextProperty, "搜索文献");
        Localized.Bind(library, ComboBox.PlaceholderTextProperty, "文献库");
        Localized.Count(count, 0);
        WindowInterop.LocalizeSystemMenu(this);
        Content = root; ShowDetail(null);
        library.SelectionChanged += (_, _) => { if (!updating) _ = Search(); };
        type.SelectionChanged += (_, _) => _ = Search(); field.SelectionChanged += (_, _) => _ = Search();
        search.TextCompositionStarted += (_, _) => composing = true;
        search.TextCompositionEnded += (_, _) => { composing = false; _ = Search(); };
        search.TextChanged += (_, _) => { if (!composing) _ = Search(true); };
        search.PreviewKeyDown += SearchKeyDown;
        root.PreviewKeyDown += (_, args) => { if (!composing && args.Key == VirtualKey.Escape && !Modified()) { args.Handled = true; Hide(); } };
        references.SelectionChanged += (_, _) => ShowDetail(loadedVersion == version ? (references.SelectedItem as ListViewItem)?.Tag as Reference : null);
        Activated += (_, args) => { if (args.WindowActivationState == WindowActivationState.Deactivated) Hide(); };
        AppWindow.Closing += (_, args) => { if (!disposed) { args.Cancel = true; Hide(); } };
    }

    internal void Toggle()
    {
        if (visible) { Hide(); return; }
        App.Helper?.Hide();
        Position(); visible = true; session++;
        AppWindow.Show(); Activate(); search.Focus(FocusState.Programmatic);
        _ = Reload();
    }
    internal void Hide()
    {
        visible = false; session++; version++; reloadVersion++; composing = false;
        AppWindow.Hide();
    }
    internal void Shutdown() { disposed = true; Hide(); Close(); }
    private void Position()
    {
        WindowInterop.GetCursorPos(out var point);
        var area = DisplayArea.GetFromPoint(new Windows.Graphics.PointInt32(point.X, point.Y), DisplayAreaFallback.Primary).WorkArea;
        var scale = WindowInterop.GetDpiForWindow(WinRT.Interop.WindowNative.GetWindowHandle(this)) / 96.0;
        var width = (int)Math.Min(680 * scale, area.Width - 16 * scale);
        var height = (int)Math.Min(580 * scale, area.Height - 16 * scale);
        AppWindow.ResizeClient(new Windows.Graphics.SizeInt32(width, height));
        var size = AppWindow.Size;
        AppWindow.Move(new Windows.Graphics.PointInt32(area.X + area.Width - size.Width - (int)(8 * scale), area.Y + area.Height - size.Height - (int)(8 * scale)));
    }
    private async Task Reload()
    {
        var request = ++reloadVersion;
        ++version; SetBusy(true); ShowDetail(null); SetError(null);
        try
        {
            var rows = await RustCore.Libraries();
            if (!visible || request != reloadVersion) return;
            var saved = (library.SelectedItem as Library)?.Name
                ?? Windows.Storage.ApplicationData.Current.LocalSettings.Values["trayLibrary"] as string;
            updating = true;
            library.ItemsSource = rows;
            library.SelectedItem = rows.FirstOrDefault(x => x.Name == saved) ?? rows.FirstOrDefault();
            Localized.Bind(library, ComboBox.PlaceholderTextProperty, rows.Count == 0 ? "暂无文献库" : "文献库");
            updating = false;
            await Search();
        }
        catch (Exception ex) { if (visible && request == reloadVersion) { ClearResults(); ShowError(ex); SetBusy(false); } }
        finally { updating = false; }
    }
    private async Task Search(bool debounce = false)
    {
        if (!visible || updating) return;
        var request = ++version;
        var selected = ((references.SelectedItem as ListViewItem)?.Tag as Reference)?.Id;
        var current = library.SelectedItem as Library;
        var query = search.Text;
        var selectedField = (field.SelectedItem as ComboBoxItem)?.Tag as string ?? "all";
        var selectedType = (type.SelectedItem as ComboBoxItem)?.Tag as string ?? "all";
        SetBusy(true); ShowDetail(null); SetError(null);
        if (current is null) { ClearResults(); SetBusy(false); return; }
        Windows.Storage.ApplicationData.Current.LocalSettings.Values["trayLibrary"] = current.Name;
        try
        {
            if (debounce) await Task.Delay(100);
            if (!visible || request != version) return;
            var rows = await RustCore.Search(current.Path, query, selectedField, selectedType);
            if (!visible || request != version) return;
            references.Items.Clear(); loadedVersion = request;
            foreach (var reference in rows)
            {
                var item = ReferenceRow(reference); references.Items.Add(item);
                if (reference.Id == selected) references.SelectedItem = item;
            }
            if (references.SelectedItem is null && rows.Count > 0) references.SelectedIndex = 0;
            Localized.Count(count, rows.Count); empty.Visibility = rows.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
        }
        catch (Exception ex) { if (visible && request == version) { ClearResults(); ShowError(ex); } }
        finally { if (request == version) SetBusy(false); }
    }
    private static ListViewItem ReferenceRow(Reference reference)
    {
        var row = new StackPanel { Spacing = 5, Padding = new Thickness(0, 7, 0, 7) };
        row.Children.Add(new TextBlock { Text = reference.Key, FontSize = 12, FontFamily = new FontFamily("Cascadia Mono"), TextTrimming = TextTrimming.CharacterEllipsis });
        row.Children.Add(new ChunkText(reference.Chunks("title"), 14, reference.Title, false, "暂无标题"));
        row.Children.Add(new TextBlock { Text = reference.Authors, FontSize = 12, Opacity = .7, TextTrimming = TextTrimming.CharacterEllipsis });
        var metadata = new TextBlock { FontSize = 11, Opacity = .6, TextTrimming = TextTrimming.CharacterEllipsis };
        Localized.BindValue(metadata, TextBlock.TextProperty, () => string.Join(" · ", new[] { reference.TypeLabel, reference.Text("year"), reference.Venue }.Where(x => x.Length > 0)));
        row.Children.Add(metadata);
        var item = new ListViewItem { Content = row, Tag = reference, HorizontalContentAlignment = HorizontalAlignment.Stretch };
        Localized.BindValue(item, AutomationProperties.NameProperty, () => reference.Key + " " + reference.Title);
        return item;
    }
    private static ComboBoxItem LocalizedCombo(string key, string value)
    {
        var item = new ComboBoxItem { Tag = value };
        Localized.Bind(item, ContentControl.ContentProperty, key);
        return item;
    }
    private void ShowDetail(Reference? reference)
    {
        detail.Children.Clear(); actions.Children.Clear();
        if (reference is null) { detail.Children.Add(Views.LocalizedText("选择一条文献查看字段和操作", 13)); return; }
        detail.Children.Add(Views.LocalizedText("文献详情", 12));
        detail.Children.Add(new ChunkText(reference.Chunks("title"), 16, reference.Title, true, "暂无标题"));
        foreach (var (label, value) in new[] { ("引用键", reference.Key), ("作者", reference.Authors), ("类型", reference.TypeLabel), ("年份", reference.Text("year")), ("期刊", reference.Venue), ("DOI", reference.Text("doi")), ("URL", reference.Text("url")) })
        {
            if (value.Length == 0) continue;
            var block = new StackPanel { Spacing = 3 };
            var heading = Views.LocalizedText(label, 12); heading.Opacity = .65; block.Children.Add(heading);
            var text = Views.Text(value, 13);
            if (label == "类型") Localized.BindValue(text, TextBlock.TextProperty, () => reference.TypeLabel);
            text.IsTextSelectionEnabled = true; Views.SelectableText(text); block.Children.Add(text); detail.Children.Add(block);
        }
        if (reference.Text("abstract_").Length > 0)
        {
            detail.Children.Add(Views.LocalizedText("摘要", 12));
            detail.Children.Add(new ChunkText(reference.Chunks("abstract_"), 13, reference.Text("abstract_"), true));
        }
        var source = Views.Text(reference.Text("source"), 12); source.IsTextSelectionEnabled = true; Views.SelectableText(source); source.FontFamily = new FontFamily("Cascadia Mono");
        detail.Children.Add(new Expander { Header = "BibTeX", Content = source, HorizontalAlignment = HorizontalAlignment.Stretch });
        actions.Children.Add(CopyButton("复制引用键", reference.Key));
        actions.Children.Add(CopyButton("复制 BibTeX", reference.Text("source")));
    }
    private Button CopyButton(string label, string value)
    {
        var button = new Button { Content = label, FontSize = 12, Padding = new Thickness(8, 6, 8, 6) };
        Localized.Bind(button, ContentControl.ContentProperty, label);
        button.Click += async (_, _) =>
        {
            var current = session;
            try
            {
                await RustCore.Copy(value);
                if (!visible || current != session) return;
                Localized.Bind(button, ContentControl.ContentProperty, "已复制");
                await Task.Delay(1500);
                if (visible && current == session) Localized.Bind(button, ContentControl.ContentProperty, label);
            }
            catch (Exception ex) { if (visible && current == session) ShowError(ex); }
        };
        Localized.Name(button, label);
        return button;
    }
    private void ClearResults() { references.Items.Clear(); Localized.Count(count, 0); empty.Visibility = Visibility.Visible; ShowDetail(null); }
    private void SetBusy(bool value) { loading = value; progress.IsActive = value; progress.Visibility = value ? Visibility.Visible : Visibility.Collapsed; }
    private void ShowError(Exception value) { Localized.Error(error, TextBlock.TextProperty, value); error.Visibility = Visibility.Visible; }
    private void SetError(string? value) { Localized.BindValue(error, TextBlock.TextProperty, () => value ?? ""); error.Visibility = value is null ? Visibility.Collapsed : Visibility.Visible; }
    private static bool Modified() => new[] { VirtualKey.Control, VirtualKey.Menu, VirtualKey.Shift, VirtualKey.LeftWindows, VirtualKey.RightWindows }
        .Any(key => (Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(key) & Windows.UI.Core.CoreVirtualKeyStates.Down) != 0);
    private void SearchKeyDown(object sender, KeyRoutedEventArgs args)
    {
        if (composing || Modified() || args.Key is not (VirtualKey.Up or VirtualKey.Down)) return;
        args.Handled = true;
        if (loading || loadedVersion != version || references.Items.Count == 0) return;
        var direction = args.Key == VirtualKey.Down ? 1 : -1;
        references.SelectedIndex = references.SelectedIndex < 0 ? (direction > 0 ? 0 : references.Items.Count - 1)
            : Math.Clamp(references.SelectedIndex + direction, 0, references.Items.Count - 1);
        references.ScrollIntoView(references.SelectedItem);
    }
}
