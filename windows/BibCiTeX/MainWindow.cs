using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Windows.Storage;
using Windows.Storage.Pickers;
using Windows.System;

namespace BibCiTeX;

internal sealed class MainWindow : Window
{
    private readonly Grid root = new();
    private readonly ListView libraries = new() { SelectionMode = ListViewSelectionMode.Single };
    private readonly ListView references = new() { SelectionMode = ListViewSelectionMode.Single };
    private readonly StackPanel detail = new() { Padding = new Thickness(16), Spacing = 16 };
    private readonly TextBlock heading = Views.LocalizedText("文献工作台", 20);
    private readonly TextBlock count = Views.Text(L10n.References(0), 12);
    private readonly TextBlock empty = Views.LocalizedText("暂无可显示的文献");
    private readonly StackPanel emptyState = new() { Spacing = 16, HorizontalAlignment = HorizontalAlignment.Center, VerticalAlignment = VerticalAlignment.Center };
    private readonly Image brand = new()
    {
        Source = new Microsoft.UI.Xaml.Media.Imaging.BitmapImage(new Uri("ms-appx:///Assets/favicon.png")),
        Width = 96, Height = 96, Stretch = Stretch.Uniform, Visibility = Visibility.Collapsed
    };
    private readonly TextBlock librariesEmpty = Views.LocalizedText("暂无文献库");
    private readonly TextBox search = new() { PlaceholderText = L10n.Text("搜索文献"), MinWidth = 160 };
    private readonly ComboBox field = new() { MinWidth = 110 };
    private readonly ComboBox type = new() { MinWidth = 110 };
    private readonly ProgressRing progress = new() { Width = 16, Height = 16, IsActive = false, Visibility = Visibility.Collapsed };
    private readonly Grid sidebar = new() { Padding = new Thickness(10, 12, 10, 8) };
    private readonly Grid inspector = new();
    private readonly TextBlock detailEmpty = Views.LocalizedText("选择一条文献查看字段和操作");
    private bool showSidebar = true, showInspector = true, loading, reloading;
    private double sidebarWidth = 220, inspectorWidth = 350;
    private ToggleButton sidebarToggle = null!, inspectorToggle = null!;
    private PaneThumb leftDivider = null!, rightDivider = null!;
    private Window? aboutWindow;
    private int reloadVersion;
    private Library? current;
    private int searchVersion;
    private int resultsVersion;
    private bool searchComposing;
    private GlobalShortcut? shortcut;
    private TrayIcon? tray;
    private bool quitting;
    private bool initialized;
    private readonly SemaphoreSlim dialogs = new(1, 1);

