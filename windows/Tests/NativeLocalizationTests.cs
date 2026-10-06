using System.Runtime.CompilerServices;
using BibCiTeX.Core;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace BibCiTeX;

// Runs in a separately compiled test application with a temporary visual host.
// It does not touch preferences, register a hotkey or start the updater.
internal static class NativeLocalizationTests
{
    internal static async Task Run()
    {
        L10n.Select("en");
        var root = CreateControls();
        // SelectionBoxItem is populated only after the ComboBox joins a live visual tree.
        var window = new Window { Content = root };
        var loaded = new TaskCompletionSource();
        root.Loaded += (_, _) => loaded.TrySetResult();
        window.Activate();
        await loaded.Task.WaitAsync(TimeSpan.FromSeconds(10));
        await CheckSelectedFilters(root);
        var markdown = UpdateMarkdown.Render("# Notes\n\nText\n\n```\ncode\n```");
        await Task.Delay(50);
        Assert(root, "File", "References", "All types", "0 references", "Journal article");
        GC.Collect(); GC.WaitForPendingFinalizers(); GC.Collect();
        foreach (var language in new[] { "zh-Hans", "en", "zh-Hans" })
        {
            L10n.Select(language);
            await Task.Delay(50);
            if (language == "en") Assert(root, "File", "References", "All types", "0 references", "Journal article");
            else Assert(root, "文件", "文献", "全部类型", "0 条文献", "期刊论文");
            AssertMarkdown(markdown, language == "en" ? "Copy" : "复制", language == "en" ? "Select All" : "全选");
        }
        var menu = (MenuBar)root.Children[0];
        if (((MenuFlyoutItem)menu.Items[0].Items[0]).Text != "新增文献库") throw new Exception("Flyout item did not update");
        var count = (TextBlock)root.Children[2];
        Localized.Count(count, 1);
        L10n.Select("en"); await Task.Delay(50);
        if (count.Text != "1 reference") throw new Exception("Replaced count binding retained old state");
        GC.KeepAlive(root);
        GC.KeepAlive(window);
    }

    [MethodImpl(MethodImplOptions.NoInlining)]
    private static StackPanel CreateControls()
    {
        var root = new StackPanel();
        var menu = new MenuBar();
        foreach (var key in new[] { "menu.file", "文献" })
        {
            var group = new MenuBarItem(); Localized.Bind(group, MenuBarItem.TitleProperty, key);
            var command = new MenuFlyoutItem(); Localized.Bind(command, MenuFlyoutItem.TextProperty, "新增文献库");
            group.Items.Add(command); menu.Items.Add(group);
        }
        root.Children.Add(menu);
        var filter = new ComboBox();
        var option = Localized.ComboItem("全部类型", "all");
        filter.Items.Add(option); filter.SelectedIndex = 0; root.Children.Add(filter);
        var count = new TextBlock(); Localized.Count(count, 0); root.Children.Add(count);
        var record = new ReferenceRecord
        {
            id = "test", cite_key = "原文-key", source = "@article{原文-key}", entry_type = "Article",
            author = ["原文作者"], title = [], journal = null, year = null, full_journal = null,
            volume = null, number = null, pages = null, note = [], doi = null, mrclass = null,
            publisher = [], isbn = null, series = null, url = null, file = null, abstract_text = [],
            edition = null, issue = [], book_pages = null, school = null, address = null,
            book_title = [], editor = [], month = null, organization = [], institution = null,
            eprint = null, archive_prefix = null, arxiv_primary_class = null, how_published = null
        };
        root.Children.Add(Views.ReferenceItem(new Reference(record), citeKeyCopies: true));
        return root;
    }

    private static void Assert(StackPanel root, string file, string references, string all, string count, string type)
    {
        var menu = (MenuBar)root.Children[0];
        if (menu.Items[0].Title != file || menu.Items[1].Title != references) throw new Exception("Menu bar did not update");
        if (((ComboBoxItem)((ComboBox)root.Children[1]).Items[0]).Content is not LocalizationValue label || label.Value != all) throw new Exception("Filter did not update");
        if (((TextBlock)root.Children[2]).Text != count) throw new Exception("Count did not update");
        var item = (ListViewItem)root.Children[3];
        // ReferenceItem's content is native-owned; no managed child wrappers are retained by the test.
        var row = (Grid)item.Content;
        var details = (StackPanel)row.Children[0];
        if (((TextBlock)details.Children[2]).Text != type) throw new Exception("Existing reference type did not update");
        if (((TextBlock)details.Children[1]).Text != "原文作者") throw new Exception("Bibliography data was translated");
        var expectedTitle = L10n.Text("暂无标题") + " 原文作者";
        if (Microsoft.UI.Xaml.Automation.AutomationProperties.GetName(item) != expectedTitle) throw new Exception("Untitled accessible name did not update");
    }

