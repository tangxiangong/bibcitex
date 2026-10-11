using BibCiTeX.Core;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace BibCiTeX;

/// <summary>Manages the shared tag catalog and explicit reference associations using revision-checked service mutations.</summary>
internal static class TagManager
{
    /// <summary>Raised immediately after durable metadata changes so open windows invalidate their results even if reloading fails.</summary>
    internal static event Action? Changed;
    /// <summary>Shows the native editor for this XAML root. Service failures are reported without applying a partial change.</summary>
    internal static async Task Show(XamlRoot root, string path = "", string citeKey = "")
    {
        var state = await RustCore.TagSnapshot(new() { path = path, cite_key = citeKey });
        var panel = new StackPanel { Spacing = 10, MinWidth = 420 };
        var list = new ListView { MaxHeight = 220, SelectionMode = ListViewSelectionMode.Single };
        var name = new TextBox { PlaceholderText = L10n.Text("标签名称") };
        var target = new ComboBox { PlaceholderText = L10n.Text("目标标签"), HorizontalAlignment = HorizontalAlignment.Stretch };
        var pending = new ComboBox { PlaceholderText = L10n.Text("待关联记录"), HorizontalAlignment = HorizontalAlignment.Stretch };
        var source = new ComboBox { PlaceholderText = L10n.Text("原文献来源"), HorizontalAlignment = HorizontalAlignment.Stretch };
        var targetKey = new TextBox { PlaceholderText = L10n.Text("目标引用键"), Text = citeKey };
        var targetPath = new TextBox { PlaceholderText = L10n.Text("文献库路径"), Text = path };
        var error = new TextBlock { TextWrapping = TextWrapping.Wrap };
        Localized.Name(list, "标签"); Localized.Name(name, "标签名称"); Localized.Name(target, "目标标签");
        Localized.Name(pending, "待关联记录"); Localized.Name(source, "原文献来源"); Localized.Name(targetKey, "目标引用键"); Localized.Name(targetPath, "文献库路径");
        bool busy = false;
        void Render()
        {
            list.Items.Clear(); target.Items.Clear(); pending.Items.Clear(); source.Items.Clear();
            foreach (var record in state.sources) source.Items.Add(new ComboBoxItem { Tag = record, Content = PathDisplay.Format(record.path) });
            foreach (var tag in state.tags)
            {
                list.Items.Add(new ListViewItem { Tag = tag, Content = (state.assigned_ids.Contains(tag.id) ? "✓ " : "") + tag.name });
                target.Items.Add(new ComboBoxItem { Tag = tag, Content = tag.name });
            }
            foreach (var record in state.pending) pending.Items.Add(new ComboBoxItem { Tag = record, Content = record.cite_key + " · " + PathDisplay.Format(record.path) });
        }
        async Task Mutate(TagAction action)
        {
            if (busy) return;
            var selected = (list.SelectedItem as ListViewItem)?.Tag as TagRecord;
            if (!action.IsCreate && !action.IsRelink && !action.IsRelocate && selected is null) return;
            var destination = (target.SelectedItem as ComboBoxItem)?.Tag as TagRecord;
            var originalSource = (source.SelectedItem as ComboBoxItem)?.Tag as SearchSourceRecord;
            var orphan = (pending.SelectedItem as ComboBoxItem)?.Tag as PendingAssociation;
            if (action.IsMerge && destination is null || action.IsRelink && orphan is null || action.IsRelocate && originalSource is null) return;
            busy = true; foreach (var control in panel.Children.OfType<Control>()) control.IsEnabled = false;
            try
            {
                state = await RustCore.MutateTag(new() { action = action, id = action.IsRelocate ? originalSource!.id : action.IsRelink ? orphan!.record_id : selected?.id ?? "", name = name.Text,
                    target_id = destination?.id ?? "", path = action.IsRelink || action.IsRelocate ? targetPath.Text : path, cite_key = action.IsRelocate ? "" : action.IsRelink ? targetKey.Text : citeKey, expected_revision = state.revision });
                Changed?.Invoke();
                if (action.IsRelocate && originalSource?.path == path)
                    path = state.sources.FirstOrDefault(x => x.id == originalSource.id)?.path ?? targetPath.Text;
                // Read in this inspector's context after every mutation.
                state = await RustCore.TagSnapshot(new() { path = path, cite_key = citeKey });
                error.Text = ""; Render();
            }
            catch (Exception ex)
            {
                error.Text = L10n.ErrorMessage(ex);
                // A revision conflict needs a fresh catalog, never an automatic retry.
                try
                {
                    state = await RustCore.TagSnapshot(new() { path = path, cite_key = citeKey }); Render();
                    list.SelectedItem = list.Items.OfType<ListViewItem>().FirstOrDefault(x => (x.Tag as TagRecord)?.id == selected?.id);
                }
                catch { /* Retain the original storage error and the last readable snapshot. */ }
            }
            finally { busy = false; foreach (var control in panel.Children.OfType<Control>()) control.IsEnabled = true; }
        }
        Button Button(string label, TagAction action)
        {
            var button = new Button { Content = L10n.Text(label) }; button.Click += async (_, _) => await Mutate(action); return button;
        }
        panel.Children.Add(list); panel.Children.Add(name);
        panel.Children.Add(Views.Row(Button("创建", TagAction.Create), Button("重命名", TagAction.Rename), Button("删除", TagAction.Delete)));
        panel.Children.Add(target); panel.Children.Add(Button("合并", TagAction.Merge));
        if (path.Length > 0 && citeKey.Length > 0) panel.Children.Add(Views.Row(Button("添加标签", TagAction.Assign), Button("移除标签", TagAction.Unassign)));
        panel.Children.Add(source); panel.Children.Add(targetPath); panel.Children.Add(Button("重新定位原文件", TagAction.Relocate));
        panel.Children.Add(pending); panel.Children.Add(targetKey); panel.Children.Add(Button("重新关联", TagAction.Relink)); panel.Children.Add(error);
        list.SelectionChanged += (_, _) => { if ((list.SelectedItem as ListViewItem)?.Tag is TagRecord tag) name.Text = tag.name; };
        Render();
        await Views.ShowDialog(new ContentDialog { XamlRoot = root, Title = L10n.Text("标签管理"), Content = new ScrollViewer { Content = panel, MaxHeight = 560 }, CloseButtonText = L10n.Text("关闭") });
    }
}
