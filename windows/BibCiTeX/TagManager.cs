using BibCiTeX.Core;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Media;
using Windows.UI;

namespace BibCiTeX;

/// <summary>Compact UID-based assignment picker and global tag catalog.</summary>
internal static class TagManager
{
    internal static event Action? Changed;
    private static readonly HashSet<XamlRoot> EditingRoots = [];
    internal static bool IsEditing(XamlRoot root) => EditingRoots.Contains(root);
    private static readonly object BatchControlsMarker = new();
    // The persistent selection toolbar must not consume Escape from the list or query.
    internal static bool HasBlockingPopup(XamlRoot root) => VisualTreeHelper.GetOpenPopupsForXamlRoot(root)
        .Any(popup => popup.Child is not FrameworkElement child || !ReferenceEquals(child.Tag, BatchControlsMarker));
    private static readonly string[] Palette = ["#64748B", "#EF4444", "#F59E0B", "#22C55E", "#14B8A6", "#3B82F6", "#8B5CF6", "#EC4899"];
    internal static Color Color(string value) => Windows.UI.Color.FromArgb(255, Convert.ToByte(value.Substring(1, 2), 16), Convert.ToByte(value.Substring(3, 2), 16), Convert.ToByte(value.Substring(5, 2), 16));

    internal static FrameworkElement Chips(Reference reference)
    {
        var row = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 4 };
        foreach (var tag in reference.Data.tags.Take(2))
        {
            var button = new Button { Height = 20, MinHeight = 0, MinWidth = 0, Padding = new Thickness(6, 2, 6, 2), CornerRadius = new CornerRadius(10), MaxWidth = 64,
                Background = new SolidColorBrush(Color(tag.color)) { Opacity = .14 } };
            var content = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 4 };
            content.Children.Add(new Border { Width = 6, Height = 6, CornerRadius = new CornerRadius(3), Background = new SolidColorBrush(Color(tag.color)), VerticalAlignment = VerticalAlignment.Center });
            content.Children.Add(new TextBlock { Text = tag.name, FontSize = 10, TextTrimming = TextTrimming.CharacterEllipsis, MaxWidth = 40 });
            button.Content = content; ToolTipService.SetToolTip(button, tag.name); AutomationProperties.SetName(button, tag.name);
            button.Click += async (_, _) => { try { await Show(button.XamlRoot, reference.Data.library_uid, reference.Id, reference.Key, button); } catch (Exception ex) { await Views.Error(button, ex); } };
            row.Children.Add(button);
        }
        if (reference.Data.tags.Count > 2)
        {
            var overflow = new Button { Height = 20, MinHeight = 0, MinWidth = 0, Content = "+" + (reference.Data.tags.Count - 2), FontSize = 10, Padding = new Thickness(4, 2, 4, 2) };
            ToolTipService.SetToolTip(overflow, string.Join(", ", reference.Data.tags.Select(t => t.name)));
            Localized.Name(overflow, "标签");
            overflow.Click += async (_, _) => { try { await Show(overflow.XamlRoot, reference.Data.library_uid, reference.Id, reference.Key, overflow); } catch (Exception ex) { await Views.Error(overflow, ex); } };
            row.Children.Add(overflow);
        }
        return row;
    }

    internal static FrameworkElement DetailChips(Reference reference)
    {
        var panel = new DetailTagPanel();
        foreach (var tag in reference.Data.tags)
        {
            var capsule = new Border { Height = 26, Padding = new Thickness(8, 0, 8, 0), CornerRadius = new CornerRadius(13),
                Background = new SolidColorBrush(Color(tag.color)) { Opacity = .14 } };
            var content = new Grid { ColumnSpacing = 5 };
            content.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
            content.ColumnDefinitions.Add(new() { Width = new GridLength(1, GridUnitType.Star) });
            content.Children.Add(new Border { Width = 6, Height = 6, CornerRadius = new CornerRadius(3), Background = new SolidColorBrush(Color(tag.color)), VerticalAlignment = VerticalAlignment.Center });
            var label = new TextBlock { Text = tag.name, FontSize = 12, TextWrapping = TextWrapping.NoWrap, TextTrimming = TextTrimming.CharacterEllipsis, VerticalAlignment = VerticalAlignment.Center };
            Grid.SetColumn(label, 1); content.Children.Add(label);
            capsule.Child = content; ToolTipService.SetToolTip(capsule, tag.name);
            panel.Children.Add(capsule);
        }
        var add = new Button { Content = new SvgIcon("add", 12), Height = 26, MinHeight = 0, MinWidth = 0, Padding = new Thickness(8, 0, 8, 0), CornerRadius = new CornerRadius(13), BorderThickness = new Thickness(0) };
        Localized.Name(add, "添加"); Localized.Tooltip(add, "添加");
        add.Click += async (_, _) =>
        {
            try { await Show(add.XamlRoot, reference.Data.library_uid, reference.Id, reference.Key, add); }
            catch (Exception ex) { await Views.Error(add, ex); }
        };
        panel.Children.Add(add);
        return panel;
    }

    private sealed class DetailTagPanel : Panel
    {
        private const double Spacing = 6;

        protected override Windows.Foundation.Size MeasureOverride(Windows.Foundation.Size availableSize)
        {
            foreach (var child in Children)
            {
                child.Measure(new Windows.Foundation.Size(double.PositiveInfinity, double.PositiveInfinity));
                if (child.DesiredSize.Width > availableSize.Width)
                    child.Measure(new Windows.Foundation.Size(availableSize.Width, double.PositiveInfinity));
            }
            return Layout(availableSize.Width, arrange: false);
        }

        protected override Windows.Foundation.Size ArrangeOverride(Windows.Foundation.Size finalSize)
        {
            Layout(finalSize.Width, arrange: true);
            return finalSize;
        }

        private Windows.Foundation.Size Layout(double width, bool arrange)
        {
            double x = 0, y = 0, rowHeight = 0, contentWidth = 0;
            foreach (var child in Children)
            {
                var size = child.DesiredSize;
                if (x > 0 && x + size.Width > width) { x = 0; y += rowHeight + Spacing; rowHeight = 0; }
                if (arrange) child.Arrange(new Windows.Foundation.Rect(x, y, Math.Min(size.Width, width), size.Height));
                contentWidth = Math.Max(contentWidth, x + size.Width);
                x += size.Width + Spacing; rowHeight = Math.Max(rowHeight, size.Height);
            }
            return new Windows.Foundation.Size(double.IsFinite(width) ? width : contentWidth, y + rowHeight);
        }
    }

    internal static Button BatchControls(ListView references, out Func<bool> endSelection, Action? opening = null, Action? closed = null, bool nativeMultiple = false, Func<IReadOnlyList<Reference>, Task<bool>>? cite = null)
    {
        var button = new Button
        {
            Padding = nativeMultiple ? new Thickness(8) : new Thickness(6, 2, 6, 2),
            MinWidth = 32, MinHeight = 32,
        };
        var label = new TextBlock { VerticalAlignment = VerticalAlignment.Center };
        button.Content = Views.Row(new SvgIcon("checklist"), label);
        Button ActionButton(string icon, string title)
        {
            var action = new Button { Content = Views.Row(new SvgIcon(icon, 16), Views.LocalizedText(title, 12)), Padding = new Thickness(8, 6, 8, 6), CornerRadius = new CornerRadius(16) };
            Localized.Name(action, title); Localized.Tooltip(action, title);
            return action;
        }
        var mode = ActionButton("x", "结束");
        var all = ActionButton("check", "全选");
        var clear = ActionButton("trash", "清空");
        var edit = ActionButton("tag", "批量");
        var actions = Views.Row(all, clear, edit);
        var citation = cite is null ? null : Views.CopyButton("copy", "引用", () => cite(Selected()), showLabel: true);
        if (citation is not null) actions.Children.Add(citation);
        actions.Children.Add(Views.Divider(vertical: true)); actions.Children.Add(mode);
        actions.Spacing = 4;
        var surface = Views.Capsule(actions);
        surface.Tag = BatchControlsMarker;
        var popup = new Popup { Child = surface, PlacementTarget = button, DesiredPlacement = PopupPlacementMode.Bottom, IsLightDismissEnabled = false };
        bool explicitSelection = false;
        bool editing = false;
        XamlRoot? observedRoot = null;
        bool Active() => nativeMultiple ? explicitSelection || references.SelectedItems.Count > 1 : references.SelectionMode == ListViewSelectionMode.Multiple;
        List<Reference> Selected() => references.Items.OfType<ListViewItem>().Where(references.SelectedItems.Contains).Select(item => item.Tag).OfType<Reference>().DistinctBy(reference => reference.Id).ToList();
        void Render()
        {
            if (nativeMultiple && references.SelectedItems.Count > 1) explicitSelection = true;
            label.Visibility = !nativeMultiple || Active() ? Visibility.Visible : Visibility.Collapsed;
            label.Text = Active() ? $"{L10n.Text("已选")} {Selected().Count}" : L10n.Text("多选");
            ToolTipService.SetToolTip(button, label.Text); AutomationProperties.SetName(button, label.Text);
            all.IsEnabled = Active() && references.Items.OfType<ListViewItem>().Any(item => item.Tag is Reference);
            clear.IsEnabled = edit.IsEnabled = Active() && Selected().Count > 0;
            if (citation is not null) citation.IsEnabled = Active() && Selected().Count > 0;
            surface.RequestedTheme = button.ActualTheme;
            var show = Active() && !editing && button.IsLoaded && button.XamlRoot?.IsHostVisible == true;
            if (show) popup.XamlRoot = button.XamlRoot;
            popup.IsOpen = show;
        }
        bool EndSelection()
        {
            if (!Active()) return false;
            var focused = references.SelectedItem;
            explicitSelection = false;
            if (!nativeMultiple) references.SelectionMode = ListViewSelectionMode.Single;
            references.SelectedItems.Clear();
            references.SelectedItem = focused;
            Render();
            return true;
        }
        endSelection = EndSelection;
        mode.Click += (_, _) => EndSelection();
        all.Click += (_, _) => { foreach (var item in references.Items.OfType<ListViewItem>().Where(item => item.Tag is Reference)) if (!references.SelectedItems.Contains(item)) references.SelectedItems.Add(item); };
        clear.Click += (_, _) => references.SelectedItems.Clear();
        edit.Click += async (_, _) => { var selected = Selected(); if (selected.Count == 0) return; editing = true; Render(); try { opening?.Invoke(); await ShowBatch(button.XamlRoot, selected); } catch (Exception ex) { await Views.Error(button, ex); } finally { editing = false; closed?.Invoke(); Render(); } };
        references.SelectionChanged += (_, _) => Render();
        void RootChanged(XamlRoot _, XamlRootChangedEventArgs args) => Render();
        button.Loaded += (_, _) => { observedRoot = button.XamlRoot; if (observedRoot != null) observedRoot.Changed += RootChanged; Render(); };
        button.Unloaded += (_, _) => { popup.IsOpen = false; if (observedRoot != null) observedRoot.Changed -= RootChanged; observedRoot = null; };
        button.ActualThemeChanged += (_, _) => Render();
        surface.KeyDown += (_, args) => {
            if (args.Key != Windows.System.VirtualKey.Escape || HasBlockingPopup(button.XamlRoot)) return;
            if (new[] { Windows.System.VirtualKey.Control, Windows.System.VirtualKey.Menu, Windows.System.VirtualKey.Shift, Windows.System.VirtualKey.LeftWindows, Windows.System.VirtualKey.RightWindows }.Any(key =>
                Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(key).HasFlag(Windows.UI.Core.CoreVirtualKeyStates.Down))) return;
            if (EndSelection()) { references.Focus(FocusState.Programmatic); args.Handled = true; }
        };
        button.Click += (_, _) => {
            if (Active()) EndSelection();
            else if (references.Items.OfType<ListViewItem>().Any(item => item.Tag is Reference)) {
                explicitSelection = true;
                references.SelectionMode = nativeMultiple ? ListViewSelectionMode.Extended : ListViewSelectionMode.Multiple;
                references.SelectedItems.Clear(); Render();
            }
        };
        Render();
        return button;
    }

    private static async Task ShowBatch(XamlRoot root, List<Reference> references)
    {
        var targets = references.Select(reference => new TagTarget { library_uid = reference.Data.library_uid, record_uid = reference.Id }).ToList();
        if (targets.Count == 0) return;
        var request = new BatchTagRequest { targets = targets };
        var state = await RustCore.BatchTagSnapshot(request);
        var panel = new StackPanel { Spacing = 8, Width = 430 };
        var controls = new ContentControl { Content = panel, HorizontalContentAlignment = HorizontalAlignment.Stretch };
        var filter = new TextBox { Header = L10n.Text("搜索或创建标签") };
        var list = new StackPanel { Spacing = 6 };
        var error = new TextBlock { TextWrapping = TextWrapping.Wrap };
        var create = new Button { Content = L10n.Text("创建并添加") };
        var busy = false; var changed = false; var valid = true;
        async Task Mutate(TagAction action, string id = "", string name = "")
        {
            if (busy || !valid) return;
            busy = true; controls.IsEnabled = false;
            try {
                state = await RustCore.MutateBatchTag(new() { targets = targets, action = action, id = id, name = name, expected_revision = state.revision });
                changed = true; Changed?.Invoke(); error.Text = "";
                if (action.IsCreate) filter.Text = "";
            } catch (Exception ex) {
                error.Text = L10n.ErrorMessage(ex);
                try { state = await RustCore.BatchTagSnapshot(request); } catch { valid = false; }
            } finally { busy = false; controls.IsEnabled = true; Render(); }
        }
        void Render()
        {
            list.Children.Clear(); create.IsEnabled = valid && !string.IsNullOrWhiteSpace(filter.Text);
            foreach (var tag in state.tags.Where(tag => tag.name.Contains(filter.Text, StringComparison.CurrentCultureIgnoreCase))) {
                bool all = state.assigned_ids.Contains(tag.id), some = state.partial_ids.Contains(tag.id);
                var add = new Button { Content = L10n.Text("添加"), IsEnabled = valid && !all };
                var remove = new Button { Content = L10n.Text("移除"), IsEnabled = valid && (all || some) };
                add.Click += async (_, _) => await Mutate(TagAction.Assign, tag.id);
                remove.Click += async (_, _) => await Mutate(TagAction.Unassign, tag.id);
                var row = new Grid { ColumnSpacing = 6 };
                row.ColumnDefinitions.Add(new()); row.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
                var label = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 6 };
                label.Children.Add(new Border { Width = 12, Height = 12, CornerRadius = new CornerRadius(6), Background = new SolidColorBrush(Color(tag.color)) });
                label.Children.Add(new TextBlock { Text = tag.name, MaxWidth = 140, TextTrimming = TextTrimming.CharacterEllipsis, VerticalAlignment = VerticalAlignment.Center });
                ToolTipService.SetToolTip(label, tag.name);
                var actions = Views.Row(new TextBlock { Text = L10n.Text(all ? "全部已添加" : some ? "部分已添加" : "未添加"), FontSize = 11, VerticalAlignment = VerticalAlignment.Center }, add, remove);
                row.Children.Add(label); Grid.SetColumn(actions, 1); row.Children.Add(actions); list.Children.Add(row);
            }
        }
        filter.TextChanged += (_, _) => Render(); create.Click += async (_, _) => await Mutate(TagAction.Create, name: filter.Text);
        panel.Children.Add(filter); panel.Children.Add(new ScrollViewer { Content = list, MaxHeight = 240 }); panel.Children.Add(create); panel.Children.Add(error); Render();
        var dialog = new ContentDialog { XamlRoot = root, Title = $"{L10n.Text("批量标签")} · {L10n.Text("已选")} {state.count}", Content = controls, CloseButtonText = L10n.Text("关闭") };
        dialog.Closing += (_, args) => { if (busy) args.Cancel = true; };
        EditingRoots.Add(root);
        try { await dialog.ShowAsync(); } finally { EditingRoots.Remove(root); if (changed) Changed?.Invoke(); }
    }

    internal static async Task Show(XamlRoot root, string libraryUid = "", string recordUid = "", string citeKey = "", FrameworkElement? anchor = null)
    {
        var request = new TagSnapshotRequest { library_uid = libraryUid, record_uid = recordUid, path = "", cite_key = "" };
        var state = await RustCore.TagSnapshot(request);
        if (anchor is not null && anchor.XamlRoot is null) return;
        var assignment = recordUid.Length > 0;
        var panel = new StackPanel { Spacing = 8, Width = assignment ? 330 : 430 };
        var controls = new ContentControl { Content = panel, HorizontalContentAlignment = HorizontalAlignment.Stretch };
        var filter = new TextBox { Header = L10n.Text("搜索或创建标签") };
        var list = new StackPanel { Spacing = 4 };
        var error = new TextBlock { TextWrapping = TextWrapping.Wrap };
        var create = new Button { Content = L10n.Text(assignment ? "创建并添加" : "创建") };
        var confirmation = new StackPanel { Spacing = 6, Visibility = Visibility.Collapsed };
        Expander? recoveryEntry = null;
        bool busy = false;
        bool changed = false;
        async Task Mutate(TagAction action, string id = "", string name = "", string? color = null, string target = "", string? targetLibrary = null, string? targetRecord = null)
        {
            if (busy) return;
            busy = true; controls.IsEnabled = false;
            try
            {
                state = await RustCore.MutateTag(new() { action = action, id = id, name = name, color = color, target_id = target,
                    library_uid = targetLibrary ?? libraryUid, record_uid = targetRecord ?? recordUid,
                    path = "", cite_key = "", expected_revision = state.revision });
                changed = true;
                Changed?.Invoke();
                if (action.IsCreate) filter.Text = "";
                if (!assignment) state = await RustCore.TagSnapshot(request);
                error.Text = ""; confirmation.Visibility = Visibility.Collapsed; Render();
            }
            catch (Exception ex)
            {
                error.Text = L10n.ErrorMessage(ex);
                try { state = await RustCore.TagSnapshot(request); if (!action.IsRename) Render(); } catch { }
            }
            finally { busy = false; controls.IsEnabled = true; }
        }
        void Confirm(string name, Func<Task> action)
        {
            confirmation.Children.Clear(); confirmation.Visibility = Visibility.Visible;
            confirmation.Children.Add(new TextBlock { Text = name, TextWrapping = TextWrapping.Wrap });
            var accept = new Button { Content = L10n.Text("确认") };
            accept.Click += async (_, _) => await action();
            var cancel = new Button { Content = L10n.Text("取消") };
            cancel.Click += (_, _) => confirmation.Visibility = Visibility.Collapsed;
            confirmation.Children.Add(Views.Row(accept, cancel));
        }
        void Render()
        {
            if (recoveryEntry is not null) recoveryEntry.Visibility = state.pending.Count > 0 ? Visibility.Visible : Visibility.Collapsed;
            list.Children.Clear();
            foreach (var tag in state.tags.Where(t => t.name.Contains(filter.Text, StringComparison.CurrentCultureIgnoreCase)).OrderByDescending(t => assignment && state.assigned_ids.Contains(t.id)).ThenBy(t => t.name, StringComparer.CurrentCultureIgnoreCase))
            {
                var row = new Grid { ColumnSpacing = 6 };
                row.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
                row.ColumnDefinitions.Add(new()); row.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
                var colorButton = new Button { Width = 24, Height = 24, MinWidth = 0, MinHeight = 0, Padding = new Thickness(4), Content = new Border { Background = new SolidColorBrush(Color(tag.color)), CornerRadius = new CornerRadius(7), Width = 14, Height = 14 } };
                Localized.Name(colorButton, "标签颜色");
                var colors = new StackPanel { Spacing = 6, Width = 220 };
                var swatches = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 3 };
                var colorFlyout = new Flyout { Content = colors };
                foreach (var value in Palette)
                {
                    var swatch = new Button { Width = 24, Height = 24, MinWidth = 0, MinHeight = 0, Padding = new Thickness(3), Background = new SolidColorBrush(Color(value)) };
                    Localized.Name(swatch, value); swatch.Click += async (_, _) => { colorFlyout.Hide(); await Mutate(TagAction.Recolor, tag.id, color: value); }; swatches.Children.Add(swatch);
                }
                var custom = new ColorPicker { Color = Color(tag.color), IsAlphaEnabled = false, IsColorSliderVisible = false, IsColorChannelTextInputVisible = true, Width = 220 };
                var apply = new Button { Content = L10n.Text("保存") };
                apply.Click += async (_, _) => { var value = custom.Color; colorFlyout.Hide(); await Mutate(TagAction.Recolor, tag.id, color: $"#{value.R:X2}{value.G:X2}{value.B:X2}"); };
                colors.Children.Add(swatches); colors.Children.Add(new TextBlock { Text = L10n.Text("自定义颜色") }); colors.Children.Add(custom); colors.Children.Add(apply); colorButton.Flyout = colorFlyout; row.Children.Add(colorButton);
                FrameworkElement label;
                if (assignment)
                {
                    var check = new CheckBox { Content = tag.name, IsChecked = state.assigned_ids.Contains(tag.id), MinWidth = 0 };
                    check.Click += async (_, _) => await Mutate(check.IsChecked == true ? TagAction.Assign : TagAction.Unassign, tag.id); label = check;
                }
                else label = new TextBlock { Text = tag.name, VerticalAlignment = VerticalAlignment.Center, TextTrimming = TextTrimming.CharacterEllipsis };
                Grid.SetColumn(label, 1); row.Children.Add(label);
                var more = new Button { Content = new SvgIcon("more", 14), Padding = new Thickness(4), MinWidth = 24 };
                Localized.Name(more, "标签管理"); var menu = new MenuFlyout();
                var rename = new MenuFlyoutItem { Text = L10n.Text("重命名") };
                rename.Click += (_, _) => {
                    var name = new TextBox { Text = tag.name, Header = L10n.Text("标签名称") };
                    Grid.SetColumn(name, 1); row.Children.Remove(label); row.Children.Add(name);
                    var save = new Button { Content = L10n.Text("保存") }; Grid.SetColumn(save, 2); row.Children.Remove(more); row.Children.Add(save);
                    save.Click += async (_, _) => await Mutate(TagAction.Rename, tag.id, name.Text);
                    name.KeyDown += async (_, e) => { if (e.Key == Windows.System.VirtualKey.Enter) { e.Handled = true; await Mutate(TagAction.Rename, tag.id, name.Text); } else if (e.Key == Windows.System.VirtualKey.Escape) { e.Handled = true; Render(); } };
                    name.Focus(FocusState.Programmatic); name.SelectAll();
                }; menu.Items.Add(rename);
                if (!assignment)
                {
                    var merge = new MenuFlyoutSubItem { Text = L10n.Text("合并到") };
                    foreach (var other in state.tags.Where(t => t.id != tag.id))
                    { var item = new MenuFlyoutItem { Text = other.name }; item.Click += (_, _) => Confirm(tag.name + " → " + other.name, () => Mutate(TagAction.Merge, tag.id, target: other.id)); merge.Items.Add(item); }
                    menu.Items.Add(merge);
                    var delete = new MenuFlyoutItem { Text = L10n.Text("删除") }; delete.Click += (_, _) => Confirm(L10n.Text("删除标签") + " “" + tag.name + "”", () => Mutate(TagAction.Delete, tag.id)); menu.Items.Add(delete);
                }
                more.Flyout = menu; Grid.SetColumn(more, 2); row.Children.Add(more); list.Children.Add(row);
            }
            create.IsEnabled = filter.Text.Trim().Length > 0 && !state.tags.Any(t => string.Equals(t.name, filter.Text.Trim(), StringComparison.CurrentCultureIgnoreCase));
        }
        filter.TextChanged += (_, _) => Render();
        create.Click += async (_, _) => await Mutate(TagAction.Create, name: filter.Text);
        filter.KeyDown += async (_, e) => { if (e.Key == Windows.System.VirtualKey.Enter && create.IsEnabled) { e.Handled = true; await Mutate(TagAction.Create, name: filter.Text); } };
        panel.Children.Add(filter); panel.Children.Add(new ScrollViewer { Content = list, MaxHeight = assignment ? 240 : 320 }); panel.Children.Add(create); panel.Children.Add(confirmation); panel.Children.Add(error);
        if (!assignment && state.pending.Count > 0)
        {
            var recovery = new StackPanel { Spacing = 6 };
            var pending = new ComboBox { Header = L10n.Text("旧记录"), HorizontalAlignment = HorizontalAlignment.Stretch };
            foreach (var item in state.pending) pending.Items.Add(new ComboBoxItem { Content = item.cite_key, Tag = item.record_id });
            var libraries = await RustCore.Libraries();
            var library = new ComboBox { Header = L10n.Text("目标文献库"), HorizontalAlignment = HorizontalAlignment.Stretch };
            foreach (var item in libraries.Where(l => l.Available).DistinctBy(l => l.Uid)) library.Items.Add(item);
            var record = new ComboBox { Header = L10n.Text("目标记录"), HorizontalAlignment = HorizontalAlignment.Stretch };
            int recoveryGeneration = 0;
            library.SelectionChanged += async (_, _) => {
                var generation = ++recoveryGeneration;
                record.Items.Clear();
                if (library.SelectedItem is not Library selected) return;
                try {
                    var rows = await RustCore.Search(selected.Path, "");
                    var fresh = await RustCore.TagSnapshot(request);
                    if (generation != recoveryGeneration) return;
                    foreach (var item in rows) record.Items.Add(new ComboBoxItem { Content = item.Key, Tag = item });
                    state = fresh; Render();
                } catch (Exception ex) { if (generation == recoveryGeneration) error.Text = L10n.ErrorMessage(ex); }
            };
            var relink = new Button { Content = L10n.Text("重新关联") };
            relink.Click += async (_, _) => { if ((pending.SelectedItem as ComboBoxItem)?.Tag is string id && library.SelectedItem is Library selected && (record.SelectedItem as ComboBoxItem)?.Tag is Reference destination) { await Mutate(TagAction.Relink, id, targetLibrary: selected.Uid, targetRecord: destination.Id); pending.Items.Clear(); foreach (var item in state.pending) pending.Items.Add(new ComboBoxItem { Content = item.cite_key, Tag = item.record_id }); } };
            recovery.Children.Add(pending); recovery.Children.Add(library); recovery.Children.Add(record); recovery.Children.Add(relink);
            recoveryEntry = new Expander { Header = L10n.Text("失效标签关联"), Content = recovery, HorizontalAlignment = HorizontalAlignment.Stretch };
            panel.Children.Add(recoveryEntry);
        }
        Render();
        if (assignment)
        {
            var flyout = new Flyout { Content = controls }; var completion = new TaskCompletionSource();
            flyout.Closed += (_, _) => completion.TrySetResult();
            EditingRoots.Add(root);
            try { flyout.ShowAt(anchor ?? (FrameworkElement)root.Content); await completion.Task; }
            finally { EditingRoots.Remove(root); if (changed) Changed?.Invoke(); }
        }
        else await Views.ShowDialog(new ContentDialog { XamlRoot = root, Title = L10n.Text("标签管理"), Content = new ScrollViewer { Content = controls, MaxHeight = 560 }, CloseButtonText = L10n.Text("关闭") });
    }
}
