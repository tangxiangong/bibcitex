using BibCiTeX.Core;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Media;

namespace BibCiTeX;

/// <summary>Transactional native flyout with inline tag chips, never a tag combo box.</summary>
internal sealed class TagFilterFlyout
{
    private TagFilter draft;
    private TagFilter target;
    private TagFilter? replacing;
    private List<TagRecord> tags = [];
    private readonly Flyout flyout = new() { Placement = FlyoutPlacementMode.Bottom };
    private readonly StackPanel panel = new() { Spacing = 10, Width = 480 };
    private readonly StackPanel tree = new();
    private readonly TagManager.DetailTagPanel palette = new();
    private readonly TextBlock expression = new() { TextWrapping = TextWrapping.Wrap, MaxLines = 3, IsTextSelectionEnabled = true };
    private readonly TextBlock error = new() { TextWrapping = TextWrapping.Wrap };
    private readonly TextBlock validation = new() { TextWrapping = TextWrapping.Wrap };
    private readonly TextBlock heading = new();
    private readonly ProgressRing progress = new() { IsActive = true, Width = 20, Height = 20 };
    private readonly TextBox search = new() { PlaceholderText = L10n.Text("搜索标签") };
    private readonly Button apply;
    private bool loaded;
    private bool closed;
    private int loadVersion;

    internal static Button CreateButton(Func<TagFilter> current, Action<TagFilter> commit)
    {
        var button = new Button { Content = Views.Row(new SvgIcon("tag", 16), Views.LocalizedText("标签筛选")) };
        Localized.Name(button, "标签筛选");
        button.Click += (_, _) =>
        {
            var editor = new TagFilterFlyout(current(), value =>
            {
                commit(value);
                AutomationProperties.SetItemStatus(button, L10n.Text(value.IsEmpty ? "未启用" : "已启用"));
                button.Style = value.IsEmpty ? null : (Style)Application.Current.Resources["AccentButtonStyle"];
            });
            editor.flyout.ShowAt(button);
        };
        return button;
    }