    private static async Task CheckSelectedFilters(StackPanel host)
    {
        foreach (var (key, alternate, tag, english, alternateEnglish) in new[]
        {
            ("全部类型", "期刊论文", "Article", "All types", "Journal article"),
            ("全部字段", "作者", "author", "All fields", "Authors")
        })
        {
            var filter = new ComboBox();
            filter.Items.Add(Localized.ComboItem(key, "all"));
            filter.Items.Add(Localized.ComboItem(alternate, tag));
            host.Children.Add(filter);
            foreach (var index in new[] { 0, 1 })
            {
                L10n.Select("en");
                filter.SelectedIndex = index;
                filter.ApplyTemplate();
                host.UpdateLayout();
                await Task.Delay(50);
                var selected = (ComboBoxItem)filter.SelectedItem;
                if (filter.SelectionBoxItem is not LocalizationValue selectedLabel
                    || !ReferenceEquals(selectedLabel, selected.Content)
                    || filter.SelectionBoxItemTemplate is not DataTemplate template)
                    throw new Exception("Collapsed filter does not retain the live localization source/template");
                // Exercise the actual selection-box template with the cached selected item.
                var collapsed = (TextBlock)template.LoadContent();
                collapsed.DataContext = filter.SelectionBoxItem;
                var dropdown = (TextBlock)selected.ContentTemplate.LoadContent();
                dropdown.DataContext = selected.Content;
                var changes = 0;
                void Changed(object sender, SelectionChangedEventArgs args) => changes++;
                filter.SelectionChanged += Changed;
                GC.Collect(); GC.WaitForPendingFinalizers(); GC.Collect();
                foreach (var language in new[] { "zh-Hans", "en", "zh-Hans" })
                {
                    L10n.Select(language); await Task.Delay(50);
                    var expected = language == "en" ? (index == 0 ? english : alternateEnglish) : (index == 0 ? key : alternate);
                    if (collapsed.Text != expected || dropdown.Text != expected)
                        throw new Exception("Selected filter label did not refresh in both presentations");
                    if (Microsoft.UI.Xaml.Automation.AutomationProperties.GetName(selected) != expected
                        || selectedLabel.ToString() != expected)
                        throw new Exception("Selected filter accessibility/search text did not refresh");
                    if (!ReferenceEquals(filter.SelectedItem, selected) || !ReferenceEquals(filter.SelectionBoxItem, selectedLabel)
                        || filter.SelectedIndex != index || selected.Tag as string != (index == 0 ? "all" : tag) || changes != 0)
                        throw new Exception("Language switch changed filtering or triggered a new search");
                }
                filter.SelectionChanged -= Changed;
            }
            host.Children.Remove(filter);
        }
        L10n.Select("en");
    }

    private static void AssertMarkdown(UIElement markdown, string copy, string selectAll)
    {
        var blocks = Descendants(markdown).OfType<TextBlock>().Where(text => text.IsTextSelectionEnabled).ToArray();
        if (blocks.Length != 3) throw new Exception("Markdown fixture did not render heading, paragraph and code");
        foreach (var block in blocks)
        {
            foreach (var flyout in new[] { block.ContextFlyout, block.SelectionFlyout })
            {
                if (flyout is not MenuFlyout menu || menu.Items.Count != 2
                    || ((MenuFlyoutItem)menu.Items[0]).Text != copy
                    || ((MenuFlyoutItem)menu.Items[1]).Text != selectAll)
                    throw new Exception("Existing Markdown selection menu did not change language");
            }
        }
    }

    private static IEnumerable<DependencyObject> Descendants(DependencyObject parent)
    {
        yield return parent;
        if (parent is Panel panel)
            foreach (var child in panel.Children)
                foreach (var descendant in Descendants(child)) yield return descendant;
    }
}
