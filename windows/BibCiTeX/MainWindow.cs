using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Windows.Storage;
using Windows.Storage.Pickers;
using Windows.System;

namespace BibCiTeX;

internal sealed class MainWindow : Window
{
    private readonly Grid root = new() { ColumnSpacing = 1 };
    private readonly ListView libraries = new() { SelectionMode = ListViewSelectionMode.Single };
    private readonly ListView references = new() { SelectionMode = ListViewSelectionMode.Single };
    private readonly StackPanel detail = new() { Padding = new Thickness(20), Spacing = 16 };
    private readonly TextBlock heading = Views.Text("文献工作台", 20);
    private readonly TextBlock count = Views.Text("0 条文献", 12);
    private readonly TextBlock empty = Views.Text("暂无可显示的文献");
    private readonly TextBlock librariesEmpty = Views.Text("暂无文献库");
    private readonly TextBox search = new() { PlaceholderText = "搜索文献", MinWidth = 160 };
    private readonly ComboBox field = new() { MinWidth = 110 };
    private readonly ComboBox type = new() { MinWidth = 110 };
    private readonly ProgressRing progress = new() { Width = 16, Height = 16, IsActive = false, Visibility = Visibility.Collapsed };
    private Library? current;
    private int searchVersion;
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
        root.ColumnDefinitions.Add(new() { Width = new GridLength(230) });
        root.ColumnDefinitions.Add(new() { Width = new GridLength(1, GridUnitType.Star) });
        root.ColumnDefinitions.Add(new() { Width = new GridLength(340) });
        root.RowDefinitions.Add(new() { Height = GridLength.Auto }); root.RowDefinitions.Add(new());
        var toolbar = Views.Row(Views.Button("add", "新增文献库", () => _ = AddLibrary()),
            Views.Button("refresh", "刷新", () => _ = ReloadLibraries()),
            Views.Button("search", "快捷助手", () => _ = App.Helper!.Toggle()),
            Views.Button("download", "检查更新", () => _ = CheckUpdates()),
            Views.ThemedButton(() => root.ActualTheme == ElementTheme.Dark ? "moon" : "sun", "切换主题", () => App.SetTheme(root.ActualTheme == ElementTheme.Dark ? ElementTheme.Light : ElementTheme.Dark)));
        toolbar.Padding = new Thickness(12, 8, 12, 8); Grid.SetColumnSpan(toolbar, 3); root.Children.Add(toolbar);
        var sidebar = new Grid { Padding = new Thickness(10, 12, 10, 8) };
        sidebar.RowDefinitions.Add(new() { Height = GridLength.Auto }); sidebar.RowDefinitions.Add(new());
        sidebar.Children.Add(Views.Text("文献库", 18));
        Grid.SetRow(libraries, 1); sidebar.Children.Add(libraries);
        librariesEmpty.Margin = new Thickness(8, 32, 8, 8); Grid.SetRow(librariesEmpty, 1); sidebar.Children.Add(librariesEmpty);
        Grid.SetRow(sidebar, 1); root.Children.Add(sidebar);
        var center = new Grid { Padding = new Thickness(16, 12, 16, 8), RowSpacing = 12 };
        center.RowDefinitions.Add(new() { Height = GridLength.Auto }); center.RowDefinitions.Add(new() { Height = GridLength.Auto }); center.RowDefinitions.Add(new());
        var titleRow = new Grid { ColumnSpacing = 8 }; titleRow.ColumnDefinitions.Add(new()); titleRow.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        titleRow.Children.Add(heading);
        var countRow = Views.Row(count, progress); countRow.VerticalAlignment = VerticalAlignment.Center;
        Grid.SetColumn(countRow, 1); titleRow.Children.Add(countRow);
        Grid.SetRow(titleRow, 0); center.Children.Add(titleRow);
        foreach (var (label, value) in Reference.Types) type.Items.Add(new ComboBoxItem { Content = label, Tag = value });
        foreach (var (label, value) in new[] { ("全部字段", "all"), ("作者", "author"), ("标题", "title"), ("期刊", "journal"), ("年份", "year") }) field.Items.Add(new ComboBoxItem { Content = label, Tag = value });
        type.SelectedIndex = field.SelectedIndex = 0;
        var filters = new Grid { ColumnSpacing = 8, RowSpacing = 8 };
        filters.ColumnDefinitions.Add(new()); filters.ColumnDefinitions.Add(new() { Width = GridLength.Auto }); filters.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        filters.Children.Add(search); Grid.SetColumn(field, 1); filters.Children.Add(field); Grid.SetColumn(type, 2); filters.Children.Add(type);
        Grid.SetRow(filters, 1); center.Children.Add(filters);
        Grid.SetRow(references, 2); center.Children.Add(references);
        empty.HorizontalAlignment = HorizontalAlignment.Center; empty.VerticalAlignment = VerticalAlignment.Center; empty.Opacity = .65; Grid.SetRow(empty, 2); center.Children.Add(empty);
        Grid.SetRow(center, 1); Grid.SetColumn(center, 1); root.Children.Add(center);
        var inspector = new ScrollViewer { Content = detail, HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled }; Grid.SetColumn(inspector, 2); Grid.SetRow(inspector, 1); root.Children.Add(inspector);
        Content = root; ShowDetail(null);
        libraries.SelectionChanged += (_, _) =>
        {
            current = (libraries.SelectedItem as ListViewItem)?.Tag as Library;
            heading.Text = current?.Name ?? "文献工作台";
            references.Items.Clear();
            count.Text = "0 条文献";
            empty.Visibility = Visibility.Visible;
            ShowDetail(null);
            _ = Search();
        };
        references.SelectionChanged += (_, _) => ShowDetail((references.SelectedItem as ListViewItem)?.Tag as Reference);
        search.TextChanged += (_, _) => _ = Search(true); type.SelectionChanged += (_, _) => _ = Search(); field.SelectionChanged += (_, _) => _ = Search();
        root.Loaded += async (_, _) => { SetMinimumSize(); root.XamlRoot.Changed += (_, _) => SetMinimumSize(); if (!initialized) { initialized = true; await ReloadLibraries(); } };
        AppWindow.Closing += (_, args) => { if (!quitting && tray is not null) { args.Cancel = true; AppWindow.Hide(); } };
        Closed += (_, _) => { shortcut?.Dispose(); tray?.Dispose(); };
        root.SizeChanged += (_, e) => { root.ColumnDefinitions[0].Width = new GridLength(e.NewSize.Width < 1050 ? 190 : 230); root.ColumnDefinitions[2].Width = new GridLength(e.NewSize.Width < 1050 ? 280 : 340); };
    }

    private void SetMinimumSize()
    {
        if (AppWindow.Presenter is Microsoft.UI.Windowing.OverlappedPresenter presenter)
        {
            var scale = WindowInterop.GetDpiForWindow(WinRT.Interop.WindowNative.GetWindowHandle(this)) / 96.0;
            presenter.PreferredMinimumWidth = (int)(940 * scale); presenter.PreferredMinimumHeight = (int)(560 * scale);
        }
    }

    internal void InstallShortcut()
    {
        try { shortcut = new GlobalShortcut(this, () => _ = App.Helper!.Toggle()); }
        catch (Exception error) { _ = Report(error); }
        try { tray = new TrayIcon(this, Activate, () => _ = App.Helper!.Toggle(), () => { Activate(); _ = CheckUpdates(); }, () => { quitting = true; Close(); }); }
        catch (Exception error) { _ = Report(error); }
    }
    private async Task Report(Exception error)
    {
        await dialogs.WaitAsync(); try { await Views.Error(root, error); } finally { dialogs.Release(); }
    }
    // ProgressRing only spins while IsActive; keep visibility and activity in step.
    private void SetBusy(bool busy) { progress.IsActive = busy; progress.Visibility = busy ? Visibility.Visible : Visibility.Collapsed; }
    private async Task ReloadLibraries()
    {
        try
        {
            var name = current?.Name; var rows = await RustCore.Libraries();
            libraries.Items.Clear();
            foreach (var library in rows)
            {
                var item = Views.LibraryItem(library);
                var menu = new MenuFlyout();
                var open = new MenuFlyoutItem { Text = "打开文件" }; open.Click += async (_, _) => await OpenFile(library.Path);
                var remove = new MenuFlyoutItem { Text = "删除" }; remove.Click += async (_, _) => await RemoveLibrary(library);
                menu.Items.Add(open); menu.Items.Add(remove); item.ContextFlyout = menu; libraries.Items.Add(item);
                if (library.Name == name) libraries.SelectedItem = item;
            }
            librariesEmpty.Visibility = rows.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
            if (libraries.SelectedItem is null && libraries.Items.Count > 0) libraries.SelectedIndex = 0;
            if (libraries.Items.Count == 0) { current = null; await Search(); }
        }
        catch (Exception error) { await Report(error); }
    }
    private async Task Search(bool debounce = false)
    {
        var version = ++searchVersion; var library = current;
        if (library is null) { references.Items.Clear(); count.Text = "0 条文献"; empty.Visibility = Visibility.Visible; SetBusy(false); return; }
        var query = search.Text; var searchField = (field.SelectedItem as ComboBoxItem)?.Tag as string ?? "all"; var searchType = (type.SelectedItem as ComboBoxItem)?.Tag as string ?? "all";
        try
        {
            if (debounce) await Task.Delay(100);
            if (version != searchVersion) return;
            SetBusy(true);
            var rows = await RustCore.Search(library.Path, query, searchField, searchType);
            if (version != searchVersion) return;
            var selected = ((references.SelectedItem as ListViewItem)?.Tag as Reference)?.Id;
            references.Items.Clear();
            foreach (var reference in rows) { var item = Views.ReferenceItem(reference); references.Items.Add(item); if (reference.Id == selected) references.SelectedItem = item; }
            count.Text = $"{rows.Count} 条文献"; empty.Visibility = rows.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
        }
        catch (Exception error) { if (version == searchVersion) await Report(error); }
        finally { if (version == searchVersion) SetBusy(false); }
    }
    private async Task AddLibrary()
    {
        await dialogs.WaitAsync();
        try
        {
            var name = new TextBox { Header = "文献库名称", PlaceholderText = "为文献库起一个名字" };
            var path = new TextBox { Header = "文件路径", PlaceholderText = "尚未选择文件", IsReadOnly = true };
            var description = new TextBox { Header = "描述", PlaceholderText = "简单描述一下这个文献库...", AcceptsReturn = true, MinHeight = 80 };
            var error = Views.Text(""); error.Visibility = Visibility.Collapsed;
            var choose = new Button { Content = "选择文件" };
            choose.Click += async (_, _) =>
            {
                try
                {
                    var picker = new FileOpenPicker(); picker.FileTypeFilter.Add(".bib");
                    WinRT.Interop.InitializeWithWindow.Initialize(picker, WinRT.Interop.WindowNative.GetWindowHandle(this));
                    if (await picker.PickSingleFileAsync() is { } file) { path.Text = file.Path; if (name.Text.Length == 0) name.Text = System.IO.Path.GetFileNameWithoutExtension(file.Name); }
                }
                catch (Exception ex) { error.Text = ex.Message; error.Visibility = Visibility.Visible; }
            };
            var form = new StackPanel { Spacing = 12, MinWidth = 400 };
            form.Children.Add(Views.Text("添加一个 .bib 文件到你的工作空间")); form.Children.Add(name); form.Children.Add(path); form.Children.Add(choose); form.Children.Add(description); form.Children.Add(error);
            var dialog = new ContentDialog { XamlRoot = root.XamlRoot, Title = "新增文献库", Content = form, PrimaryButtonText = "保存", CloseButtonText = "取消", IsPrimaryButtonEnabled = false };
            void Validate() => dialog.IsPrimaryButtonEnabled = name.Text.Trim().Length > 0 && path.Text.Length > 0;
            name.TextChanged += (_, _) => Validate(); path.TextChanged += (_, _) => Validate();
            dialog.PrimaryButtonClick += async (_, args) =>
            {
                var deferral = args.GetDeferral();
                try { await RustCore.AddLibrary(name.Text.Trim(), path.Text, description.Text); }
                catch (Exception ex) { args.Cancel = true; error.Text = ex.Message; error.Visibility = Visibility.Visible; }
                finally { deferral.Complete(); }
            };
            await dialog.ShowAsync();
        }
        finally { dialogs.Release(); }
        await ReloadLibraries();
    }
    private async Task RemoveLibrary(Library library)
    {
        await dialogs.WaitAsync();
        try
        {
            var dialog = new ContentDialog { XamlRoot = root.XamlRoot, Title = "删除", Content = library.Name, PrimaryButtonText = "删除", CloseButtonText = "取消" };
            if (await dialog.ShowAsync() == ContentDialogResult.Primary) await RustCore.RemoveLibrary(library.Name);
        }
        catch (Exception error) { await Views.Error(root, error); }
        finally { dialogs.Release(); }
        await ReloadLibraries();
    }
    private void ShowDetail(Reference? reference)
    {
        detail.Children.Clear(); detail.Children.Add(Views.Text("文献详情", 18));
        if (reference is null) { detail.Children.Add(Views.Text("选择一条文献查看字段和操作")); return; }
        detail.Children.Add(new ChunkText(reference.Chunks("title"), 20, reference.Title));
        detail.Children.Add(new TextBlock { Text = reference.TypeLabel, FontSize = 12, Opacity = .6 });
        detail.Children.Add(new TextBlock { Text = reference.Key, FontSize = 12, FontFamily = new FontFamily("Cascadia Mono"), IsTextSelectionEnabled = true });
        var actions = Views.Row(Views.Button("copy", "复制引用键", () => _ = Copy(reference.Key)));
        if (reference.Text("doi") is { Length: > 0 } doi) actions.Children.Add(Views.Button("externalLink", "打开 DOI", () => _ = OpenUrl(doi.StartsWith("https://", StringComparison.OrdinalIgnoreCase) || doi.StartsWith("http://", StringComparison.OrdinalIgnoreCase) ? doi : "https://doi.org/" + doi)));
        if (reference.Text("url") is { Length: > 0 } url) actions.Children.Add(Views.Button("link", "打开 URL", () => _ = OpenUrl(url)));
        if (reference.Text("file") is { Length: > 0 } file) actions.Children.Add(Views.Button("folderOpen", "打开文件", () => _ = OpenFile(file)));
        detail.Children.Add(actions);
        foreach (var (key, label) in Metadata)
        {
            var value = reference.Text(key); if (value.Length == 0) continue;
            var section = new StackPanel { Spacing = 4 };
            section.Children.Add(new TextBlock { Text = label, Opacity = .6, FontSize = 12 });
            if (key is "title" or "note" or "abstract_" or "book_title" or "issue") section.Children.Add(new ChunkText(reference.Chunks(key), 14, value));
            else section.Children.Add(new TextBlock { Text = value, TextWrapping = TextWrapping.Wrap, IsTextSelectionEnabled = true, FontFamily = key == "source" ? new FontFamily("Cascadia Mono") : new FontFamily("Segoe UI Variable") });
            detail.Children.Add(section);
        }
    }
    // Mirrors the macOS inspector: same labels, same order. Backslash-free keys are
    // Rust field names; the label column is the macOS wording and must not drift.
    private static readonly (string Key, string Label)[] Metadata = [
        ("author", "作者"), ("year", "年份"), ("month", "月份"), ("journal", "期刊"), ("full_journal", "期刊全称"),
        ("volume", "卷号"), ("number", "编号"), ("pages", "页码"), ("book_pages", "总页数"), ("publisher", "出版社"),
        ("edition", "版本"), ("series", "丛书"), ("editor", "编辑"), ("school", "学校"), ("address", "地址"),
        ("organization", "组织"), ("institution", "机构"), ("doi", "DOI"), ("isbn", "ISBN"), ("mrclass", "MR 分类"),
        ("url", "URL"), ("file", "文件"), ("eprint", "Eprint"), ("archive_prefix", "Archive Prefix"),
        ("arxiv_primary_class", "arXiv 分类"), ("how_published", "发表方式"), ("abstract_", "摘要"),
        ("book_title", "书名"), ("issue", "期号"), ("note", "备注"), ("source", "BibTeX") ];
    private async Task Copy(string text) { try { await RustCore.Copy(text); } catch (Exception error) { await Report(error); } }
    private async Task OpenUrl(string value)
    {
        try { var uri = new Uri(value); if (uri.Scheme is not ("http" or "https")) throw new InvalidOperationException("仅支持 HTTP/HTTPS 链接"); if (!await Launcher.LaunchUriAsync(uri)) throw new IOException(value); }
        catch (Exception error) { await Report(error); }
    }
    private async Task OpenFile(string value)
    {
        try
        {
            var path = value;
            // BibTeX managers commonly store attachments as :path:PDF or label:path:PDF.
            var separator = path.LastIndexOf(':');
            if (separator > 1 && path[(separator + 1)..].Equals("PDF", StringComparison.OrdinalIgnoreCase))
            {
                path = path[..separator];
                var labelSeparator = path.IndexOf(':');
                if (labelSeparator == 0 || labelSeparator > 1) path = path[(labelSeparator + 1)..];
            }
            if (!System.IO.Path.IsPathRooted(path) && current is { } library) path = System.IO.Path.Combine(System.IO.Path.GetDirectoryName(library.Path)!, path);
            if (!await Launcher.LaunchFileAsync(await StorageFile.GetFileFromPathAsync(path))) throw new IOException(path);
        }
        catch (Exception error) { await Report(error); }
    }
    private async Task CheckUpdates()
    {
        await dialogs.WaitAsync();
        try { await Updater.Check(root); }
        catch (Exception error) { await Views.Error(root, error); }
        finally { dialogs.Release(); }
    }
}