    private TagFilterFlyout(TagFilter filter, Action<TagFilter> commit)
    {
        draft = filter.Clone(); target = draft;
        apply = TextButton("应用", () => { commit(draft.IsEmpty ? new() : draft); flyout.Hide(); });
        apply.Style = (Style)Application.Current.Resources["AccentButtonStyle"];
        panel.Children.Add(Views.LocalizedText("标签筛选", 16));
        panel.Children.Add(new ScrollViewer { Content = tree, MaxHeight = 280,
            HorizontalScrollBarVisibility = ScrollBarVisibility.Auto, VerticalScrollBarVisibility = ScrollBarVisibility.Auto });
        panel.Children.Add(expression); panel.Children.Add(validation); panel.Children.Add(Views.Divider());
        panel.Children.Add(heading); panel.Children.Add(search); panel.Children.Add(progress); panel.Children.Add(error);
        var retry = new Button { Content = L10n.Text("重试") };
        retry.Click += (_, _) => { _ = Load(); };
        retry.Visibility = Visibility.Collapsed;
        panel.Children.Add(retry);
        panel.Children.Add(new ScrollViewer { Content = palette, MaxHeight = 96,
            HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled, VerticalScrollBarVisibility = ScrollBarVisibility.Auto });
        panel.Children.Add(Views.Divider());
        var footer = new Grid();
        footer.ColumnDefinitions.Add(new()); footer.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        var clear = TextButton("清空", () => { draft = new(); target = draft; replacing = null; Render(); });
        clear.HorizontalAlignment = HorizontalAlignment.Left; footer.Children.Add(clear);
        var actions = Views.Row(TextButton("取消", flyout.Hide), apply);
        Grid.SetColumn(actions, 1); footer.Children.Add(actions); panel.Children.Add(footer);
        search.TextChanged += (_, _) => RenderPalette();
        AutomationProperties.SetName(search, L10n.Text("搜索标签"));
        flyout.Content = panel;
        // Keep WinUI's presenter background, shadow, focus containment and light-dismiss behavior.
        var presenter = new Style(typeof(FlyoutPresenter));
        presenter.Setters.Add(new Setter(FrameworkElement.MaxWidthProperty, 560d));
        flyout.FlyoutPresenterStyle = presenter;
        flyout.Opened += async (_, _) => { TagManager.Changed += Changed; await Load(); };
        flyout.Closed += (_, _) => { closed = true; loadVersion++; TagManager.Changed -= Changed; };
        async Task Load()
        {
            var version = ++loadVersion;
            loaded = false; progress.IsActive = true; progress.Visibility = Visibility.Visible; apply.IsEnabled = draft.IsEmpty; error.Text = ""; retry.Visibility = Visibility.Collapsed;
            try
            {
                var snapshot = await RustCore.TagSnapshot(new() { path = "", cite_key = "", library_uid = "", record_uid = "" });
                if (closed || version != loadVersion) return;
                var catalogIds = snapshot.tags.Select(tag => tag.id).ToHashSet(StringComparer.Ordinal);
                var redirects = new Dictionary<string, string>(StringComparer.Ordinal);
                // Ask Rust to distinguish merge aliases from deleted bindings.
                foreach (var id in draft.TagIds.Except(catalogIds).ToArray())
                {
                    var refreshed = await RustCore.RefreshQuery(new TagFilter { TagId = id }.Document());
                    if (closed || version != loadVersion) return;
                    var resolved = refreshed.document?.nodes.FirstOrDefault(node => node.tag_id is not null)?.tag_id;
                    if (refreshed.status.IsValid && resolved is not null && catalogIds.Contains(resolved))
                        redirects[id] = resolved;
                }
                tags = snapshot.tags;
                foreach (var (oldId, newId) in redirects) draft.ReplaceTagId(oldId, newId);
                loaded = true; Render();
            }
            catch (Exception ex)
            {
                if (closed || version != loadVersion) return;
                error.Text = L10n.ErrorMessage(ex); retry.Visibility = Visibility.Visible;
            }
            if (closed || version != loadVersion) return;
            progress.IsActive = false; progress.Visibility = Visibility.Collapsed;
        }
        void Changed() { _ = Load(); }
        Render();
    }

