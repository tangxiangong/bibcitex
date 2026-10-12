using BibCiTeX.Core;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace BibCiTeX;

/// All parsing, condition construction, grouping and serialization belongs to Rust.
/// <summary>Edits an ordered query tree using the shared parser and semantic operations; cancellation preserves the caller’s draft.</summary>
internal static class AdvancedSearchEditor
{
    /// <summary>Shows the native editor for this XAML root. Service failures are reported without applying a partial change.</summary>
    internal static async Task<QueryAnalysis?> Show(XamlRoot root, string text, QueryAnalysis? bound = null, uint? selectedNode = null)
    {
        var current = bound ?? await RustCore.AnalyzeQuery(new() { text = text });
        var catalog = await RustCore.SearchCatalog();
        var editorSession = new SearchSession(); editorSession.Accept(current);
        var panel = new StackPanel { Spacing = 10, MinWidth = 400 };
        var diagnostics = new TextBlock { TextWrapping = TextWrapping.Wrap };
        var tree = new ListView { MaxHeight = 220, SelectionMode = ListViewSelectionMode.Single };
        var preview = new TextBox { Text = current.text, AcceptsReturn = true, TextWrapping = TextWrapping.Wrap, MaxHeight = 120 };
        editorSession.Attach(preview);
        var editingButtons = new List<Button>();
        var field = new ComboBox { MinWidth = 130 };
        field.Items.Add(Localized.ComboItem("全部字段", ""));
        foreach (var descriptor in catalog.field_descriptors)
        {
            var item = new ComboBoxItem { Tag = descriptor };
            Localized.Bind(item, ContentControl.ContentProperty, descriptor.label_key);
            field.Items.Add(item);
        }
        field.SelectedIndex = 0;
        SearchFieldDescriptor? SelectedField() => (field.SelectedItem as ComboBoxItem)?.Tag as SearchFieldDescriptor;
        string FieldLabel(string name) => catalog.field_descriptors.FirstOrDefault(x => x.name == name) is { } descriptor ? L10n.Text(descriptor.label_key) : name;
        var value = new TextBox { PlaceholderText = L10n.Text("关键词"), MinWidth = 220 };
        var phrase = new CheckBox { Content = L10n.Text("完整短语") };
        var exclude = new CheckBox { Content = L10n.Text("排除") };
        var operation = new ComboBox { ItemsSource = new[] { "AND", "OR" }, SelectedIndex = 0 };
        var tags = new ComboBox { MinWidth = 160 };
        Localized.Name(preview, "查询"); Localized.Name(tree, "条件组"); Localized.Name(field, "字段筛选");
        Localized.Name(value, "关键词"); Localized.Name(operation, "组合方式"); Localized.Name(tags, "标签");
        TagSnapshot? snapshot = null;
        panel.Children.Add(preview);
        panel.Children.Add(tree);
        panel.Children.Add(Views.Row(field, value));
        panel.Children.Add(Views.Row(phrase, exclude, operation));
        panel.Children.Add(tags); tags.Visibility = Visibility.Collapsed;
        var dialog = new ContentDialog { XamlRoot = root, Title = L10n.Text("高级搜索"), Content = new ScrollViewer { Content = panel, MaxHeight = 560 }, PrimaryButtonText = L10n.Text("应用"), CloseButtonText = L10n.Text("取消") };
        bool busy = false, composingEditor = false;
        int previewVersion = 0;
        void TrackComposition(TextBox input)
        {
            input.TextCompositionStarted += (_, _) => { composingEditor = true; dialog.IsPrimaryButtonEnabled = false; };
            input.TextCompositionEnded += (_, _) => { composingEditor = false; dialog.IsPrimaryButtonEnabled = current.status.IsValid && preview.Text == current.text && !busy; };
        }
        TrackComposition(preview); TrackComposition(value);
        void Render()
        {
            editorSession.Accept(current); preview.Text = current.text;
            foreach (var button in editingButtons) button.IsEnabled = !busy && current.status.IsValid;
            diagnostics.Text = SearchSession.Diagnostic(current);
            dialog.IsPrimaryButtonEnabled = current.status.IsValid && !busy;
            tree.Items.Clear();
            if (current.document?.root is not { } rootId) return;
            var stack = new Stack<(uint Id, int Depth)>(); stack.Push((rootId, 0));
            while (stack.TryPop(out var item))
            {
                var node = current.document.nodes[(int)item.Id];
                string label = node.kind.IsAnd ? "AND" : node.kind.IsOr ? "OR" : node.kind.IsNot ? "NOT" : node.kind.IsScope ? FieldLabel(node.text_value) + ":" : node.text_value;
                var row = new ListViewItem { Tag = item.Id, Content = new TextBlock { Text = label, Margin = new Thickness(item.Depth * 12, 0, 0, 0) } };
                tree.Items.Add(row); if (item.Id == selectedNode) tree.SelectedItem = row;
                for (int i = node.children.Count - 1; i >= 0; i--) stack.Push((node.children[i], item.Depth + 1));
            }
        }
        async Task Run(Func<Task<QueryAnalysis>> action)
        {
            if (busy || composingEditor) return; busy = true; dialog.IsPrimaryButtonEnabled = false;
            string? failure = null; int request = previewVersion; bool obsolete = false;
            try
            {
                var next = await action();
                obsolete = request != previewVersion;
                if (!obsolete) { if (next.status.IsValid) current = next; else failure = SearchSession.Diagnostic(next); }
            }
            catch (Exception error) { failure = L10n.ErrorMessage(error); }
            finally
            {
                busy = false;
                if (obsolete) dialog.IsPrimaryButtonEnabled = false;
                else if (failure is null) Render();
                else { diagnostics.Text = failure; dialog.IsPrimaryButtonEnabled = false; }
            }
        }
        Button Action(string title, Func<Task<QueryAnalysis>> action, bool parsesText = false)
        {
            var button = new Button { Content = L10n.Text(title) };
            if (!parsesText) editingButtons.Add(button);
            button.Click += async (_, _) => { if (parsesText || preview.Text == current.text) await Run(action); };
            return button;
        }
        uint? Target() => (tree.SelectedItem as ListViewItem)?.Tag as uint?;
        QueryDocumentRecord Empty() => new() { nodes = [], root = null };
        async Task<QueryAnalysis> Append()
        {
            if (!current.status.IsValid) return current;
            var selected = (tags.SelectedItem as ComboBoxItem)?.Tag as TagRecord;
            var condition = await RustCore.BuildCondition(new() {
                field_name = SelectedField()?.name ?? "", text_value = selected?.name ?? value.Text,
                phrase = phrase.IsEnabled && phrase.IsChecked == true, exclude = exclude.IsChecked == true, tag_id = selected?.id
            });
            if (!condition.status.IsValid) return condition;
            return await RustCore.EditQuery(new() { document = current.document ?? Empty(), target = Target(),
                action = operation.SelectedIndex == 1 ? QueryEditAction.AppendOr : QueryEditAction.AppendAnd, operand = condition.document });
        }
        var basic = new StackPanel { Spacing = 6 };
        foreach (var (label, kind) in new[] { ("包含全部关键词", 0), ("包含完整短语", 1), ("包含任意关键词", 2), ("排除关键词或短语", 3) })
        {
            var input = new TextBox { PlaceholderText = L10n.Text(label), MinWidth = 270 };
            TrackComposition(input);
            var add = Action("添加", async () =>
            {
                if (!current.status.IsValid) return current;
                var operand = kind == 1 ? await RustCore.BuildCondition(new() { field_name = "", text_value = input.Text, phrase = true, exclude = false, tag_id = null })
                    : await RustCore.AnalyzeQuery(new() { text = input.Text });
                if (!operand.status.IsValid || operand.document is not { } document) return operand;
                if (document.root is { } index)
                {
                    var node = document.nodes[(int)index];
                    if (kind == 2 && node.kind.IsAnd) node.kind = QueryNodeKind.Or;
                    if (kind == 3)
                    {
                        var children = node.kind.IsAnd ? node.children.ToList() : new List<uint> { index };
                        // Target indexes can change after an edit; process from the end and re-analyze via the service.
                        if (node.kind.IsAnd)
                        {
                            for (int n = children.Count - 1; n >= 0; n--)
                            {
                                var rootNode = document.nodes[(int)document.root!.Value];
                                var negated = await RustCore.EditQuery(new() { document = document, target = rootNode.children[n], action = QueryEditAction.Negate, operand = null });
                                if (!negated.status.IsValid || negated.document is null) return negated;
                                document = negated.document;
                            }
                        }
                        else
                        {
                            var negated = await RustCore.EditQuery(new() { document = document, target = null, action = QueryEditAction.Negate, operand = null });
                            if (!negated.status.IsValid || negated.document is null) return negated;
                            document = negated.document;
                        }
                    }
                }
                return await RustCore.EditQuery(new() { document = current.document ?? Empty(), target = null, action = QueryEditAction.AppendAnd, operand = document });
            });
            basic.Children.Add(Views.Row(input, add));
        }
        panel.Children.Insert(0, new Expander { Header = L10n.Text("高级搜索"), Content = basic, HorizontalAlignment = HorizontalAlignment.Stretch });
        panel.Children.Add(Views.Row(Action("添加条件", Append),
            Action("整组取反", () => !current.status.IsValid ? Task.FromResult(current) : RustCore.EditQuery(new() { document = current.document ?? Empty(), target = Target(), action = QueryEditAction.Negate, operand = null })),
            Action("移除", () => !current.status.IsValid ? Task.FromResult(current) : RustCore.EditQuery(new() { document = current.document ?? Empty(), target = Target(), action = QueryEditAction.Remove, operand = null }))));
        panel.Children.Add(Action("解析查询", () => editorSession.AnalyzeDraft(preview.Text), parsesText: true));
        panel.Children.Add(diagnostics);
        field.SelectionChanged += async (_, _) =>
        {
            var descriptor = SelectedField();
            phrase.IsEnabled = descriptor?.supports_phrase ?? true;
            if (!phrase.IsEnabled) phrase.IsChecked = false;
            bool tag = descriptor?.name == "tag";
            tags.Visibility = tag ? Visibility.Visible : Visibility.Collapsed; value.Visibility = tag ? Visibility.Collapsed : Visibility.Visible;
            tags.SelectedItem = null;
            if (tag) try { snapshot = await RustCore.TagSnapshot(new() { library_uid = "", record_uid = "", path = "", cite_key = "" }); tags.Items.Clear(); foreach (var candidate in snapshot.tags) tags.Items.Add(new ComboBoxItem { Content = candidate.name, Tag = candidate }); } catch (Exception ex) { diagnostics.Text = L10n.ErrorMessage(ex); }
        };
        preview.TextChanged += (_, _) =>
        {
            previewVersion++;
            editorSession.TextEdited(preview.Text);
            bool unchanged = preview.Text == current.text;
            dialog.IsPrimaryButtonEnabled = unchanged && current.status.IsValid && !busy;
            foreach (var button in editingButtons) button.IsEnabled = unchanged && current.status.IsValid && !busy;
        };
        dialog.PrimaryButtonClick += (_, args) => { if (busy || composingEditor || !current.status.IsValid || preview.Text != current.text) args.Cancel = true; };
        Render();
        return await Views.ShowDialog(dialog) == ContentDialogResult.Primary ? current : null;
    }
}
