using Microsoft.UI;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Documents;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.Storage.Streams;
using Windows.ApplicationModel.DataTransfer;

namespace BibCiTeX;

internal sealed class SvgIcon : UserControl
{
    private string name;
    private readonly Image image = new() { Stretch = Stretch.Uniform };
    internal SvgIcon(string name, double size = 20)
    {
        this.name = name; Width = Height = size;
        Content = image;
        Loaded += (_, _) => Reload(); ActualThemeChanged += (_, _) => Reload();
        AutomationProperties.SetAccessibilityView(this, Microsoft.UI.Xaml.Automation.Peers.AccessibilityView.Raw);
    }
    internal string Icon
    {
        get => name;
        set { if (name == value) return; name = value; Reload(); }
    }
    private void Reload() => image.Source = new SvgImageSource(new Uri($"ms-appx:///Assets/Icons/{(ActualTheme == ElementTheme.Dark ? "Dark/" : "")}{name}.svg"));
}

internal static class Views
{
    internal static TextBlock Text(string text, double size = 14) => new() { Text = text, FontSize = size, TextWrapping = TextWrapping.Wrap };
    internal static TextBlock LocalizedText(string key, double size = 14)
    {
        var text = Text("", size);
        Localized.Bind(text, TextBlock.TextProperty, key);
        return text;
    }
    internal static Button Button(string icon, string name, Action action)
    {
        var button = new Button { Content = new SvgIcon(icon), Padding = new Thickness(8), MinWidth = 32, MinHeight = 32 };
        Localized.Tooltip(button, name); Localized.Name(button, name);
        button.Click += (_, _) => action(); return button;
    }
    // For icons that depend on the active theme: re-resolved whenever the button's
    // theme changes, so the glyph never contradicts the palette it is rendered in.
    internal static Button ThemedButton(Func<string> icon, string name, Action action)
    {
        var glyph = new SvgIcon(icon());
        var button = new Button { Content = glyph, Padding = new Thickness(8), MinWidth = 32, MinHeight = 32 };
        Localized.Tooltip(button, name); Localized.Name(button, name);
        button.ActualThemeChanged += (_, _) => glyph.Icon = icon();
        button.Click += (_, _) => action(); return button;
    }
    private static MenuFlyout SelectionMenu(FrameworkElement source, Func<string> selected, Action copy, Action selectAll)
    {
        var menu = new MenuFlyout();
        var copyItem = new MenuFlyoutItem(); Localized.Bind(copyItem, MenuFlyoutItem.TextProperty, "windowsMenu.copy");
        copyItem.Click += async (_, _) => await InvokeTextAction(source, copy);
        var selectItem = new MenuFlyoutItem(); Localized.Bind(selectItem, MenuFlyoutItem.TextProperty, "menu.selectAll");
        selectItem.Click += async (_, _) => await InvokeTextAction(source, selectAll);
        menu.Items.Add(copyItem); menu.Items.Add(selectItem);
        menu.Opening += (_, _) => copyItem.IsEnabled = selected().Length > 0;
        return menu;
    }
    internal static TextBlock SelectableText(TextBlock text)
    {
        Localized.BindValue(text, FrameworkElement.LanguageProperty, () => L10n.Language);
        text.ContextFlyout = SelectionMenu(text, () => text.SelectedText, text.CopySelectionToClipboard, text.SelectAll);
        text.SelectionFlyout = SelectionMenu(text, () => text.SelectedText, text.CopySelectionToClipboard, text.SelectAll);
        return text;
    }
    internal static void SelectableText(RichTextBlock text)
    {
        Localized.BindValue(text, FrameworkElement.LanguageProperty, () => L10n.Language);
        text.ContextFlyout = SelectionMenu(text, () => text.SelectedText, text.CopySelectionToClipboard, text.SelectAll);
        text.SelectionFlyout = SelectionMenu(text, () => text.SelectedText, text.CopySelectionToClipboard, text.SelectAll);
    }
    internal static void LocalizeTextBox(TextBox box)
    {
        Localized.BindValue(box, FrameworkElement.LanguageProperty, () => L10n.Language);
        MenuFlyout CreateMenu(bool selection)
        {
            var menu = new MenuFlyout();
            MenuFlyoutItem Add(string key, Action action)
            {
                var item = new MenuFlyoutItem();
                Localized.Bind(item, MenuFlyoutItem.TextProperty, key);
                item.Click += async (_, _) => await InvokeTextAction(box, action);
                menu.Items.Add(item); return item;
            }
            MenuFlyoutItem? undo = null, redo = null;
            if (!selection)
            {
                undo = Add("menu.system.Undo", box.Undo); redo = Add("menu.system.Redo", box.Redo);
                menu.Items.Add(new MenuFlyoutSeparator());
            }
            var cut = Add("menu.cut", box.CutSelectionToClipboard);
            var copy = Add("windowsMenu.copy", box.CopySelectionToClipboard);
            var paste = Add("menu.paste", box.PasteFromClipboard);
            var delete = selection ? null : Add("menu.delete", () => box.SelectedText = "");
            menu.Items.Add(new MenuFlyoutSeparator());
            var selectAll = Add("menu.selectAll", box.SelectAll);
            menu.Opening += (_, _) =>
            {
                if (undo is not null) undo.IsEnabled = !box.IsReadOnly && box.CanUndo;
                if (redo is not null) redo.IsEnabled = !box.IsReadOnly && box.CanRedo;
                cut.IsEnabled = !box.IsReadOnly && box.SelectionLength > 0;
                copy.IsEnabled = box.SelectionLength > 0;
                if (delete is not null) delete.IsEnabled = !box.IsReadOnly && box.SelectionLength > 0;
                selectAll.IsEnabled = box.Text.Length > 0;
                try { paste.IsEnabled = !box.IsReadOnly && Clipboard.GetContent().Contains(StandardDataFormats.Text); }
                catch { paste.IsEnabled = false; }
            };
            return menu;
        }
        box.ContextFlyout = CreateMenu(false);
        box.SelectionFlyout = CreateMenu(true);
    }
    internal static StackPanel Row(params UIElement[] children)
    {
        var row = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8 };
        foreach (var child in children) row.Children.Add(child); return row;
    }
    internal static Brush Brush(string resource) => (Brush)Application.Current.Resources[resource];
    internal static void ThemeForeground(FrameworkElement element, string resource)
    {
        var type = element is TextBlock ? "TextBlock" : "Control";
        element.Style = (Style)Microsoft.UI.Xaml.Markup.XamlReader.Load($"<Style xmlns=\"http://schemas.microsoft.com/winfx/2006/xaml/presentation\" TargetType=\"{type}\"><Setter Property=\"Foreground\" Value=\"{{ThemeResource {resource}}}\" /></Style>");
    }
    internal static Border Divider(bool vertical = false)
    {
        var line = (Border)Microsoft.UI.Xaml.Markup.XamlReader.Load("<Border xmlns=\"http://schemas.microsoft.com/winfx/2006/xaml/presentation\" Background=\"{ThemeResource ControlStrokeColorDefaultBrush}\" />");
        if (vertical) line.Width = 1; else line.Height = 1;
        return line;
    }
    internal static ListViewItem LibraryItem(Library library, bool current = false, bool sidebar = false)
    {
        var row = new Grid { ColumnSpacing = 9, Padding = new Thickness(10, 5, 4, 5) };
        row.ColumnDefinitions.Add(new() { Width = GridLength.Auto }); row.ColumnDefinitions.Add(new()); row.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        row.Children.Add(new SvgIcon("library", 15) { VerticalAlignment = VerticalAlignment.Center });
        var stack = new StackPanel { Spacing = 3, VerticalAlignment = VerticalAlignment.Center };
        var title = new TextBlock { Text = library.Name, MaxLines = 1, TextTrimming = TextTrimming.CharacterEllipsis, FontSize = 13 };
        stack.Children.Add(title);
        if (!sidebar || library.Description.Length > 0)
        {
            if (sidebar) stack.Children.Add(new TextBlock { Text = library.Description, Opacity = .6, FontSize = 11, MaxLines = 1, TextTrimming = TextTrimming.CharacterEllipsis });
            else stack.Children.Add(new MiddleEllipsisText { Value = PathDisplay.Format(library.Path), Opacity = .6, HorizontalAlignment = HorizontalAlignment.Stretch });
        }
        Grid.SetColumn(stack, 1); row.Children.Add(stack);
        if (sidebar ? library.Pinned : current)
        {
            var marker = new SvgIcon(sidebar ? "pin" : "check", 13) { VerticalAlignment = VerticalAlignment.Center };
            Grid.SetColumn(marker, 2); row.Children.Add(marker);
        }
        var item = new ListViewItem { Content = row, Tag = library, HorizontalContentAlignment = HorizontalAlignment.Stretch, Padding = new Thickness(0) };
        if (!sidebar) item.Height = 58;
        AutomationProperties.SetName(item, library.Name); return item;
    }
    /// <summary>
    /// A list row for a reference. The cite key sits on its own line at the top
    /// right: it is the value users retype, so it is off the metadata lines where
    /// it competed with the title for width. <paramref name="citeKeyCopies"/>
    /// makes it a copy button — the main window and the tray window take keys,
    /// while the hotkey panel pastes the whole row and shows the key as a label.
    /// </summary>
    internal static ListViewItem ReferenceItem(Reference reference, bool citeKeyCopies = false, Func<string, Task<bool>>? copy = null)
    {
        var details = new StackPanel { Spacing = citeKeyCopies ? 4 : 3, Padding = new Thickness(4, 8, 4, 8) };
        details.Children.Add(new ChunkText(reference.Chunks("title"), citeKeyCopies ? 14 : 13, reference.Title, false, "暂无标题", maxLines: 2));
        if (reference.Authors.Length > 0) details.Children.Add(new TextBlock { Text = reference.Authors, FontSize = citeKeyCopies ? 12 : 11, TextTrimming = TextTrimming.CharacterEllipsis, Opacity = .75 });
        var meta = string.Join(" · ", new[] { reference.TypeLabel, reference.Text("year"), reference.Venue }.Where(x => x.Length > 0));
        if (meta.Length > 0)
        {
            var metadata = new TextBlock { FontSize = 11, TextTrimming = TextTrimming.CharacterEllipsis, Opacity = .6 };
            Localized.BindValue(metadata, TextBlock.TextProperty, () => string.Join(" · ", new[] { reference.TypeLabel, reference.Text("year"), reference.Venue }.Where(x => x.Length > 0)));
            details.Children.Add(metadata);
        }

        var row = new Grid { ColumnSpacing = 8 };
        row.ColumnDefinitions.Add(new() { Width = new GridLength(1, GridUnitType.Star) });
        row.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        if (!citeKeyCopies)
        {
            row.ColumnDefinitions.Insert(0, new() { Width = GridLength.Auto });
            row.Children.Add(new SvgIcon("fileText", 15) { Margin = new Thickness(0, 10, 0, 0), VerticalAlignment = VerticalAlignment.Top });
            Grid.SetColumn(details, 1);
        }
        row.Children.Add(details);
        // Assigned through the base type: the two arms are a Button and a TextBlock.
        FrameworkElement key = citeKeyCopies
            ? CiteKey(reference.Key, copy)
            : new TextBlock { Text = reference.Key, FontSize = 11, FontFamily = new FontFamily("Cascadia Mono"), TextTrimming = TextTrimming.CharacterEllipsis, Foreground = Brush("SystemControlForegroundAccentBrush"), VerticalAlignment = VerticalAlignment.Top, Margin = new Thickness(0, 10, 4, 0) };
        key.MaxWidth = 180;
        row.SizeChanged += (_, _) => key.MaxWidth = Math.Max(60, row.ActualWidth * .4);
        if (!citeKeyCopies) { ((TextBlock)key).ClearValue(TextBlock.ForegroundProperty); ThemeForeground(key, "SystemControlForegroundAccentBrush"); }
        Grid.SetColumn(key, citeKeyCopies ? 1 : 2); row.Children.Add(key);

        var item = new ListViewItem { Content = row, Tag = reference, HorizontalContentAlignment = HorizontalAlignment.Stretch };
        if (!citeKeyCopies) { item.MinHeight = 82; item.Padding = new Thickness(0); }
        if (citeKeyCopies && copy is not null)
        {
            var menu = new MenuFlyout();
            foreach (var (label, value) in new[] { ("复制引用键", reference.Key), ("复制 BibTeX", reference.Text("source")) })
            {
                var action = new MenuFlyoutItem(); Localized.Bind(action, MenuFlyoutItem.TextProperty, label);
                action.Click += async (_, _) => await copy(value); menu.Items.Add(action);
            }
            item.ContextFlyout = menu;
        }
        Localized.BindValue(item, AutomationProperties.NameProperty, () => reference.Title + " " + reference.Authors); return item;
    }
    /// <summary>
    /// The cite key as its own copy control, pinned to the top right of a row. It
    /// confirms on itself for a moment, so a copy that worked says so without a
    /// dialog to dismiss, and it never runs the row's own activate action.
    /// </summary>
    internal static Button CiteKey(string key, Func<string, Task<bool>>? copy = null)
    {
        var glyph = new SvgIcon("copy", 10);
        var label = new TextBlock { Text = key, FontSize = 11, FontFamily = new FontFamily("Cascadia Mono"), TextTrimming = TextTrimming.CharacterEllipsis, VerticalAlignment = VerticalAlignment.Center };
        var content = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 4 };
        content.Children.Add(glyph); content.Children.Add(label);
        var button = new Button
        {
            Content = content,
            Padding = new Thickness(7, 3, 7, 3),
            MinWidth = 0, MinHeight = 0,
            CornerRadius = new CornerRadius(11),
            Background = new SolidColorBrush(ColorHelper.FromArgb(20, 128, 128, 128)),
            BorderThickness = new Thickness(0),
            VerticalAlignment = VerticalAlignment.Top,
            Margin = new Thickness(0, 4, 4, 0),
        };
        Localized.Tooltip(button, "复制引用键");
        Localized.Bind(button, AutomationProperties.NameProperty, "复制引用键 {0}", key);
        ThemeForeground(button, "SystemControlForegroundAccentBrush");
        var confirmation = 0;
        void Reset()
        {
            ThemeForeground(button, "SystemControlForegroundAccentBrush");
            glyph.Icon = "copy"; Localized.BindValue(label, TextBlock.TextProperty, () => key); label.FontFamily = new FontFamily("Cascadia Mono");
        }
        button.Click += async (_, _) =>
        {
            var request = ++confirmation;
            Reset();
            if (copy is null || !await copy(key) || request != confirmation) return;
            ThemeForeground(button, "SystemFillColorSuccessBrush");
            glyph.Icon = "check"; Localized.Bind(label, TextBlock.TextProperty, "已复制"); label.FontFamily = new FontFamily("Segoe UI Variable");
            await Task.Delay(1400);
            if (request == confirmation) Reset();
        };
        button.Unloaded += (_, _) => { confirmation++; Reset(); };
        return button;
    }
    /// <summary>Native copy control with repeat-safe, local success feedback.</summary>
    internal static Button CopyButton(string icon, string name, Func<Task<bool>> action, bool showLabel = false)
    {
        var glyph = new SvgIcon(icon, showLabel ? 12 : 16);
        var label = LocalizedText(name, 12); label.Visibility = showLabel ? Visibility.Visible : Visibility.Collapsed;
        var content = Row(glyph, label); content.Spacing = 4;
        var button = new Button { Content = content, Padding = new Thickness(8, 6, 8, 6), MinWidth = 32, MinHeight = 32 };
        Localized.Tooltip(button, name); Localized.Name(button, name);
        var confirmation = 0;
        void Reset()
        {
            glyph.Icon = icon; Localized.Bind(label, TextBlock.TextProperty, name); label.Visibility = showLabel ? Visibility.Visible : Visibility.Collapsed;
        }
        button.Click += async (_, _) =>
        {
            var request = ++confirmation;
            Reset();
            if (!await action() || request != confirmation) return;
            glyph.Icon = "check"; label.Visibility = Visibility.Visible; Localized.Bind(label, TextBlock.TextProperty, "已复制");
            await Task.Delay(1400);
            if (request == confirmation) Reset();
        };
        button.Unloaded += (_, _) => { confirmation++; Reset(); };
        return button;
    }
    private static async Task InvokeTextAction(FrameworkElement source, Action action)
    {
        try { action(); }
        catch (Exception error) { await Error(source, error); }
    }
    internal static Task<ContentDialogResult> ShowDialog(ContentDialog dialog)
        => DialogQueue.Run(dialog.XamlRoot ?? throw new InvalidOperationException("Dialog XamlRoot is missing."), async () =>
        {
            dialog.RequestedTheme = App.Theme;
            return await dialog.ShowAsync();
        });
    internal static async Task Error(FrameworkElement root, Exception error)
    {
        if (root.XamlRoot is null) return;
        var dialog = Localized.Dialog(new ContentDialog { XamlRoot = root.XamlRoot }, "错误");
        Localized.Error(dialog, ContentControl.ContentProperty, error);
        await ShowDialog(dialog);
    }
}