    internal MainWindow()
    {
        Title = "BibCiTeX"; SystemBackdrop = Microsoft.UI.Composition.SystemBackdrops.MicaController.IsSupported() ? new MicaBackdrop() : new DesktopAcrylicBackdrop();
        var initialScale = WindowInterop.GetDpiForWindow(WinRT.Interop.WindowNative.GetWindowHandle(this)) / 96.0;
        AppWindow.Resize(new Windows.Graphics.SizeInt32((int)(1200 * initialScale), (int)(800 * initialScale)));
        SetMinimumSize();
        root.RequestedTheme = App.Theme;
        Localized.BindValue(root, FrameworkElement.LanguageProperty, () => L10n.Language);
        Views.LocalizeTextBox(search);
        root.ColumnDefinitions.Add(new() { Width = new GridLength(220) });
        root.ColumnDefinitions.Add(new() { Width = new GridLength(5) });
        root.ColumnDefinitions.Add(new() { Width = new GridLength(1, GridUnitType.Star), MinWidth = 330 });
        root.ColumnDefinitions.Add(new() { Width = new GridLength(5) });
        root.ColumnDefinitions.Add(new() { Width = new GridLength(350) });
        root.RowDefinitions.Add(new() { Height = GridLength.Auto }); root.RowDefinitions.Add(new());
        var chrome = new StackPanel(); chrome.Children.Add(BuildMenu());
        var toolbar = new Grid { Padding = new Thickness(12, 4, 12, 8) };
        toolbar.ColumnDefinitions.Add(new() { Width = GridLength.Auto }); toolbar.ColumnDefinitions.Add(new()); toolbar.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        sidebarToggle = PaneToggle("panelLeftClose", "文献库", () => { showSidebar = sidebarToggle.IsChecked == true; LayoutPanes(); });
        inspectorToggle = PaneToggle("panelRightClose", "文献详情", () => { showInspector = inspectorToggle.IsChecked == true; LayoutPanes(); });
        toolbar.Children.Add(Views.Row(sidebarToggle, BrandImage(48)));
        var primary = Views.Row(Views.Button("search", "快捷助手", () => _ = App.Helper!.Toggle()),
            Views.Button("refresh", "刷新", () => _ = ReloadLibraries()), inspectorToggle);
        primary.VerticalAlignment = VerticalAlignment.Center; Grid.SetColumn(primary, 2); toolbar.Children.Add(primary);
        chrome.Children.Add(toolbar); chrome.Children.Add(Views.Divider());
        Grid.SetColumnSpan(chrome, 5); root.Children.Add(chrome);
        Localized.Bind(search, TextBox.PlaceholderTextProperty, "搜索文献");
        Localized.Name(search, "搜索文献"); Localized.Name(type, "类型筛选"); Localized.Name(field, "字段筛选");
        Localized.Count(count, 0);
        sidebar.RowDefinitions.Add(new() { Height = GridLength.Auto }); sidebar.RowDefinitions.Add(new());
        var sidebarHeading = new StackPanel { Spacing = 16 };
        var libraryHeader = new Grid { Margin = new Thickness(4, 0, 4, 8) };
        libraryHeader.ColumnDefinitions.Add(new()); libraryHeader.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        var libraryTitle = Views.LocalizedText("文献库", 18); libraryTitle.VerticalAlignment = VerticalAlignment.Center;
        libraryHeader.Children.Add(libraryTitle);
        var addLibrary = Views.Button("folderAdd", "新增文献库", () => _ = AddLibrary());
        Grid.SetColumn(addLibrary, 1); libraryHeader.Children.Add(addLibrary); sidebarHeading.Children.Add(libraryHeader);
        sidebar.Children.Add(sidebarHeading);
        Grid.SetRow(libraries, 1); sidebar.Children.Add(libraries);
        librariesEmpty.Margin = new Thickness(8, 32, 8, 8); Grid.SetRow(librariesEmpty, 1); sidebar.Children.Add(librariesEmpty);
        Grid.SetRow(sidebar, 1); root.Children.Add(sidebar);
        var center = new Grid { Padding = new Thickness(16), RowSpacing = 12 };
        for (var i = 0; i < 3; i++) center.RowDefinitions.Add(new() { Height = GridLength.Auto }); center.RowDefinitions.Add(new());
        var titleRow = new Grid { ColumnSpacing = 8 }; titleRow.ColumnDefinitions.Add(new()); titleRow.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        titleRow.Children.Add(heading);
        var countRow = Views.Row(count, progress); countRow.VerticalAlignment = VerticalAlignment.Center;
        Grid.SetColumn(countRow, 1); titleRow.Children.Add(countRow);
        Grid.SetRow(titleRow, 0); center.Children.Add(titleRow);
        foreach (var (label, value) in Reference.Types) type.Items.Add(LocalizedCombo(label, value));
        foreach (var (label, value) in new[] { ("全部字段", "all"), ("作者", "author"), ("标题", "title"), ("期刊", "journal"), ("年份", "year") }) field.Items.Add(LocalizedCombo(label, value));
        type.SelectedIndex = field.SelectedIndex = 0;
        var searchRow = new Grid { ColumnSpacing = 8 };
        searchRow.ColumnDefinitions.Add(new() { Width = GridLength.Auto }); searchRow.ColumnDefinitions.Add(new());
        searchRow.Children.Add(new SvgIcon("search", 15) { VerticalAlignment = VerticalAlignment.Center });
        Grid.SetColumn(search, 1); searchRow.Children.Add(search); Grid.SetRow(searchRow, 1); center.Children.Add(searchRow);
        var filters = Views.Row(type, field); Grid.SetRow(filters, 2); center.Children.Add(filters);
        Grid.SetRow(references, 3); center.Children.Add(references);
        Microsoft.UI.Xaml.Automation.AutomationProperties.SetName(brand, "BibCiTeX");
        empty.HorizontalAlignment = HorizontalAlignment.Center; empty.Opacity = .65;
        emptyState.Children.Add(brand); emptyState.Children.Add(empty);
        Grid.SetRow(emptyState, 3); center.Children.Add(emptyState);
        Grid.SetRow(center, 1); Grid.SetColumn(center, 2); root.Children.Add(center);
        inspector.RowDefinitions.Add(new() { Height = GridLength.Auto }); inspector.RowDefinitions.Add(new() { Height = GridLength.Auto }); inspector.RowDefinitions.Add(new());
        var detailHeading = Views.LocalizedText("文献详情", 16); detailHeading.Margin = new Thickness(16); inspector.Children.Add(detailHeading);
        var rule = Views.Divider(); Grid.SetRow(rule, 1); inspector.Children.Add(rule);
        var scroll = new ScrollViewer { Content = detail, HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled };
        Grid.SetRow(scroll, 2); inspector.Children.Add(scroll);
        detailEmpty.Margin = new Thickness(16); detailEmpty.Opacity = .65; detailEmpty.VerticalAlignment = VerticalAlignment.Center; detailEmpty.HorizontalAlignment = HorizontalAlignment.Center;
        Grid.SetRow(detailEmpty, 2); inspector.Children.Add(detailEmpty);
        Grid.SetColumn(inspector, 4); Grid.SetRow(inspector, 1); root.Children.Add(inspector);
        leftDivider = PaneDivider(true); rightDivider = PaneDivider(false);
        Grid.SetColumn(leftDivider, 1); Grid.SetColumn(rightDivider, 3);
        Grid.SetRow(leftDivider, 1); Grid.SetRow(rightDivider, 1); root.Children.Add(leftDivider); root.Children.Add(rightDivider);
        WindowInterop.LocalizeSystemMenu(this);
        Content = root; ShowDetail(null);
        libraries.SelectionChanged += (_, _) =>
        {
            if (reloading) return;
            current = (libraries.SelectedItem as ListViewItem)?.Tag as Library;
            if (current is { } selected) { Localized.BindValue(heading, TextBlock.TextProperty, () => selected.Name); NativeSettings.Values["mainLibrary"] = selected.Name; }
            else Localized.Bind(heading, TextBlock.TextProperty, "文献工作台");
            references.Items.Clear();
            Localized.Count(count, 0);
            emptyState.Visibility = Visibility.Visible;
            ShowDetail(null);
            _ = Search();
        };
        references.SelectionChanged += (_, _) => ShowDetail((references.SelectedItem as ListViewItem)?.Tag as Reference);
        search.TextCompositionStarted += (_, _) => searchComposing = true;
        search.TextCompositionEnded += (_, _) => { searchComposing = false; _ = Search(); };
        search.PreviewKeyDown += SearchKeyDown;
        search.TextChanged += (_, _) => { if (!searchComposing) _ = Search(true); }; type.SelectionChanged += (_, _) => _ = Search(); field.SelectionChanged += (_, _) => _ = Search();
        root.Loaded += async (_, _) => { SetMinimumSize(); root.XamlRoot.Changed += (_, _) => SetMinimumSize(); if (!initialized) { initialized = true; await ReloadLibraries(); } };
        AppWindow.Closing += (_, args) => { if (!quitting && tray is not null) { args.Cancel = true; AppWindow.Hide(); } };
        Closed += (_, _) => { aboutWindow?.Close(); shortcut?.Dispose(); tray?.Dispose(); };
        root.SizeChanged += (_, _) => LayoutPanes();
    }

