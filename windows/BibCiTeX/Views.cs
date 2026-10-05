using Microsoft.UI;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Documents;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.Storage.Streams;

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
    internal static Button Button(string icon, string name, Action action)
    {
        var button = new Button { Content = new SvgIcon(icon), Padding = new Thickness(8), MinWidth = 32, MinHeight = 32 };
        ToolTipService.SetToolTip(button, name); AutomationProperties.SetName(button, name);
        button.Click += (_, _) => action(); return button;
    }
    // For icons that depend on the active theme: re-resolved whenever the button's
    // theme changes, so the glyph never contradicts the palette it is rendered in.
    internal static Button ThemedButton(Func<string> icon, string name, Action action)
    {
        var glyph = new SvgIcon(icon());
        var button = new Button { Content = glyph, Padding = new Thickness(8), MinWidth = 32, MinHeight = 32 };
        ToolTipService.SetToolTip(button, name); AutomationProperties.SetName(button, name);
        button.ActualThemeChanged += (_, _) => glyph.Icon = icon();
        button.Click += (_, _) => action(); return button;
    }
    internal static StackPanel Row(params UIElement[] children)
    {
        var row = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8 };
        foreach (var child in children) row.Children.Add(child); return row;
    }
    internal static ListViewItem LibraryItem(Library library, bool current = false)
    {
        var stack = new StackPanel { Spacing = 4, Padding = new Thickness(4, 8, 4, 8) };
        var title = Views.Row(Text(library.Name));
        if (current) title.Children.Add(new SvgIcon("check", 13) { VerticalAlignment = VerticalAlignment.Center });
        stack.Children.Add(title);
        stack.Children.Add(new TextBlock { Text = library.Path, Opacity = .6, FontSize = 11, TextTrimming = TextTrimming.CharacterEllipsis });
        var item = new ListViewItem { Content = stack, Tag = library, HorizontalContentAlignment = HorizontalAlignment.Stretch };
        AutomationProperties.SetName(item, library.Name); return item;
    }
    /// <summary>
    /// A list row for a reference. The cite key sits on its own line at the top
    /// right: it is the value users retype, so it is off the metadata lines where
    /// it competed with the title for width. <paramref name="citeKeyCopies"/>
    /// makes it a copy button — the main window and the tray window take keys,
    /// while the hotkey panel pastes the whole row and shows the key as a label.
    /// </summary>
    internal static ListViewItem ReferenceItem(Reference reference, bool citeKeyCopies = false)
    {
        var details = new StackPanel { Spacing = 4, Padding = new Thickness(4, 8, 4, 8) };
        details.Children.Add(new ChunkText(reference.Chunks("title"), 15, reference.Title, false));
        if (reference.Authors.Length > 0) details.Children.Add(new TextBlock { Text = reference.Authors, FontSize = 12, TextTrimming = TextTrimming.CharacterEllipsis, Opacity = .75 });
        var meta = string.Join(" · ", new[] { reference.TypeLabel, reference.Venue, reference.Text("year") }.Where(x => x.Length > 0));
        if (meta.Length > 0) details.Children.Add(new TextBlock { Text = meta, FontSize = 11, TextTrimming = TextTrimming.CharacterEllipsis, Opacity = .6 });

        var row = new Grid { ColumnSpacing = 8 };
        row.ColumnDefinitions.Add(new() { Width = new GridLength(1, GridUnitType.Star) });
        row.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        row.Children.Add(details);
        // Assigned through the base type: the two arms are a Button and a TextBlock.
        FrameworkElement key = citeKeyCopies
            ? CiteKey(reference.Key)
            : new TextBlock { Text = reference.Key, FontSize = 11, FontFamily = new FontFamily("Cascadia Mono"), TextTrimming = TextTrimming.CharacterEllipsis, Foreground = new SolidColorBrush(Windows.UI.Color.FromArgb(255, 0, 120, 212)), VerticalAlignment = VerticalAlignment.Top, Margin = new Thickness(0, 10, 4, 0) };
        Grid.SetColumn(key, 1); row.Children.Add(key);

        var item = new ListViewItem { Content = row, Tag = reference, HorizontalContentAlignment = HorizontalAlignment.Stretch };
        AutomationProperties.SetName(item, reference.Title + " " + reference.Authors); return item;
    }
    /// <summary>
    /// The cite key as its own copy control, pinned to the top right of a row. It
    /// confirms on itself for a moment, so a copy that worked says so without a
    /// dialog to dismiss, and it never runs the row's own activate action.
    /// </summary>
    internal static Button CiteKey(string key)
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
        ToolTipService.SetToolTip(button, "复制引用键");
        AutomationProperties.SetName(button, "复制引用键 " + key);
        button.Click += async (_, _) =>
        {
            if (!await Views.CopyKey(key)) return;
            glyph.Icon = "check"; label.Text = "已复制"; label.FontFamily = new FontFamily("Segoe UI Variable");
            await Task.Delay(1400);
            glyph.Icon = "copy"; label.Text = key; label.FontFamily = new FontFamily("Cascadia Mono");
        };
        return button;
    }
    /// <summary>Copies a cite key, reporting a failure the way the rest of the app does.</summary>
    internal static async Task<bool> CopyKey(string key)
    {
        try { await RustCore.Copy(key); return true; }
        catch (Exception) { return false; }
    }
    /// <summary>An icon button that flips to a check for a moment once its action succeeded.</summary>
    internal static Button CopyButton(string icon, string name, Func<Task<bool>> action)
    {
        var glyph = new SvgIcon(icon);
        var button = new Button { Content = glyph, Padding = new Thickness(8), MinWidth = 32, MinHeight = 32 };
        ToolTipService.SetToolTip(button, name); AutomationProperties.SetName(button, name);
        button.Click += async (_, _) =>
        {
            if (!await action()) return;
            glyph.Icon = "check";
            await Task.Delay(1400);
            glyph.Icon = icon;
        };
        return button;
    }
    internal static async Task Error(FrameworkElement root, Exception error)
    {
        if (root.XamlRoot is null) return;
        await new ContentDialog { XamlRoot = root.XamlRoot, Title = "BibCiTeX", Content = error.Message, CloseButtonText = "确定" }.ShowAsync();
    }
}

// Native RichTextBlock text and Skia-rendered formula runs; no HTML/WebView surface.
internal sealed class ChunkText : UserControl
{
    private readonly List<Chunk> chunks;
    private readonly RichTextBlock text = new() { IsTextSelectionEnabled = true, TextWrapping = TextWrapping.Wrap };
    private int generation;
    private double lastScale;
    private double lastWidth;
    private XamlRoot? observedRoot;
    internal ChunkText(List<Chunk> chunks, double size, string fallback = "", bool selectable = true)
    {
        this.chunks = chunks.Count == 0 ? [new("normal", fallback)] : chunks;
        FontSize = size; text.FontSize = size; text.IsTextSelectionEnabled = selectable; Content = text;
        Loaded += (_, _) => { observedRoot = XamlRoot; if (observedRoot is not null) observedRoot.Changed += RootChanged; Render(); };
        Unloaded += (_, _) => { generation++; if (observedRoot is not null) observedRoot.Changed -= RootChanged; observedRoot = null; };
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
        foreach (var chunk in chunks)
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
