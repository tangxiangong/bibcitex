using System.Runtime.CompilerServices;
using BibCiTeX.Core;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace BibCiTeX;

// Runs in a separately compiled test application, without opening a window,
// touching preferences, registering a hotkey or starting the updater.
internal static class NativeLocalizationTests
{
    internal static async Task Run()
    {
        L10n.Select("en");
        var root = CreateControls();
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
        var option = new ComboBoxItem(); Localized.Bind(option, ContentControl.ContentProperty, "全部类型");
        filter.Items.Add(option); root.Children.Add(filter);
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
        if (((ComboBoxItem)((ComboBox)root.Children[1]).Items[0]).Content as string != all) throw new Exception("Filter did not update");
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
