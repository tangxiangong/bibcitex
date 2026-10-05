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
    internal static ListViewItem LibraryItem(Library library)
    {
        var stack = new StackPanel { Spacing = 4, Padding = new Thickness(4, 8, 4, 8) };
        stack.Children.Add(Text(library.Name));
        stack.Children.Add(new TextBlock { Text = library.Path, Opacity = .6, FontSize = 11, TextTrimming = TextTrimming.CharacterEllipsis });
        var item = new ListViewItem { Content = stack, Tag = library, HorizontalContentAlignment = HorizontalAlignment.Stretch };
        AutomationProperties.SetName(item, library.Name); return item;
    }
    internal static ListViewItem ReferenceItem(Reference reference)
    {
        var row = new StackPanel { Spacing = 4, Padding = new Thickness(4, 8, 4, 8) };
        row.Children.Add(new ChunkText(reference.Chunks("title"), 15, reference.Title, false));
        if (reference.Authors.Length > 0) row.Children.Add(new TextBlock { Text = reference.Authors, FontSize = 12, TextTrimming = TextTrimming.CharacterEllipsis, Opacity = .75 });
        row.Children.Add(new TextBlock { Text = reference.Key, FontSize = 12, FontFamily = new FontFamily("Cascadia Mono"), TextTrimming = TextTrimming.CharacterEllipsis });
        var meta = string.Join(" · ", new[] { reference.TypeLabel, reference.Venue, reference.Text("year") }.Where(x => x.Length > 0));
        if (meta.Length > 0) row.Children.Add(new TextBlock { Text = meta, FontSize = 11, TextTrimming = TextTrimming.CharacterEllipsis, Opacity = .6 });
        var item = new ListViewItem { Content = row, Tag = reference, HorizontalContentAlignment = HorizontalAlignment.Stretch };
        AutomationProperties.SetName(item, reference.Title + " " + reference.Authors); return item;
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