    private static Button TextButton(string key, Action action)
    {
        var button = new Button { Content = L10n.Text(key), Padding = new Thickness(8, 4, 8, 4) };
        button.Click += (_, _) => action();
        return button;
    }
    private void Render()
    {
        tree.Children.Clear(); tree.Children.Add(Group(draft));
        UpdateSummary(); RenderPalette();
    }
    private void UpdateSummary()
    {
        var names = tags.ToDictionary(tag => tag.id, tag => tag.name);
        expression.Text = draft.IsEmpty ? "" : draft.Expression(names);
        var missing = loaded && draft.TagIds.Any(id => !names.ContainsKey(id));
        validation.Text = missing ? L10n.Text("筛选标签已删除，请替换或移除")
            : !draft.IsEmpty && !draft.IsComplete ? L10n.Text("请为每个分组添加条件") : "";
        apply.IsEnabled = draft.IsEmpty || loaded && draft.IsComplete && !missing;
        heading.Text = L10n.Text(replacing is null ? "添加到当前分组" : "替换标签");
    }
    private FrameworkElement Group(TagFilter node)
    {
        var body = new StackPanel { Spacing = 8 };
        var and = new RadioButton { Content = "AND", IsChecked = !node.UsesOr, MinWidth = 0 };
        var or = new RadioButton { Content = "OR", IsChecked = node.UsesOr, MinWidth = 0 };
        var name = Guid.NewGuid().ToString(); and.GroupName = name; or.GroupName = name;
        and.Checked += (_, _) => { node.UsesOr = false; UpdateSummary(); };
        or.Checked += (_, _) => { node.UsesOr = true; UpdateSummary(); };
        var header = Views.Row(and, or, Negation(node), TextButton("＋条件", () => { target = node; replacing = null; Render(); }),
            TextButton("＋分组", () => { var child = new TagFilter(); node.Children.Add(child); target = child; replacing = null; Render(); }));
        body.Children.Add(header);
        foreach (var child in node.Children)
        {
            var remove = Views.Button("trash", "移除", () =>
            {
                node.Children.Remove(child); target = node; replacing = null; Render();
            });
            var row = new Grid { ColumnSpacing = 8 };
            row.ColumnDefinitions.Add(new()); row.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
            if (child.TagId is { } id)
            {
                row.Children.Add(Chip(id, () => { replacing = child; target = node; Render(); }));
                var actions = Views.Row(Negation(child), remove); Grid.SetColumn(actions, 1); row.Children.Add(actions);
            }
            else
            {
                row.Margin = new Thickness(12, 0, 0, 0);
                row.Children.Add(Group(child)); Grid.SetColumn(remove, 1); row.Children.Add(remove);
                remove.VerticalAlignment = VerticalAlignment.Top;
            }
            body.Children.Add(row);
        }
        var border = new Border { Padding = new Thickness(8), CornerRadius = new CornerRadius(8), BorderThickness = new Thickness(1), Child = body };
        void Theme() => border.BorderBrush = Views.Brush(ReferenceEquals(target, node) ? "SystemControlForegroundAccentBrush" : "ControlStrokeColorDefaultBrush");
        border.ActualThemeChanged += (_, _) => Theme(); Theme();
        return border;
    }
    private CheckBox Negation(TagFilter node)
    {
        var check = new CheckBox { Content = "NOT", IsChecked = node.Negated, MinWidth = 0 };
        check.Checked += (_, _) => { node.Negated = true; UpdateSummary(); };
        check.Unchecked += (_, _) => { node.Negated = false; UpdateSummary(); };
        ToolTipService.SetToolTip(check, L10n.Text(node.TagId is null ? "整组取反" : "排除"));
        AutomationProperties.SetName(check, node.TagId is null ? L10n.Text("整组取反")
            : L10n.Text("排除") + " " + (tags.FirstOrDefault(tag => tag.id == node.TagId)?.name ?? L10n.Text("已删除标签")));
        return check;
    }
    private Button Chip(string id, Action action)
    {
        var tag = tags.FirstOrDefault(tag => tag.id == id);
        var content = Views.Row(new Border { Width = 6, Height = 6, CornerRadius = new CornerRadius(3),
            Background = tag is null ? Views.Brush("TextFillColorSecondaryBrush") : new SolidColorBrush(TagManager.Color(tag.color)), VerticalAlignment = VerticalAlignment.Center },
            Views.Text(tag?.name ?? (loaded ? L10n.Text("已删除标签") : "…"), 12));
        var button = new Button { Content = content, CornerRadius = new CornerRadius(14), Padding = new Thickness(8, 4, 8, 4), HorizontalAlignment = HorizontalAlignment.Left };
        if (tag is not null) button.Background = new SolidColorBrush(TagManager.Color(tag.color)) { Opacity = .14 };
        button.Click += (_, _) => action(); AutomationProperties.SetName(button, tag?.name ?? (loaded ? L10n.Text("已删除标签") : "…"));
        return button;
    }
    private void RenderPalette()
    {
        palette.Children.Clear();
        foreach (var tag in tags.Where(tag => tag.name.Contains(search.Text, StringComparison.CurrentCultureIgnoreCase)))
        {
            palette.Children.Add(Chip(tag.id, () =>
            {
                if (replacing is { } condition) { condition.TagId = tag.id; replacing = null; }
                else target.Children.Add(new() { TagId = tag.id });
                Render();
            }));
        }
        if (loaded && palette.Children.Count == 0) palette.Children.Add(Views.LocalizedText("无匹配标签", 12));
    }
}
