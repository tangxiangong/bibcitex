using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Windows.System;

namespace BibCiTeX;

/// <summary>Independent tray workbench; shares only data and service operations.</summary>
internal sealed class TrayWindow : Window
{
    private readonly Grid root = new() { Padding = new Thickness(16), RowSpacing = 12 };
    private readonly LibraryPicker library = new() { Width = 150, PlaceholderText = L10n.Text("文献库"), VerticalAlignment = VerticalAlignment.Center };
    private readonly TextBox search = new() { PlaceholderText = L10n.Text("搜索文献"), MinHeight = 34 };
    private readonly SearchSession searchSession = new("trayAdvancedSearch");
    private readonly ComboBox type = new() { MinWidth = 110 };
    private readonly ComboBox field = new() { MinWidth = 110 };
    private readonly ListView references = new() { SelectionMode = ListViewSelectionMode.Extended };
    private readonly Border detailPanel;
    private readonly ToggleButton detailToggle;
    private readonly Button closeDetail;
    private readonly StackPanel detail = new() { Spacing = 12, Padding = new Thickness(14) };
    private readonly TextBlock count = Views.Text(L10n.References(0), 12);
    private readonly TextBlock empty = Views.LocalizedText("暂无可显示的文献");
    private readonly TextBlock error = new() { TextWrapping = TextWrapping.Wrap, MaxLines = 3, Visibility = Visibility.Collapsed };
    private readonly Grid errorBar = new() { ColumnSpacing = 8, Visibility = Visibility.Collapsed };
    private readonly TextBlock detailEmpty = Views.LocalizedText("选择一条文献查看字段和操作", 13);
    private readonly Border detailTitle = new() { VerticalAlignment = VerticalAlignment.Center, Child = Views.LocalizedText("文献详情", 14) };
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
        TagManager.Changed += TagsChanged;
        Closed += (_, _) => TagManager.Changed -= TagsChanged;
        Views.LocalizeTextBox(search);
        searchSession.Attach(search);
        // searchBorder already carries the field's surface and focus ring.
        Views.PlainTextBox(search);
        for (var i = 0; i < 3; i++) root.RowDefinitions.Add(new() { Height = GridLength.Auto });
        root.RowDefinitions.Add(new()); root.RowDefinitions.Add(new() { Height = GridLength.Auto });
        var header = new Grid { ColumnSpacing = 10 };
        header.ColumnDefinitions.Add(new() { Width = GridLength.Auto }); header.ColumnDefinitions.Add(new());
        header.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        header.Children.Add(new TextBlock { Text = "BibCiTeX", FontSize = 14, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold, VerticalAlignment = VerticalAlignment.Center });
        detailToggle = (ToggleButton)Microsoft.UI.Xaml.Markup.XamlReader.Load("""
            <ToggleButton xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                          xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Padding="8" MinWidth="32" MinHeight="32">
                <ToggleButton.Resources>
                    <SolidColorBrush x:Key="ToggleButtonBackgroundChecked" Color="{ThemeResource SubtleFillColorSecondary}" />
                    <SolidColorBrush x:Key="ToggleButtonBackgroundCheckedPointerOver" Color="{ThemeResource SubtleFillColorTertiary}" />
                    <SolidColorBrush x:Key="ToggleButtonBackgroundCheckedPressed" Color="{ThemeResource SubtleFillColorSecondary}" />
                    <SolidColorBrush x:Key="ToggleButtonBackgroundCheckedDisabled" Color="{ThemeResource ControlFillColorDisabled}" />
                    <SolidColorBrush x:Key="ToggleButtonForegroundChecked" Color="{ThemeResource TextFillColorPrimary}" />
                    <SolidColorBrush x:Key="ToggleButtonForegroundCheckedPointerOver" Color="{ThemeResource TextFillColorPrimary}" />
                    <SolidColorBrush x:Key="ToggleButtonForegroundCheckedPressed" Color="{ThemeResource TextFillColorSecondary}" />
                    <SolidColorBrush x:Key="ToggleButtonForegroundCheckedDisabled" Color="{ThemeResource TextFillColorDisabled}" />
                    <SolidColorBrush x:Key="ToggleButtonBorderBrushChecked" Color="{ThemeResource ControlStrokeColorDefault}" />
                    <SolidColorBrush x:Key="ToggleButtonBorderBrushCheckedPointerOver" Color="{ThemeResource ControlStrokeColorDefault}" />
                    <SolidColorBrush x:Key="ToggleButtonBorderBrushCheckedPressed" Color="{ThemeResource ControlStrokeColorDefault}" />
                    <SolidColorBrush x:Key="ToggleButtonBorderBrushCheckedDisabled" Color="{ThemeResource ControlStrokeColorDefault}" />
                </ToggleButton.Resources>
            </ToggleButton>
            """);
        detailToggle.Content = new SvgIcon("panelRightOpen");
        Localized.Name(detailToggle, "文献详情"); Localized.Tooltip(detailToggle, "文献详情");
        detailToggle.Click += (_, _) => SetDetailVisible(detailToggle.IsChecked == true);
        var manageTags = Views.Button("tag", "标签管理", async () => { try { await TagManager.Show(root.XamlRoot); } catch (Exception ex) { ShowError(ex); } });
        var buttons = Views.Row(TagManager.BatchControls(references, out var endBatchSelection, nativeMultiple: true, cite: selected => CopyCitation(selected.Select(reference => reference.Key))), manageTags, detailToggle,
            Views.Button("refresh", "刷新", () => _ = Reload()),
            Views.Button("externalLink", "显示窗口", () => { Hide(); showMain(); }), Views.Button("x", "关闭", Hide));
        // One capsule for the toolbar, as on the macOS tray.
        foreach (var button in buttons.Children.OfType<Control>()) button.CornerRadius = new CornerRadius(16);
        buttons.Spacing = 4;
        var toolbar = Views.Capsule(buttons);
        Grid.SetColumn(toolbar, 2); header.Children.Add(toolbar); root.Children.Add(header);
        // One outer search field, with a separately focusable library picker inside its trailing edge.
        var searchContainer = new Grid();
        searchContainer.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        searchContainer.ColumnDefinitions.Add(new());
        searchContainer.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        search.BorderThickness = new Thickness(0);
        search.MinHeight = 32;
        search.Background = new SolidColorBrush(Microsoft.UI.Colors.Transparent);
        searchContainer.Children.Add(new SvgIcon("search", 15) { Margin = new Thickness(10, 0, 0, 0), VerticalAlignment = VerticalAlignment.Center });
        Grid.SetColumn(search, 1); searchContainer.Children.Add(search);
        var libraryArea = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 6, Margin = new Thickness(0, 2, 4, 2), VerticalAlignment = VerticalAlignment.Center };
        var separator = (Border)Microsoft.UI.Xaml.Markup.XamlReader.Load("""
            <Border xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                    Width="1" Height="16" Margin="0,0,4,0"
                    Background="{ThemeResource DividerStrokeColorDefaultBrush}" />
            """);
        libraryArea.Children.Add(separator);
        libraryArea.Children.Add(new SvgIcon("library", 13) { VerticalAlignment = VerticalAlignment.Center, Opacity = .7 });
        libraryArea.Children.Add(library);
        Localized.Tooltip(library, "文献库");
        Grid.SetColumn(libraryArea, 2); searchContainer.Children.Add(libraryArea);
        var searchBorder = (Border)Microsoft.UI.Xaml.Markup.XamlReader.Load("""
            <Border xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                    MinHeight="34" BorderThickness="1" CornerRadius="8"
                    BorderBrush="{ThemeResource ControlStrokeColorDefaultBrush}"
                    Background="{ThemeResource ControlFillColorDefaultBrush}" />
            """);
        searchBorder.Child = searchContainer;
        void SearchBorder() => searchBorder.BorderBrush = Views.Brush(search.FocusState != FocusState.Unfocused ? "SystemControlForegroundAccentBrush" : "ControlStrokeColorDefaultBrush");
        search.GotFocus += (_, _) => SearchBorder(); search.LostFocus += (_, _) => SearchBorder(); searchBorder.ActualThemeChanged += (_, _) => SearchBorder();
        Grid.SetRow(searchBorder, 1); root.Children.Add(searchBorder);
        foreach (var (label, value) in Reference.Types) type.Items.Add(Localized.ComboItem(label, value));
        foreach (var (label, value) in new[] { ("全部字段", "all"), ("作者", "author"), ("标题", "title"), ("期刊", "journal"), ("年份", "year") }) field.Items.Add(Localized.ComboItem(label, value));
        type.SelectedIndex = field.SelectedIndex = 0;
        var filters = new Grid { ColumnSpacing = 8 };
        filters.ColumnDefinitions.Add(new()); filters.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        var advanced = new ToggleButton { Content = L10n.Text("高级搜索"), IsChecked = searchSession.Advanced };
        Localized.Name(advanced, "高级搜索");
        var conditions = new Button { Content = L10n.Text("条件"), IsEnabled = searchSession.Advanced };
        conditions.Click += async (_, _) =>
        {
            try
            {
                version++; references.Items.Clear(); ShowDetail(null);
                if (await AdvancedSearchEditor.Show(root.XamlRoot, search.Text, await searchSession.AnalyzeDraft(search.Text)) is { } query)
                { searchSession.Accept(query); search.Text = query.text; }
                await Search();
            }
            catch (Exception ex) { ShowError(ex); }
        };
        advanced.Click += async (_, _) =>
        {
            try { searchSession.Switch(advanced.IsChecked == true, search, field, type); conditions.IsEnabled = searchSession.Advanced; await Search(); }
            catch (Exception ex) { advanced.IsChecked = searchSession.Advanced; ShowError(ex); }
        };
        var filterControls = new ScrollViewer { Content = Views.Row(advanced, conditions, type, field),
            HorizontalScrollMode = ScrollMode.Enabled, HorizontalScrollBarVisibility = ScrollBarVisibility.Hidden,
            VerticalScrollMode = ScrollMode.Disabled, VerticalScrollBarVisibility = ScrollBarVisibility.Disabled, ZoomMode = ZoomMode.Disabled };
        filters.Children.Add(filterControls);
        var counter = Views.Row(progress, count); counter.HorizontalAlignment = HorizontalAlignment.Right; counter.VerticalAlignment = VerticalAlignment.Center;
        Grid.SetColumn(counter, 1); filters.Children.Add(counter); Grid.SetRow(filters, 2); root.Children.Add(filters);
        var body = new Grid { ColumnSpacing = 0 };
        body.Children.Add(references); empty.HorizontalAlignment = HorizontalAlignment.Center; empty.VerticalAlignment = VerticalAlignment.Center; body.Children.Add(empty);
        var reading = new Grid();
        reading.Children.Add(Views.HideScrollbars(new ScrollViewer { Content = detail, VerticalScrollBarVisibility = ScrollBarVisibility.Auto, HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled }));
        detailEmpty.HorizontalAlignment = HorizontalAlignment.Center; detailEmpty.VerticalAlignment = VerticalAlignment.Center; detailEmpty.Margin = new Thickness(16); detailEmpty.Opacity = .65; reading.Children.Add(detailEmpty);
        var floatingContent = new Grid();
        floatingContent.RowDefinitions.Add(new() { Height = GridLength.Auto }); floatingContent.RowDefinitions.Add(new());
        var detailHeader = new Grid { Padding = new Thickness(14, 8, 8, 8) };
        detailHeader.ColumnDefinitions.Add(new()); detailHeader.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        detailHeader.Children.Add(detailTitle);
        closeDetail = Views.Button("x", "关闭", () => SetDetailVisible(false)); Grid.SetColumn(closeDetail, 1); detailHeader.Children.Add(closeDetail);
        floatingContent.Children.Add(detailHeader); Grid.SetRow(reading, 1); floatingContent.Children.Add(reading);
        detailPanel = (Border)Microsoft.UI.Xaml.Markup.XamlReader.Load("""
            <Border xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                    Width="282" Margin="12" HorizontalAlignment="Right" CornerRadius="16" BorderThickness="1"
                    Background="{ThemeResource SolidBackgroundFillColorBaseBrush}"
                    BorderBrush="{ThemeResource SurfaceStrokeColorDefaultBrush}" Visibility="Collapsed" />
            """);
        detailPanel.Child = floatingContent;
        detailPanel.Shadow = new ThemeShadow();
        detailPanel.Translation = new System.Numerics.Vector3(0, 0, 24);
        body.Children.Add(detailPanel);
        var bodyFrame = new Grid(); bodyFrame.RowDefinitions.Add(new() { Height = GridLength.Auto }); bodyFrame.RowDefinitions.Add(new());
        bodyFrame.Children.Add(Views.Divider()); Grid.SetRow(body, 1); bodyFrame.Children.Add(body); Grid.SetRow(bodyFrame, 3); root.Children.Add(bodyFrame);
        errorBar.ColumnDefinitions.Add(new() { Width = GridLength.Auto }); errorBar.ColumnDefinitions.Add(new()); errorBar.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        errorBar.Children.Add(new SvgIcon("alert", 14)); Views.ThemeForeground(error, "SystemFillColorCriticalBrush");
        Grid.SetColumn(error, 1); errorBar.Children.Add(error);
        var closeError = Views.Button("x", "关闭", () => SetError(null)); Grid.SetColumn(closeError, 2); errorBar.Children.Add(closeError);
        Grid.SetRow(errorBar, 4); root.Children.Add(errorBar);
        Localized.Name(library, "文献库"); Localized.Name(search, "搜索文献");
        Localized.Name(type, "类型筛选"); Localized.Name(field, "字段筛选");
        Localized.Bind(search, TextBox.PlaceholderTextProperty, "搜索文献");
        Localized.Bind(library, LibraryPicker.PlaceholderTextProperty, "文献库");
        Localized.Count(count, 0);
        WindowInterop.LocalizeSystemMenu(this);
        // Like the helper, the tray never shows scrollbars.
        Views.HideScrollbars(references);
        Content = root; ShowDetail(null);
        library.SelectionChanged += () => { Localized.BindValue(library, ToolTipService.ToolTipProperty, () => (library.SelectedItem as Library)?.Name ?? L10n.Text("文献库")); if (!updating) _ = Search(); };
        type.SelectionChanged += (_, _) => _ = Search(); field.SelectionChanged += (_, _) => _ = Search();
        search.TextCompositionStarted += (_, _) => { composing = true; searchSession.Invalidate(); version++; loadedVersion = -1; references.Items.Clear(); ShowDetail(null); };
        search.TextCompositionEnded += (_, _) => { composing = false; _ = Search(); };
        search.TextChanged += (_, _) => { searchSession.TextEdited(search.Text); if (!composing) _ = Search(true); };
        search.PreviewKeyDown += SearchKeyDown;
        root.PreviewKeyDown += (_, args) => {
            if (!composing && !TagManager.IsEditing(root.XamlRoot) && args.Key == VirtualKey.Escape && !Modified()) {
                if (FocusManager.GetFocusedElement(root.XamlRoot) is TextBox editor && !ReferenceEquals(editor, search)) return;
                if (TagManager.HasBlockingPopup(root.XamlRoot)) return;
                args.Handled = true;
                if (endBatchSelection()) return;
                if (detailPanel.Visibility == Visibility.Visible) SetDetailVisible(false); else Hide();
            }
        };
        references.SelectionChanged += (_, _) => ShowDetail(loadedVersion == version ? (references.SelectedItem as ListViewItem)?.Tag as Reference : null);
        Activated += (_, args) => { if (args.WindowActivationState == WindowActivationState.Deactivated) Hide(); };
        AppWindow.Closing += (_, args) => { if (!disposed) { args.Cancel = true; Hide(); } };
    }

    internal void Toggle()
    {
        if (visible) { Hide(); return; }
        App.Helper?.Hide();
        SetDetailVisible(false, restoreFocus: false);
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
        var anchor = TrayIcon.Bounds;
        if (anchor is { } icon) { point.X = icon.X + icon.Width / 2; point.Y = icon.Y + icon.Height / 2; }
        var area = DisplayArea.GetFromPoint(new Windows.Graphics.PointInt32(point.X, point.Y), DisplayAreaFallback.Primary).WorkArea;
        var scale = WindowInterop.GetDpiForWindow(WinRT.Interop.WindowNative.GetWindowHandle(this)) / 96.0;
        var width = (int)Math.Min(680 * scale, area.Width - 16 * scale);
        var height = (int)Math.Min(580 * scale, area.Height - 16 * scale);
        AppWindow.ResizeClient(new Windows.Graphics.SizeInt32(width, height));
        var size = AppWindow.Size;
        var x = Math.Clamp(point.X - size.Width / 2, area.X + (int)(8 * scale), Math.Max(area.X + (int)(8 * scale), area.X + area.Width - size.Width - (int)(8 * scale)));
        var y = point.Y < area.Y + area.Height / 2 ? area.Y + (int)(6 * scale) : area.Y + area.Height - size.Height - (int)(6 * scale);
        AppWindow.Move(new Windows.Graphics.PointInt32(x, y));
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
                ?? NativeSettings.Values["trayLibrary"] as string;
            updating = true;
            library.ItemsSource = rows; library.IsEnabled = rows.Count > 0;
            library.SelectedItem = rows.FirstOrDefault(x => x.Name == saved) ?? rows.FirstOrDefault(x => x.Available);
            Localized.Bind(library, LibraryPicker.PlaceholderTextProperty, rows.Count == 0 ? "暂无文献库" : "文献库");
            updating = false;
            await Search();
        }
        catch (Exception ex) { if (visible && request == reloadVersion) { ClearResults(); ShowError(ex); SetBusy(false); } }
        finally { updating = false; }
    }
    private void TagsChanged() { searchSession.Invalidate(); version++; if (TagManager.IsEditing(root.XamlRoot)) return; references.Items.Clear(); ShowDetail(null); if (visible) _ = Reload(); }

    private async Task Search(bool debounce = false)
    {
        if (!visible || updating || searchSession.Restoring) return;
        searchSession.Invalidate();
        var request = ++version;
        loadedVersion = -1; references.Items.Clear();
        var selected = ((references.SelectedItem as ListViewItem)?.Tag as Reference)?.Id;
        var current = library.SelectedItem as Library;
        var query = search.Text;
        var selectedField = (field.SelectedItem as ComboBoxItem)?.Tag as string ?? "all";
        var selectedType = (type.SelectedItem as ComboBoxItem)?.Tag as string ?? "all";
        SetBusy(true); ShowDetail(null); SetError(null);
        if (current is null) { ClearResults(); SetBusy(false); return; }
        NativeSettings.Values["trayLibrary"] = current.Name;
        if (!current.Available) { ClearResults(); SetBusy(false); return; }
        try
        {
            if (debounce) await Task.Delay(100);
            if (!visible || request != version) return;
            var response = await searchSession.Search(current.Path, query, selectedField, selectedType, request);
            var rows = response.references.Select(x => new Reference(x) { Evidence = response.evidence.FirstOrDefault(e => e.reference_id == x.id) }).ToList();
            if (!visible || request != version) return;
            searchSession.Publish(response); searchSession.SyncProjection(search);
            SetError(SearchSession.Diagnostic(response.analysis));
            references.Items.Clear(); loadedVersion = request;
            foreach (var reference in rows)
            {
                var item = ReferenceRow(reference); references.Items.Add(item);
                if (reference.Id == selected) references.SelectedItem = item;
            }
            if (references.SelectionMode == ListViewSelectionMode.Single && references.SelectedItem is null && rows.Count > 0) references.SelectedIndex = 0;
            Localized.Count(count, rows.Count); empty.Visibility = rows.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
        }
        catch (Exception ex) { if (visible && request == version) { ClearResults(); ShowError(ex); } }
        finally { if (request == version) SetBusy(false); }
    }
    private ListViewItem ReferenceRow(Reference reference)
    {
        var row = new StackPanel { Spacing = 5, Padding = new Thickness(0, 7, 0, 7) };
        var heading = new Grid { ColumnSpacing = 12 };
        heading.ColumnDefinitions.Add(new());
        heading.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        heading.Children.Add(new ChunkText(reference.Chunks("title"), 14, reference.Title, false, "暂无标题", maxLines: 2, highlights: SearchHighlights.Ranges(reference, "title", reference.Title)));
        var key = Views.CiteKey(reference.Key, CopyKey);
        key.HorizontalAlignment = HorizontalAlignment.Right;
        // Keep unusually long keys from consuming the title's column.
        if (key.Content is StackPanel keyContent)
            foreach (var label in keyContent.Children.OfType<TextBlock>()) label.MaxWidth = 150;
        row.Children.Add(heading);
        if (reference.Authors.Length > 0) row.Children.Add(SearchHighlights.Authors(reference, 12));
        var metadata = new TextBlock { FontSize = 11, Opacity = .6, TextTrimming = TextTrimming.CharacterEllipsis };
        Localized.BindValue(metadata, TextBlock.TextProperty, () => string.Join(" · ", new[] { reference.TypeLabel, reference.Text("year"), reference.Venue }.Where(x => x.Length > 0)));
        row.Children.Add(metadata);
        var accessory = new StackPanel { Spacing = 6, HorizontalAlignment = HorizontalAlignment.Right, MaxWidth = 160, Margin = new Thickness(0, 7, 0, 7) };
        accessory.Children.Add(key);
        if (reference.Data.tags.Count > 0) accessory.Children.Add(TagManager.Chips(reference));
        var layout = new Grid { ColumnSpacing = 12 };
        layout.ColumnDefinitions.Add(new()); layout.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        layout.Children.Add(row); Grid.SetColumn(accessory, 1); layout.Children.Add(accessory);
        if (reference.Evidence is { } evidence) ToolTipService.SetToolTip(row, (evidence.fuzzy ? L10n.Text("拼写补充") + " (" + string.Join(", ", evidence.fuzzy_terms) + ") · " : "") + string.Join(" · ", evidence.fields.Select(x => L10n.Text(x))));
        var item = new ListViewItem { Content = layout, Tag = reference, HorizontalContentAlignment = HorizontalAlignment.Stretch, CornerRadius = new CornerRadius(Views.RowRadius) };
        var menu = new MenuFlyout();
        var tags = new MenuFlyoutItem { Text = L10n.Text("标签") };
        tags.Click += async (_, _) => { try { await TagManager.Show(item.XamlRoot, reference.Data.library_uid, reference.Id, reference.Key, item); } catch (Exception ex) { ShowError(ex); } };
        menu.Items.Add(tags); item.ContextFlyout = menu;
        Localized.BindValue(item, AutomationProperties.NameProperty, () => reference.Key + " " + reference.Title);
        return item;
    }
    private void SetDetailVisible(bool value, bool restoreFocus = true)
    {
        detailPanel.Visibility = value ? Visibility.Visible : Visibility.Collapsed;
        detailToggle.IsChecked = value;
        detailToggle.Content = new SvgIcon(value ? "panelRightClose" : "panelRightOpen");
        if (value) closeDetail.Focus(FocusState.Programmatic);
        else if (restoreFocus) detailToggle.Focus(FocusState.Programmatic);
    }
    private void ShowDetail(Reference? reference)
    {
        detail.Children.Clear();
        detailEmpty.Visibility = reference is null ? Visibility.Visible : Visibility.Collapsed;
        detailTitle.Child = reference is null ? Views.LocalizedText("文献详情", 14) : new ChunkText(reference.Chunks("title"), 14, reference.Title, true, "暂无标题", maxLines: 2, highlights: SearchHighlights.Ranges(reference, "title", reference.Title));
        if (reference is null) return;
        var tags = new Button { Content = L10n.Text("标签") };
        tags.Click += async (_, _) => { try { await TagManager.Show(root.XamlRoot, reference.Data.library_uid, reference.Id, reference.Key, tags); } catch (Exception ex) { ShowError(ex); } };
        detail.Children.Add(tags);
        foreach (var (label, value) in new[] { ("引用键", reference.Key), ("作者", reference.Authors), ("类型", reference.TypeLabel), ("年份", reference.Text("year")), ("期刊", reference.Venue), ("DOI", reference.Text("doi")), ("URL", reference.Text("url")) })
        {
            if (value.Length == 0) continue;
            var block = new StackPanel { Spacing = 3 };
            var heading = Views.LocalizedText(label, 12); heading.Opacity = .65;
            block.Children.Add(label == "引用键" ? Views.Caption(heading, Views.CopyButton("copy", "引用", () => CopyKey(reference.Key), compact: true)) : heading);
            detail.Children.Add(block);
            if (label == "DOI") { block.Children.Add(Views.Link(value, 13, "打开 DOI", () => _ = OpenUrl(value.StartsWith("http", StringComparison.OrdinalIgnoreCase) ? value : "https://doi.org/" + value))); continue; }
            if (label == "URL") { block.Children.Add(Views.Link(value, 13, "打开 URL", () => _ = OpenUrl(value))); continue; }
            var text = Views.Text(value, 13);
            if (label == "类型") Localized.BindValue(text, TextBlock.TextProperty, () => reference.TypeLabel);
            text.IsTextSelectionEnabled = true; Views.SelectableText(text); block.Children.Add(text);
        }
        if (reference.Text("abstract_").Length > 0)
        {
            detail.Children.Add(Views.LocalizedText("摘要", 12));
            detail.Children.Add(new ChunkText(reference.Chunks("abstract_"), 13, reference.Text("abstract_"), true));
        }
        var sourceBlock = new StackPanel { Spacing = 3 };
        var sourceHeading = Views.Text("BibTeX", 12); sourceHeading.Opacity = .65; sourceBlock.Children.Add(Views.Caption(sourceHeading, CopyButton("clipboard", "复制 BibTeX", reference.Text("source"))));
        var source = Views.Text(reference.Text("source"), 12); source.IsTextSelectionEnabled = true; Views.SelectableText(source); source.FontFamily = new FontFamily("Cascadia Mono");
        sourceBlock.Children.Add(source); detail.Children.Add(sourceBlock);
    }
    private async Task<bool> CopyCitation(IEnumerable<string> keys)
    {
        var selected = keys.ToList();
        if (selected.Count == 0) return false;
        var revision = version;
        var current = session;
        try { var text = await RustCore.FormatCitation(selected); return !disposed && visible && current == session && revision == version && await CopyValue(text); }
        catch (Exception error) { if (!disposed && visible && current == session && revision == version) ShowError(error); return false; }
    }
    private Task<bool> CopyKey(string key) => CopyCitation([key]);
    private async Task<bool> CopyValue(string value)
    {
        var current = session;
        var revision = version;
        try { if (loadedVersion != revision || !await searchSession.Validate()) { await Search(); return false; } if (!visible || current != session || revision != version || loadedVersion != revision) return false; await RustCore.Copy(value); return visible && current == session; }
        catch (Exception ex) { if (visible && current == session) ShowError(ex); return false; }
    }
    private Button CopyButton(string icon, string label, string value)
        => Views.CopyButton(icon, label, () => CopyValue(value), compact: true);
    private async Task OpenUrl(string value)
    {
        var current = session;
        try { if (!await searchSession.Validate()) { await Search(); return; } var uri = new Uri(value); if (uri.Scheme is not ("http" or "https")) throw new LocalizedException("仅支持 HTTP/HTTPS 链接"); if (!await Launcher.LaunchUriAsync(uri)) throw new IOException(value); }
        catch (Exception ex) { if (visible && current == session) ShowError(ex); }
    }
    private void ClearResults() { references.Items.Clear(); Localized.Count(count, 0); empty.Visibility = Visibility.Visible; ShowDetail(null); }
    private void SetBusy(bool value) { loading = value; progress.IsActive = value; progress.Visibility = value ? Visibility.Visible : Visibility.Collapsed; empty.Visibility = !value && references.Items.Count == 0 ? Visibility.Visible : Visibility.Collapsed; }
    private void ShowError(Exception value) { Localized.Error(error, TextBlock.TextProperty, value); error.Visibility = errorBar.Visibility = Visibility.Visible; }
    private void SetError(string? value) { Localized.BindValue(error, TextBlock.TextProperty, () => value ?? ""); error.Visibility = errorBar.Visibility = string.IsNullOrEmpty(value) ? Visibility.Collapsed : Visibility.Visible; }
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