    private MenuBar BuildMenu()
    {
        var menu = new MenuBar();
        MenuBarItem Group(string title)
        {
            var group = new MenuBarItem(); Localized.Bind(group, MenuBarItem.TitleProperty, title); menu.Items.Add(group); return group;
        }
        MenuFlyoutItem Add(MenuBarItem parent, string label, Action action, VirtualKey? key = null, VirtualKeyModifiers modifiers = VirtualKeyModifiers.Control)
        {
            var item = new MenuFlyoutItem(); Localized.Bind(item, MenuFlyoutItem.TextProperty, label); item.Click += (_, _) => action(); parent.Items.Add(item);
            if (key is { } value)
            {
                var accelerator = new KeyboardAccelerator { Key = value, Modifiers = modifiers };
                accelerator.Invoked += (_, args) => { action(); args.Handled = true; }; item.KeyboardAccelerators.Add(accelerator);
            }
            return item;
        }
        var app = Group("BibCiTeX");
        Add(app, "关于 BibCiTeX", () => _ = ShowAbout());
        var update = Add(app, "检查更新", () => _ = CheckUpdates()); update.IsEnabled = Updater.CanCheck; Updater.AddSettings(app);
        app.Items.Add(new MenuFlyoutSeparator()); Add(app, "退出 BibCiTeX", () => { quitting = true; Close(); });
        var file = Group("menu.file"); Add(file, "新增文献库", () => _ = AddLibrary(), VirtualKey.O);
        var reference = Group("文献");
        Add(reference, "快捷助手", () => _ = App.Helper!.Toggle(), VirtualKey.K, VirtualKeyModifiers.Control | VirtualKeyModifiers.Shift);
        var copy = Add(reference, "复制引用键", () => { if ((references.SelectedItem as ListViewItem)?.Tag is Reference selected) _ = Copy(selected.Key); }, VirtualKey.C, VirtualKeyModifiers.Control | VirtualKeyModifiers.Shift);
        copy.IsEnabled = false; references.SelectionChanged += (_, _) => copy.IsEnabled = references.SelectedItem is not null;
        Add(reference, "刷新", () => _ = ReloadLibraries(), VirtualKey.R);
        var view = Group("menu.view");
        Add(view, "文献库", () => { showSidebar = !showSidebar; LayoutPanes(); });
        Add(view, "文献详情", () => { showInspector = !showInspector; LayoutPanes(); });
        var languages = new MenuFlyoutSubItem(); Localized.Bind(languages, MenuFlyoutSubItem.TextProperty, "语言");
        foreach (var code in L10n.Languages)
        {
            var option = new RadioMenuFlyoutItem { GroupName = "Language", IsChecked = code == L10n.Selection };
            if (code == "system") Localized.Bind(option, MenuFlyoutItem.TextProperty, "跟随系统"); else option.Text = code == "zh-Hans" ? "简体中文" : "English";
            option.Click += (_, _) => App.SetLanguage(code); languages.Items.Add(option);
        }
        view.Items.Add(languages);
        var themes = new MenuFlyoutSubItem(); Localized.Bind(themes, MenuFlyoutSubItem.TextProperty, "主题");
        foreach (var (label, value) in new[] { ("跟随系统", ElementTheme.Default), ("浅色", ElementTheme.Light), ("深色", ElementTheme.Dark) })
        {
            var option = new RadioMenuFlyoutItem { GroupName = "Theme", IsChecked = App.Theme == value };
            Localized.Bind(option, MenuFlyoutItem.TextProperty, label); option.Click += (_, _) => { App.SetTheme(value); if (aboutWindow?.Content is FrameworkElement content) content.RequestedTheme = value; }; themes.Items.Add(option);
        }
        view.Items.Add(themes); return menu;
    }
    private static ToggleButton PaneToggle(string icon, string label, Action action)
    {
        var button = new ToggleButton { Content = new SvgIcon(icon), Padding = new Thickness(8), MinWidth = 32, MinHeight = 32, IsChecked = true };
        Localized.Name(button, label); Localized.Tooltip(button, label); button.Click += (_, _) => action(); return button;
    }
    private PaneThumb PaneDivider(bool left)
    {
        var thumb = new PaneThumb { Width = 5, IsTabStop = true, UseSystemFocusVisuals = true, HorizontalAlignment = HorizontalAlignment.Stretch, VerticalAlignment = VerticalAlignment.Stretch, Background = new SolidColorBrush(Microsoft.UI.Colors.Transparent) };
        Localized.Name(thumb, left ? "文献库" : "文献详情");
        void Move(double delta)
        {
            var available = root.ActualWidth - 330 - 10;
            if (left) sidebarWidth = Math.Clamp(root.ColumnDefinitions[0].ActualWidth + delta, 180, Math.Max(180, Math.Min(300, available - root.ColumnDefinitions[4].ActualWidth)));
            else inspectorWidth = Math.Clamp(root.ColumnDefinitions[4].ActualWidth - delta, 280, Math.Max(280, Math.Min(520, available - root.ColumnDefinitions[0].ActualWidth)));
            LayoutPanes();
        }
        thumb.Dragged += Move;
        thumb.KeyDown += (_, args) => { if (args.Key is VirtualKey.Left or VirtualKey.Right) { Move(args.Key == VirtualKey.Left ? -10 : 10); args.Handled = true; } };
        return thumb;
    }
    private void LayoutPanes()
    {
        if (leftDivider is null) return;
        var widths = WorkbenchLayout.Columns(root.ActualWidth, showSidebar, showInspector, sidebarWidth, inspectorWidth);
        root.ColumnDefinitions[0].Width = new GridLength(widths.Left);
        root.ColumnDefinitions[1].Width = new GridLength(showSidebar ? 5 : 0);
        root.ColumnDefinitions[3].Width = new GridLength(showInspector ? 5 : 0);
        root.ColumnDefinitions[4].Width = new GridLength(widths.Right);
        sidebar.Visibility = leftDivider.Visibility = showSidebar ? Visibility.Visible : Visibility.Collapsed;
        inspector.Visibility = rightDivider.Visibility = showInspector ? Visibility.Visible : Visibility.Collapsed;
        sidebarToggle.IsChecked = showSidebar; inspectorToggle.IsChecked = showInspector;
        ((SvgIcon)sidebarToggle.Content).Icon = showSidebar ? "panelLeftClose" : "panelLeftOpen";
        ((SvgIcon)inspectorToggle.Content).Icon = showInspector ? "panelRightClose" : "panelRightOpen";
    }