// Native RichTextBlock text and Skia-rendered formula runs; no HTML/WebView surface.
internal sealed class ChunkText : UserControl
{
    private readonly List<Chunk> chunks;
    private readonly string? fallbackKey;
    private readonly RichTextBlock text = new() { IsTextSelectionEnabled = true, TextWrapping = TextWrapping.Wrap };
    private int generation;
    private double lastScale;
    private double lastWidth;
    private XamlRoot? observedRoot;
    internal ChunkText(List<Chunk> chunks, double size, string fallback = "", bool selectable = true, string? fallbackKey = null, int maxLines = 0)
    {
        this.fallbackKey = chunks.Count == 0 ? fallbackKey : null;
        this.chunks = chunks.Count == 0 ? [new("normal", fallback)] : chunks;
        FontSize = size; text.FontSize = size; text.MaxLines = maxLines; text.TextTrimming = maxLines > 0 ? TextTrimming.CharacterEllipsis : TextTrimming.None; text.IsTextSelectionEnabled = selectable; Content = text;
        if (selectable) Views.SelectableText(text);
        Loaded += (_, _) => { if (this.fallbackKey is not null) L10n.Changed += Render; observedRoot = XamlRoot; if (observedRoot is not null) observedRoot.Changed += RootChanged; Render(); };
        Unloaded += (_, _) => { if (this.fallbackKey is not null) L10n.Changed -= Render; generation++; if (observedRoot is not null) observedRoot.Changed -= RootChanged; observedRoot = null; };
        ActualThemeChanged += (_, _) => Render();
        SizeChanged += (_, args) => { if (args.NewSize.Width > 0 && Math.Abs(args.NewSize.Width - lastWidth) > 1) Render(); };
    }
    private void RootChanged(XamlRoot sender, XamlRootChangedEventArgs args)
    { if (Math.Abs(sender.RasterizationScale - lastScale) > .001) Render(); }
    private async void Render()
    {
        var version = ++generation;
        var paragraph = new Paragraph(); text.Blocks.Clear(); text.Blocks.Add(paragraph);
        var dark = ActualTheme == ElementTheme.Dark;
        var scale = lastScale = XamlRoot?.RasterizationScale ?? 1;
        var availableWidth = lastWidth = ActualWidth > 0 ? ActualWidth : 640;
        foreach (var chunk in fallbackKey is { } key ? new List<Chunk> { new("normal", L10n.Text(key)) } : chunks)
        {
            if (version != generation) return;
            if (chunk.Kind != "math") { paragraph.Inlines.Add(new Run { Text = chunk.Text }); continue; }
            try
            {
                var fontSize = FontSize;
                var formula = await Task.Run(() => FormulaRenderer.Render(chunk.Text, (float)fontSize, (float)scale, availableWidth, dark));
                if (version != generation) return;
                using var stream = new InMemoryRandomAccessStream();
                using (var writer = new DataWriter(stream.GetOutputStreamAt(0))) { writer.WriteBytes(formula.Bytes); await writer.StoreAsync(); }
                stream.Seek(0);
                var bitmap = new BitmapImage(); await bitmap.SetSourceAsync(stream);
                if (version != generation) return;
                var image = new Image { Source = bitmap, Width = formula.Width, Height = formula.Height, VerticalAlignment = VerticalAlignment.Center };
                AutomationProperties.SetName(image, chunk.Text);
                paragraph.Inlines.Add(new InlineUIContainer { Child = image });
            }
            catch { if (version == generation) paragraph.Inlines.Add(new Run { Text = chunk.Text }); }
        }
    }
}