    private void SearchKeyDown(object sender, Microsoft.UI.Xaml.Input.KeyRoutedEventArgs args)
    {
        if (searchComposing || args.Key is not (VirtualKey.Up or VirtualKey.Down)) return;
        foreach (var key in new[] { VirtualKey.Control, VirtualKey.Menu, VirtualKey.Shift, VirtualKey.LeftWindows, VirtualKey.RightWindows })
            if ((Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(key) & Windows.UI.Core.CoreVirtualKeyStates.Down) != 0) return;
        args.Handled = true;
        if (loading || resultsVersion != searchVersion || references.Items.Count == 0) return;
        var direction = args.Key == VirtualKey.Down ? 1 : -1;
        references.SelectedIndex = references.SelectedIndex < 0
            ? (direction > 0 ? 0 : references.Items.Count - 1)
            : Math.Clamp(references.SelectedIndex + direction, 0, references.Items.Count - 1);
        references.ScrollIntoView(references.SelectedItem);
    }

    private void SetMinimumSize()
    {
        if (AppWindow.Presenter is Microsoft.UI.Windowing.OverlappedPresenter presenter)
        {
            var scale = WindowInterop.GetDpiForWindow(WinRT.Interop.WindowNative.GetWindowHandle(this)) / 96.0;
            presenter.PreferredMinimumWidth = (int)(816 * scale); presenter.PreferredMinimumHeight = (int)(520 * scale);
        }
    }

    internal void InstallShortcut()
    {
        try { shortcut = new GlobalShortcut(this, () => _ = App.Helper!.Toggle()); }
        catch (Exception error) { _ = Report(error); }
        try { tray = new TrayIcon(this, Activate, () => App.Tray!.Toggle(), () => _ = App.Helper!.Toggle(), () => { Activate(); _ = CheckUpdates(); }, () => { quitting = true; Close(); }); }
        catch (Exception error) { _ = Report(error); }
    }
    private async Task Report(Exception error)
    {
        await dialogs.WaitAsync(); try { await Views.Error(root, error); } finally { dialogs.Release(); }
    }
    // ProgressRing only spins while IsActive; keep visibility and activity in step.
    private void SetBusy(bool busy) { loading = busy; progress.IsActive = busy; progress.Visibility = busy ? Visibility.Visible : Visibility.Collapsed; emptyState.Visibility = !busy && references.Items.Count == 0 ? Visibility.Visible : Visibility.Collapsed; }
    private async Task ReloadLibraries(string? select = null)
    {
        var request = ++reloadVersion;
        try
        {
            var rows = await RustCore.Libraries();
            if (request != reloadVersion) return;
            // Resolve selection after the await: a user may have switched libraries meanwhile.
            var previousPath = current?.Path;
            var name = select ?? current?.Name ?? NativeSettings.Values["mainLibrary"] as string;
            reloading = true;
            libraries.Items.Clear();
            foreach (var library in rows)
            {
                var item = Views.LibraryItem(library, sidebar: true);
                var more = Views.Button("more", "文献库操作", () => { });
                Localized.Bind(more, Microsoft.UI.Xaml.Automation.AutomationProperties.NameProperty, "{0}的操作", library.Name);
                more.Opacity = 0;
                var row = new Grid { ColumnSpacing = 4 };
                row.ColumnDefinitions.Add(new()); row.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
                var content = (UIElement)item.Content; item.Content = null;
                row.Children.Add(content); Grid.SetColumn(more, 1); row.Children.Add(more); item.Content = row;
                more.VerticalAlignment = VerticalAlignment.Center;
                void Rename() => BeginLibraryRename(library, row, content, more);
                var menu = LibraryMenu(library, Rename); more.Flyout = menu;
                bool hovered = false, focused = false, opened = false;
                void UpdateMore() => more.Opacity = hovered || focused || opened ? 1 : 0;
                item.PointerEntered += (_, _) => { hovered = true; UpdateMore(); };
                item.PointerExited += (_, _) => { hovered = false; UpdateMore(); };
                more.GotFocus += (_, _) => { focused = true; UpdateMore(); };
                more.LostFocus += (_, _) => { focused = false; UpdateMore(); };
                menu.Opened += (_, _) => { opened = true; UpdateMore(); };
                menu.Closed += (_, _) => { opened = false; UpdateMore(); };
                item.ContextFlyout = LibraryMenu(library, Rename); libraries.Items.Add(item);
                if (library.Name == name) libraries.SelectedItem = item;
            }
            librariesEmpty.Visibility = rows.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
            brand.Visibility = rows.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
            if (libraries.SelectedItem is null && libraries.Items.Count > 0) libraries.SelectedIndex = 0;
            reloading = false;
            current = (libraries.SelectedItem as ListViewItem)?.Tag as Library;
            if (current is { } selected)
            {
                NativeSettings.Values["mainLibrary"] = selected.Name;
                Localized.BindValue(heading, TextBlock.TextProperty, () => selected.Name);
            }
            else Localized.Bind(heading, TextBlock.TextProperty, "文献工作台");
            if (current?.Path != previousPath) { references.Items.Clear(); ShowDetail(null); }
            await Search();
        }
        catch (Exception error) { await Report(error); }
        finally { if (request == reloadVersion) reloading = false; }
    }
    private async Task Search(bool debounce = false)
    {
        var version = ++searchVersion; var library = current;
        if (library is null) { references.Items.Clear(); ShowDetail(null); Localized.Count(count, 0); emptyState.Visibility = Visibility.Visible; SetBusy(false); return; }
        var query = search.Text; var searchField = (field.SelectedItem as ComboBoxItem)?.Tag as string ?? "all"; var searchType = (type.SelectedItem as ComboBoxItem)?.Tag as string ?? "all";
        SetBusy(true);
        try
        {
            if (debounce) await Task.Delay(100);
            if (version != searchVersion) return;
            var rows = await RustCore.Search(library.Path, query, searchField, searchType);
            if (version != searchVersion) return;
            var selected = ((references.SelectedItem as ListViewItem)?.Tag as Reference)?.Id;
            references.Items.Clear();
            foreach (var reference in rows) { var item = Views.ReferenceItem(reference, citeKeyCopies: true, copy: Copy); references.Items.Add(item); if (reference.Id == selected) references.SelectedItem = item; }
            resultsVersion = version;
            if (references.SelectedItem is null && rows.Count > 0) references.SelectedIndex = 0;
            Localized.Count(count, rows.Count); emptyState.Visibility = rows.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
        }
        catch (Exception error) { if (version == searchVersion) { references.Items.Clear(); ShowDetail(null); Localized.Count(count, 0); await Report(error); } }
        finally { if (version == searchVersion) SetBusy(false); }
    }
    private void BeginLibraryRename(Library library, Grid row, UIElement content, Button more)
    {
        if (!row.Children.Contains(content)) return;
        var libraryRow = (Grid)content;
        var labels = (StackPanel)libraryRow.Children[1];
        var originalName = labels.Children[0];
        var name = new TextBox { Text = library.Name, VerticalAlignment = VerticalAlignment.Center };
        Views.LocalizeTextBox(name);
        Localized.Name(name, "文献库名称");
        var error = Views.Text("", 11); Views.ThemeForeground(error, "SystemFillColorCriticalBrush");
        error.Visibility = Visibility.Collapsed;
        var editor = new StackPanel { Spacing = 4, VerticalAlignment = VerticalAlignment.Center };
        editor.Children.Add(name); editor.Children.Add(error);
        bool editing = true, saving = false;
        void Cancel()
        {
            if (!editing || saving) return;
            editing = false; labels.Children.Remove(editor); labels.Children.Insert(0, originalName); more.IsEnabled = true;
        }
        async Task Save()
        {
            if (!editing || saving) return;
            var value = name.Text.Trim();
            if (value.Length == 0) { Localized.Bind(error, TextBlock.TextProperty, "文献库名称不能为空"); error.Visibility = Visibility.Visible; name.Focus(FocusState.Programmatic); return; }
            if (value == library.Name) { Cancel(); return; }
            saving = true; name.IsReadOnly = true;
            try
            {
                var updated = await RustCore.UpdateLibrary(library.Name, value, null, null);
                if (current?.Name == library.Name) current = updated;
                editing = false;
                await ReloadLibraries();
            }
            catch (Exception ex) { Localized.Error(error, TextBlock.TextProperty, ex); error.Visibility = Visibility.Visible; name.Focus(FocusState.Programmatic); }
            finally { saving = false; name.IsReadOnly = false; }
        }
        name.KeyDown += async (_, args) =>
        {
            if (args.Key == VirtualKey.Escape) { args.Handled = true; Cancel(); }
            else if (args.Key == VirtualKey.Enter) { args.Handled = true; await Save(); }
        };
        name.LostFocus += async (_, _) => await Save();
        name.Loaded += (_, _) => { name.Focus(FocusState.Programmatic); name.SelectAll(); };
        labels.Children.Remove(originalName); labels.Children.Insert(0, editor); more.IsEnabled = false;
    }
    private MenuFlyout LibraryMenu(Library library, Action rename)
    {
        var menu = new MenuFlyout();
        MenuFlyoutItem Add(string title, string icon, Func<Task> action)
        {
            var item = new MenuFlyoutItem { Text = title, Icon = new ImageIcon { Source = new Microsoft.UI.Xaml.Media.Imaging.SvgImageSource(new Uri($"ms-appx:///Assets/Icons/{(root.ActualTheme == ElementTheme.Dark ? "Dark/" : "")}{icon}.svg")) } };
            Localized.Bind(item, MenuFlyoutItem.TextProperty, title);
            item.Click += async (_, _) => await action(); menu.Items.Add(item); return item;
        }
        Add("编辑", "settings", () => AddLibrary(library));
        Add("重命名", "rename", () => { rename(); return Task.CompletedTask; });
        Add(library.Pinned ? "取消置顶" : "置顶", "pin", async () =>
        {
            try { await RustCore.SetLibraryPinned(library.Name, !library.Pinned); await ReloadLibraries(); }
            catch (Exception error) { await Report(error); }
        });
        menu.Items.Add(new MenuFlyoutSeparator());
        Add("打开文件", "folderOpen", () => OpenFile(library.Path));
        Views.ThemeForeground(Add("移除", "x", () => RemoveLibrary(library)), "SystemFillColorCriticalBrush");
        return menu;
    }
    private async Task AddLibrary(Library? library = null)
    {
        await dialogs.WaitAsync();
        string? selectAfterSave = null;
        try
        {
            var name = new TextBox { Header = L10n.Text("文献库名称"), PlaceholderText = L10n.Text("为文献库起一个名字") };
            var path = new TextBox { Header = L10n.Text("文件路径"), PlaceholderText = L10n.Text("尚未选择文件"), IsReadOnly = true };
            var description = new TextBox { Header = L10n.Text("描述"), PlaceholderText = L10n.Text("简单描述一下这个文献库..."), AcceptsReturn = true, TextWrapping = TextWrapping.Wrap, MinHeight = 80, MaxHeight = 130 };
            foreach (var box in new[] { name, path, description }) Views.LocalizeTextBox(box);
            Localized.Bind(name, TextBox.HeaderProperty, "文献库名称"); Localized.Bind(name, TextBox.PlaceholderTextProperty, "为文献库起一个名字");
            Localized.Bind(path, TextBox.HeaderProperty, "文件路径"); Localized.Bind(path, TextBox.PlaceholderTextProperty, "尚未选择文件");
            Localized.Bind(description, TextBox.HeaderProperty, "描述"); Localized.Bind(description, TextBox.PlaceholderTextProperty, "简单描述一下这个文献库...");
            name.Text = library?.Name ?? ""; path.Text = library?.Path ?? ""; description.Text = library?.Description ?? "";
            var error = Views.Text(""); error.Visibility = Visibility.Collapsed; error.IsTextSelectionEnabled = true; Views.SelectableText(error); Views.ThemeForeground(error, "SystemFillColorCriticalBrush");
            var choose = new Button { Content = L10n.Text("选择文件") };
            Localized.Bind(choose, ContentControl.ContentProperty, "选择文件");
            choose.Click += async (_, _) =>
            {
                try
                {
                    var picker = new FileOpenPicker(); picker.FileTypeFilter.Add(".bib");
                    WinRT.Interop.InitializeWithWindow.Initialize(picker, WinRT.Interop.WindowNative.GetWindowHandle(this));
                    if (await picker.PickSingleFileAsync() is { } file) { path.Text = file.Path; if (name.Text.Length == 0) name.Text = System.IO.Path.GetFileNameWithoutExtension(file.Name); }
                }
                catch (Exception ex) { Localized.Error(error, TextBlock.TextProperty, ex); error.Visibility = Visibility.Visible; }
            };
            var form = new StackPanel { Spacing = 20, Width = 464, Margin = new Thickness(0, 8, 0, 8) };
            var title = Views.LocalizedText(library is null ? "新增文献库" : "编辑文献库", 22);
            var titleRow = Views.Row(new Border { Child = new SvgIcon("folderAdd", 24), Padding = new Thickness(12), CornerRadius = new CornerRadius(12), Background = Views.Brush("SubtleFillColorSecondaryBrush") }, title);
            title.VerticalAlignment = VerticalAlignment.Center; form.Children.Add(titleRow);
            if (library is null) { var subtitle = Views.LocalizedText("添加一个 .bib 文件到你的工作空间"); subtitle.Opacity = .65; form.Children.Add(subtitle); }
            form.Children.Add(name);
            {
                var fileSection = new Grid { ColumnSpacing = 12 };
                fileSection.ColumnDefinitions.Add(new() { Width = GridLength.Auto }); fileSection.ColumnDefinitions.Add(new()); fileSection.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
                fileSection.Children.Add(new SvgIcon("fileText", 24) { VerticalAlignment = VerticalAlignment.Center });
                var pathLabel = new MiddleEllipsisText(14, 2) { VerticalAlignment = VerticalAlignment.Center };
                void UpdatePath() { Localized.BindValue(pathLabel, MiddleEllipsisText.ValueProperty, () => path.Text.Length == 0 ? L10n.Text("尚未选择文件") : PathDisplay.Format(path.Text)); ToolTipService.SetToolTip(pathLabel, PathDisplay.Format(path.Text)); }
                path.TextChanged += (_, _) => UpdatePath(); UpdatePath();
                Grid.SetColumn(pathLabel, 1); fileSection.Children.Add(pathLabel); Grid.SetColumn(choose, 2); fileSection.Children.Add(choose);
                form.Children.Add(Views.LocalizedText("文件路径"));
                form.Children.Add(new Border { Child = fileSection, Padding = new Thickness(16), CornerRadius = new CornerRadius(8), BorderThickness = new Thickness(1), BorderBrush = (Brush)Application.Current.Resources["ControlStrokeColorDefaultBrush"] });
                form.Children.Add(description);
            }
            form.Children.Add(error);
            var saveProgress = new ProgressRing { Width = 20, Height = 20, IsActive = false, Visibility = Visibility.Collapsed, HorizontalAlignment = HorizontalAlignment.Left }; form.Children.Add(saveProgress);
            var dialog = Localized.Dialog(new ContentDialog { XamlRoot = root.XamlRoot, Content = new ScrollViewer { Content = form, VerticalScrollBarVisibility = ScrollBarVisibility.Auto }, IsPrimaryButtonEnabled = false, DefaultButton = ContentDialogButton.Primary }, library is null ? "新增文献库" : "编辑文献库", "取消", "保存");
            // The composed heading is part of the form, so do not duplicate the standard dialog title.
            Localized.BindValue(dialog, ContentDialog.TitleProperty, () => "");
            Localized.Name(dialog, library is null ? "新增文献库" : "编辑文献库");
            var saving = false;
            dialog.Closing += (_, args) => { if (saving) args.Cancel = true; };
            void Validate() => dialog.IsPrimaryButtonEnabled = !saving && name.Text.Trim().Length > 0 && path.Text.Length > 0;
            name.TextChanged += (_, _) => Validate(); path.TextChanged += (_, _) => Validate(); Validate();
            dialog.PrimaryButtonClick += async (_, args) =>
            {
                if (libraries.Items.Cast<ListViewItem>().Any(item => item.Tag is Library other && other.Name == name.Text.Trim() && other.Name != library?.Name))
                {
                    args.Cancel = true; Localized.Bind(error, TextBlock.TextProperty, "该名称已存在，请换一个"); error.Visibility = Visibility.Visible; return;
                }
                var deferral = args.GetDeferral();
                saving = true; name.IsEnabled = description.IsEnabled = choose.IsEnabled = false; dialog.IsPrimaryButtonEnabled = false; dialog.IsSecondaryButtonEnabled = false;
                saveProgress.IsActive = true; saveProgress.Visibility = Visibility.Visible;
                try
                {
                    if (library is null) await RustCore.AddLibrary(name.Text.Trim(), path.Text, description.Text);
                    else
                    {
                        var updated = await RustCore.UpdateLibrary(library.Name, name.Text.Trim(), path.Text == library.Path ? null : path.Text, description.Text);
                        if (current?.Name == library.Name) current = updated;
                    }
                    if (library is null || current?.Name == library.Name || current?.Name == name.Text.Trim()) selectAfterSave = name.Text.Trim();
                }
                catch (Exception ex) { args.Cancel = true; Localized.Error(error, TextBlock.TextProperty, ex); error.Visibility = Visibility.Visible; }
                finally { saving = false; name.IsEnabled = description.IsEnabled = choose.IsEnabled = true; saveProgress.IsActive = false; saveProgress.Visibility = Visibility.Collapsed; Validate(); deferral.Complete(); }
            };
            await Views.ShowDialog(dialog);
        }
        finally { dialogs.Release(); }
        await ReloadLibraries(selectAfterSave);
    }
    private async Task RemoveLibrary(Library library)
    {
        try { await RustCore.RemoveLibrary(library.Name); await ReloadLibraries(); }
        catch (Exception error) { await Report(error); }
    }
    private static ComboBoxItem LocalizedCombo(string key, string value)
    {
        var item = new ComboBoxItem { Tag = value };
        Localized.Bind(item, ContentControl.ContentProperty, key);
        return item;
    }
    private void ShowDetail(Reference? reference)
    {
        detail.Children.Clear(); detailEmpty.Visibility = reference is null ? Visibility.Visible : Visibility.Collapsed;
        if (reference is null) return;
        detail.Children.Add(new ChunkText(reference.Chunks("title"), 18, reference.Title, fallbackKey: "暂无标题"));
        var libraryPath = current?.Path;
        var actions = Views.Row(Views.CopyButton("copy", "复制引用键", () => Copy(reference.Key)), Views.CopyButton("clipboard", "复制 BibTeX", () => Copy(reference.Text("source"))));
        if (reference.Text("file") is { Length: > 0 } file) actions.Children.Add(Views.Button("folderOpen", "打开文件", () => { if (current?.Path == libraryPath) _ = OpenFile(file, libraryPath, attachment: true); }));
        if (reference.Text("url") is { Length: > 0 } url) actions.Children.Add(Views.Button("externalLink", "打开 URL", () => _ = OpenUrl(url)));
        if (reference.Text("doi") is { Length: > 0 } doi) actions.Children.Add(Views.Button("link", "打开 DOI", () => _ = OpenUrl(doi.StartsWith("http", StringComparison.OrdinalIgnoreCase) ? doi : "https://doi.org/" + doi)));
        detail.Children.Add(new ScrollViewer { Content = actions, HorizontalScrollBarVisibility = ScrollBarVisibility.Auto, VerticalScrollBarVisibility = ScrollBarVisibility.Disabled });
        foreach (var (key, label) in Metadata)
        {
            var value = key == "cite_key" ? reference.Key : key == "type" ? reference.TypeLabel : reference.Text(key); if (value.Length == 0) continue;
            if (key == "file") value = PathDisplay.Format(value);
            var section = new StackPanel { Spacing = 4 };
            var caption = Views.LocalizedText(label, 12); caption.Opacity = .6; section.Children.Add(caption);
            if (key is "title" or "note" or "abstract_" or "book_title" or "issue") section.Children.Add(new ChunkText(reference.Chunks(key), 14, value));
            else section.Children.Add(Views.SelectableText(new TextBlock { Text = value, TextWrapping = TextWrapping.Wrap, IsTextSelectionEnabled = true, FontFamily = key == "source" ? new FontFamily("Cascadia Mono") : new FontFamily("Segoe UI Variable") }));
            if (key == "type" && section.Children[1] is TextBlock typeText) Localized.BindValue(typeText, TextBlock.TextProperty, () => reference.TypeLabel);
            if (key == "source") detail.Children.Add(new Expander { Header = "BibTeX", Content = section.Children[1] is UIElement source ? Detach(section, source) : null, HorizontalAlignment = HorizontalAlignment.Stretch });
            else detail.Children.Add(section);
        }
    }
    private static UIElement Detach(Panel parent, UIElement child) { parent.Children.Remove(child); return child; }
    // Mirrors the macOS inspector: same labels, same order. Backslash-free keys are
    // Rust field names; the label column is the macOS wording and must not drift.
    private static readonly (string Key, string Label)[] Metadata = [
        ("cite_key", "引用键"), ("type", "类型"), ("author", "作者"), ("year", "年份"), ("month", "月份"), ("journal", "期刊"), ("full_journal", "期刊全称"),
        ("volume", "卷号"), ("number", "编号"), ("pages", "页码"), ("book_pages", "总页数"), ("publisher", "出版社"),
        ("edition", "版本"), ("series", "丛书"), ("editor", "metadata.editor"), ("school", "学校"), ("address", "地址"),
        ("organization", "组织"), ("institution", "机构"), ("doi", "DOI"), ("isbn", "ISBN"), ("mrclass", "MR 分类"),
        ("url", "URL"), ("file", "文件"), ("eprint", "Eprint"), ("archive_prefix", "Archive Prefix"),
        ("arxiv_primary_class", "arXiv 分类"), ("how_published", "发表方式"), ("abstract_", "摘要"),
        ("book_title", "书名"), ("issue", "期号"), ("note", "备注"), ("source", "BibTeX") ];
    private async Task<bool> Copy(string text)
    {
        try { await RustCore.Copy(text); return true; }
        catch (Exception error) { await Report(error); return false; }
    }
    private async Task OpenUrl(string value)
    {
        try { var uri = new Uri(value); if (uri.Scheme is not ("http" or "https")) throw new LocalizedException("仅支持 HTTP/HTTPS 链接"); if (!await Launcher.LaunchUriAsync(uri)) throw new IOException(value); }
        catch (Exception error) { await Report(error); }
    }
    private async Task OpenFile(string value, string? libraryPath = null, bool attachment = false)
    {
        try
        {
            var path = attachment ? AttachmentPath.Resolve(value, libraryPath) : System.IO.Path.GetFullPath(value);
            if (!await Launcher.LaunchFileAsync(await StorageFile.GetFileFromPathAsync(path))) throw new IOException(path);
        }
        catch (Exception error) { await Report(error); }
    }
    private static Image BrandImage(double size)
    {
        var image = new Image
        {
            Source = new Microsoft.UI.Xaml.Media.Imaging.BitmapImage(new Uri("ms-appx:///Assets/favicon.png")),
            Width = size, Height = size, Stretch = Stretch.Uniform, HorizontalAlignment = HorizontalAlignment.Left
        };
        Microsoft.UI.Xaml.Automation.AutomationProperties.SetName(image, "BibCiTeX");
        return image;
    }

    private Task ShowAbout()
    {
        if (aboutWindow is not null) { aboutWindow.Activate(); return Task.CompletedTask; }
        var window = new Window { Title = L10n.Text("关于 BibCiTeX"), SystemBackdrop = Microsoft.UI.Composition.SystemBackdrops.MicaController.IsSupported() ? new MicaBackdrop() : new DesktopAcrylicBackdrop() };
        var content = new StackPanel { Spacing = 12, Padding = new Thickness(28), HorizontalAlignment = HorizontalAlignment.Center, VerticalAlignment = VerticalAlignment.Center, RequestedTheme = App.Theme };
        var logo = BrandImage(96); logo.HorizontalAlignment = HorizontalAlignment.Center; content.Children.Add(logo);
        var name = Views.Text("BibCiTeX", 24); name.HorizontalAlignment = HorizontalAlignment.Center; content.Children.Add(name);
        var version = Updater.CurrentVersion;
        var versionText = Views.Text(version, 13); versionText.HorizontalAlignment = HorizontalAlignment.Center; versionText.IsTextSelectionEnabled = true;
        content.Children.Add(Views.SelectableText(versionText));
        var close = new Button { HorizontalAlignment = HorizontalAlignment.Center }; Localized.Bind(close, ContentControl.ContentProperty, "关闭"); close.Click += (_, _) => window.Close(); content.Children.Add(close);
        content.KeyDown += (_, args) => { if (args.Key == VirtualKey.Escape) { window.Close(); args.Handled = true; } };
        Localized.BindValue(content, FrameworkElement.LanguageProperty, () => L10n.Language);
        window.Content = content; aboutWindow = window;
        void Localize() { window.Title = L10n.Text("关于 BibCiTeX"); WindowInterop.LocalizeSystemMenu(window); }
        L10n.Changed += Localize; window.Closed += (_, _) => { L10n.Changed -= Localize; aboutWindow = null; };
        var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(window); var scale = WindowInterop.GetDpiForWindow(hwnd) / 96.0;
        window.AppWindow.ResizeClient(new Windows.Graphics.SizeInt32((int)(340 * scale), (int)(300 * scale)));
        if (window.AppWindow.Presenter is Microsoft.UI.Windowing.OverlappedPresenter presenter) { presenter.IsResizable = false; presenter.IsMaximizable = false; presenter.IsMinimizable = false; }
        window.AppWindow.Move(new Windows.Graphics.PointInt32(AppWindow.Position.X + (AppWindow.Size.Width - window.AppWindow.Size.Width) / 2, AppWindow.Position.Y + (AppWindow.Size.Height - window.AppWindow.Size.Height) / 2));
        WindowInterop.LocalizeSystemMenu(window); window.Activate(); return Task.CompletedTask;
    }

    private async Task CheckUpdates()
    {
        await dialogs.WaitAsync();
        try { await Updater.Check(root); }
        catch (Exception error) { await Views.Error(root, error); }
        finally { dialogs.Release(); }
    }
}
