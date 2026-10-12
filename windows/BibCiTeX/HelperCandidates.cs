using BibCiTeX.Core;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Windows.Foundation;
using Windows.System;

namespace BibCiTeX;

/// <summary>Filters type, field and tag names in the search row while showing only candidate rows in an independent popup.</summary>
internal sealed class HelperCandidates
{
    /// <summary>The inline editor searches catalog names; its optional second stage can add a selected field constraint.</summary>
    internal Border InlineEditor { get; } = new() { Visibility = Visibility.Collapsed };
    private readonly TextBlock marker = new() { FontSize = 16, VerticalAlignment = VerticalAlignment.Center };
    private readonly Border editorBorder = new() { CornerRadius = new CornerRadius(6), BorderThickness = new Thickness(1) };
    private readonly SearchTextBox filter = new() { MinWidth = 110, MaxWidth = 220, FontSize = 16, BorderThickness = new Thickness(0), Padding = new Thickness(8, 5, 8, 5) };
    private readonly ListView items = new() { IsItemClickEnabled = true, Padding = new Thickness(0), HorizontalContentAlignment = HorizontalAlignment.Stretch };
    private readonly Popup popup = new() { IsLightDismissEnabled = false, ShouldConstrainToRootBounds = false,
        DesiredPlacement = PopupPlacementMode.BottomEdgeAlignedLeft, VerticalOffset = 6 };
    private readonly TextBox search;
    private readonly HelperInputGuard input;
    private readonly TextBox optionalValue = new() { MinWidth = 130, MaxWidth = 220, FontSize = 16, BorderThickness = new Thickness(0), PlaceholderText = L10n.Text("字段值（可选）") };
    private readonly HelperInputGuard valueInput;
    private SearchFieldDescriptor? optionalField, previousField;
    private readonly Func<SearchFieldDescriptor?> currentField;
    private readonly Func<string, Task<QueryAnalysis>> analyze;
    private readonly Action<QueryAnalysis> commit;
    private readonly Action<SearchFieldDescriptor?> selectField;
    private readonly Action changed;
    private readonly Action<bool> visibility;
    private readonly Func<Rect> workArea;
    private readonly Func<bool> tailComposing;
    private sealed record Candidate(string Name, string Label, string EnglishLabel, string? Id, SearchFieldDescriptor? Field);
    private List<Candidate> entries = [];
    private Task<QueryAnalysis>? original;
    private string before = "";
    private char trigger;
    private int generation, filterVersion;
    private ToolTip? valueError;
    private bool active, updating, busy;
    /// <summary>Suspends ordinary helper search and result actions during catalog selection.</summary>
    internal bool Open => active;
    /// <summary>Whether the inline catalog filter currently owns native marked text.</summary>
    internal bool IsComposing => input.IsComposing || valueInput.IsComposing;
    /// <summary>Invalidates helper-wide deferred focus work when filter composition begins.</summary>
    internal int InputEpoch => input.Epoch + valueInput.Epoch;
    /// <summary>Creates the inline catalog selector without adding layout space to the result list.</summary>
    internal HelperCandidates(TextBox search, Action changed, Action<bool> visibility,
        Func<string, Task<QueryAnalysis>> analyze, Action<QueryAnalysis> commit,
        Action<SearchFieldDescriptor?> selectField, Func<Rect> workArea, Func<bool> tailComposing, Func<SearchFieldDescriptor?> currentField)
    {
        this.search = search; this.changed = changed; this.visibility = visibility; this.analyze = analyze;
        this.commit = commit; this.selectField = selectField; this.workArea = workArea; this.tailComposing = tailComposing; this.currentField = currentField;
        Localized.Name(filter, "筛选"); Localized.Name(items, "候选");
        editorBorder.BorderBrush = Views.Brush("SystemControlForegroundAccentBrush"); editorBorder.Child = filter;
        InlineEditor.Child = Views.Row(marker, editorBorder);
        popup.PlacementTarget = editorBorder;
        popup.Child = new Border { Child = items, Width = 320, CornerRadius = new CornerRadius(9), BorderThickness = new Thickness(1),
            Background = Views.Brush("SolidBackgroundFillColorBaseBrush"), BorderBrush = Views.Brush("SurfaceStrokeColorDefaultBrush") };
        ScrollViewer.SetVerticalScrollMode(items, ScrollMode.Enabled);
        ScrollViewer.SetVerticalScrollBarVisibility(items, ScrollBarVisibility.Hidden);
        ScrollViewer.SetHorizontalScrollMode(items, ScrollMode.Disabled);
        ScrollViewer.SetHorizontalScrollBarVisibility(items, ScrollBarVisibility.Disabled);
        input = new HelperInputGuard(filter); valueInput = new HelperInputGuard(optionalValue);
        Localized.Name(optionalValue, "字段值（可选）");
        optionalValue.TextChanged += (_, _) => { filterVersion++; if (valueError is not null) valueError.IsOpen = false; };
        valueInput.Started += () => { if (valueError is not null) valueError.IsOpen = false; };
        optionalValue.PreviewKeyDown += async (_, args) =>
        {
            if (args.Handled || valueInput.IsComposing) return;
            if (valueInput.Consumes(args.Key)) { args.Handled = true; return; }
            if (args.Key is VirtualKey.Enter or VirtualKey.Escape or VirtualKey.Tab) { args.Handled = true; await Handle(args.Key); }
        };
        input.Ended += () => { if (Open) Populate(); };
        InlineEditor.SizeChanged += (_, _) => { if (Open && optionalField is null && !IsComposing && !tailComposing()) ShowPopup(); };
        InlineEditor.Loaded += (_, _) => { if (Open && optionalField is null) ShowPopup(); };
        filter.TextChanged += (_, _) => { filterVersion++; if (!input.IsComposing) Populate(); };
        filter.PreviewKeyDown += async (_, args) =>
        {
            if (args.Handled || input.IsComposing) return;
            if (input.Consumes(args.Key)) { args.Handled = true; return; }
            if (args.Key is VirtualKey.Enter or VirtualKey.Escape or VirtualKey.Up or VirtualKey.Down or VirtualKey.Tab)
            { args.Handled = true; await Handle(args.Key); }
        };
        items.ItemClick += async (_, args) => { items.SelectedItem = args.ClickedItem; await Confirm(); };
        items.SelectionChanged += (_, _) => { if (items.SelectedItem is { } selected) items.ScrollIntoView(selected); };
    }
    /// <summary>Recognizes a single standalone typed trigger at the end of unquoted, unescaped input.</summary>
    internal static bool IsTrigger(string previous, string text, int caret)
    {
        if (text.Length != previous.Length + 1 || caret != text.Length || !text.StartsWith(previous, StringComparison.Ordinal) || text[^1] is not ('@' or '$' or '#')) return false;
        if (previous.Length > 0 && !char.IsWhiteSpace(previous[^1])) return false;
        bool quoted = false, escaped = false;
        foreach (char c in previous) { if (escaped) { escaped = false; continue; } if (c == '\\') escaped = true; else if (c == '"') quoted = !quoted; }
        return !quoted && !escaped;
    }
    /// <summary>Moves the typed trigger into the inline name filter and opens candidates without altering confirmed query nodes.</summary>
    internal async Task Begin(string previous, char symbol)
    {
        if (tailComposing()) return;
        int request = ++generation;
        active = true; busy = false; optionalField = null; previousField = currentField(); editorBorder.Child = filter; before = previous; trigger = symbol; entries = []; items.Items.Clear();
        original = analyze(previous);
        marker.Text = symbol.ToString(); filter.Text = "";
        updating = true; try { search.Text = before; } finally { updating = false; }
        InlineEditor.Visibility = Visibility.Visible; visibility(true); ShowPopup();
        // Focus is transferred only at this explicit trigger boundary, never by a late catalog response.
        if (!tailComposing() && !input.IsComposing)
        {
            if (filter.Focus(FocusState.Programmatic)) { filter.Select(filter.Text.Length, 0); filter.StartBringIntoView(); }
            else
            {
                int epoch = input.Epoch, textVersion = filterVersion;
                filter.DispatcherQueue.TryEnqueue(() =>
                {
                    if (!Open || request != generation || epoch != input.Epoch || textVersion != filterVersion || input.IsComposing || tailComposing() || filter.Text.Length != 0) return;
                    filter.Focus(FocusState.Programmatic); filter.Select(0, 0); filter.StartBringIntoView(); ShowPopup();
                });
            }
        }
        try
        {
            var catalog = await RustCore.SearchCatalog(); if (!Open || request != generation) return;
            if (symbol == '#')
            {
                var tags = await RustCore.TagSnapshot(new() { path = "", cite_key = "" }); if (!Open || request != generation) return;
                entries = tags.tags.Select(x => new Candidate(x.name, x.name, x.name, x.id, null)).ToList();
            }
            else if (symbol == '@') entries = catalog.type_descriptors.Select(x => new Candidate(x.name, L10n.Text(x.label_key), L10n.Translate(x.label_key, "en"), null, null)).ToList();
            else entries = catalog.field_descriptors.Select(x => new Candidate(x.name, L10n.Text(x.label_key), L10n.Translate(x.label_key, "en"), null, x)).ToList();
            if (!input.IsComposing) Populate();
        }
        catch (Exception error) { if (Open && request == generation) ToolTipService.SetToolTip(filter, L10n.ErrorMessage(error)); }
    }
    private void Populate()
    {
        if (input.IsComposing) return;
        items.Items.Clear();
        foreach (var entry in entries.Where(x => x.Name.Contains(filter.Text, StringComparison.CurrentCultureIgnoreCase) || x.Label.Contains(filter.Text, StringComparison.CurrentCultureIgnoreCase) || (L10n.Language == "zh-Hans" && x.EnglishLabel.Contains(filter.Text, StringComparison.OrdinalIgnoreCase))))
            items.Items.Add(new ListViewItem { Content = new TextBlock { Text = entry.Label, FontSize = 13, TextTrimming = TextTrimming.CharacterEllipsis }, Tag = entry, Height = 32, MinHeight = 32, Padding = new Thickness(16, 4, 8, 4) });
        if (items.Items.Count > 0) items.SelectedIndex = 0;
        else items.Items.Add(new ListViewItem { Content = L10n.Text("未找到匹配记录"), IsEnabled = false, Height = 32, MinHeight = 32, Padding = new Thickness(16, 4, 8, 4) });
        if (Open && optionalField is null && !IsComposing && !tailComposing()) ShowPopup();
    }
    private void ShowPopup()
    {
        if (!Open || optionalField is not null || IsComposing || tailComposing() || editorBorder.ActualWidth <= 0 || search.XamlRoot is null) return;
        popup.XamlRoot = search.XamlRoot;
        var area = workArea();
        double desired = 140;
        foreach (var item in items.Items.OfType<ListViewItem>())
            if (item.Tag is Candidate candidate) { var label = new TextBlock { Text = candidate.Label, FontSize = 13 }; label.Measure(new Size(double.PositiveInfinity, 32)); desired = Math.Max(desired, label.DesiredSize.Width + 40); }
        double width = Math.Min(Math.Clamp(desired, 140, 220), Math.Max(120, area.Width - 16));
        double height = Math.Min(Math.Clamp(items.Items.Count, 1, 6) * 32 + 14, Math.Max(40, area.Height - 16));
        if (popup.Child is Border border) { border.Width = width; border.Padding = new Thickness(6); }
        // Give the native list a finite viewport so wheel/trackpad input can scroll it.
        items.Height = Math.Max(32, height - 14);
        popup.IsOpen = true;
    }
    /// <summary>Yields immediately when native composition starts in the main tail.</summary>
    internal void BeginMainComposition() { if (Open) Close(false); }
    /// <summary>Closes catalog selection when the user returns to editing the main tail.</summary>
    internal void MainTextChanged() { if (Open && !updating && search.Text != before) Close(false); }
    /// <summary>Closes the floating selector; cancellation restores only its original tail, never replacing a bound prefix.</summary>
    internal void Close(bool restore)
    {
        if (valueError is not null) valueError.IsOpen = false;
        generation++; active = false; busy = false; popup.IsOpen = false; InlineEditor.Visibility = Visibility.Collapsed; visibility(false);
        if (restore && !tailComposing() && !input.IsComposing)
        { updating = true; try { search.Text = before; } finally { updating = false; } changed(); search.Focus(FocusState.Programmatic); search.Select(search.Text.Length, 0); }
    }
    /// <summary>Consumes catalog navigation before helper paste shortcuts and leaves IME keystrokes to the native editor.</summary>
    internal async Task<bool> Handle(VirtualKey key)
    {
        if (!Open || IsComposing || tailComposing()) return false;
        if (input.Consumes(key) || valueInput.Consumes(key)) return true;
        if (key == VirtualKey.Escape)
        {
            if (optionalField is { } field) { Close(false); selectField(field); }
            else Close(true);
            return true;
        }
        if (key == VirtualKey.Tab) { if (optionalField is { } field) { Close(false); selectField(field); } else filter.Focus(FocusState.Programmatic); return true; }
        if (key == VirtualKey.Enter) { await Confirm(); return true; }
        if (optionalField is null && key is VirtualKey.Up or VirtualKey.Down)
        { if (items.Items.Count > 0) items.SelectedIndex = (items.SelectedIndex + (key == VirtualKey.Down ? 1 : -1) + items.Items.Count) % items.Items.Count; return true; }
        return false;
    }
    private async Task Confirm()
    {
        if (busy || IsComposing || tailComposing()) return;
        if (optionalField is not null) { await ConfirmOptional(); return; }
        if ((items.SelectedItem as ListViewItem)?.Tag is not Candidate selected) return;
        int request = generation, epoch = input.Epoch, textVersion = filterVersion; busy = true;
        try
        {
            if (selected.Field is { } field)
            {
                optionalField = field; popup.IsOpen = false; selectField(field);
                optionalValue.Text = "";
                var valueRow = Views.Row(new TextBlock { Text = L10n.Text(field.label_key), VerticalAlignment = VerticalAlignment.Center }, optionalValue);
                valueRow.Padding = new Thickness(5, 0, 5, 0); editorBorder.Child = valueRow;
                if (!IsComposing && !tailComposing())
                {
                    if (optionalValue.Focus(FocusState.Programmatic)) { optionalValue.Select(0, 0); optionalValue.StartBringIntoView(); }
                    else
                    {
                        int focusEpoch = InputEpoch;
                        optionalValue.DispatcherQueue.TryEnqueue(() =>
                        {
                            if (!Open || request != generation || focusEpoch != InputEpoch || IsComposing || tailComposing() || optionalField is null || optionalValue.Text.Length != 0) return;
                            optionalValue.Focus(FocusState.Programmatic); optionalValue.Select(0, 0); optionalValue.StartBringIntoView();
                        });
                    }
                }
                return;
            }
            var operand = await RustCore.BuildCondition(new() { field_name = trigger == '@' ? "type" : "tag", text_value = selected.Name, tag_id = selected.Id, phrase = false, exclude = false });
            var saved = original is { } task ? await task : await analyze(before);
            if (!Open || request != generation || epoch != input.Epoch || textVersion != filterVersion || input.IsComposing || tailComposing()) return;
            if (!operand.status.IsValid || !saved.status.IsValid || saved.document is null) { ToolTipService.SetToolTip(filter, SearchSession.Diagnostic(!operand.status.IsValid ? operand : saved)); return; }
            var result = await RustCore.EditQuery(new() { document = saved.document, action = QueryEditAction.AppendAnd, target = null, operand = operand.document });
            result.tag_revision ??= operand.tag_revision ?? saved.tag_revision;
            if (result.document is { } document && document.nodes.Any(x => x.tag_id is not null)) result = await RustCore.RefreshQuery(document);
            if (!Open || request != generation || epoch != input.Epoch || textVersion != filterVersion || input.IsComposing || tailComposing()) return;
            if (!result.status.IsValid) { ToolTipService.SetToolTip(filter, SearchSession.Diagnostic(result)); return; }
            Close(false); commit(result);
        }
        catch (Exception error) { if (Open && request == generation) ToolTipService.SetToolTip(filter, L10n.ErrorMessage(error)); }
        finally { if (request == generation) busy = false; }
    }
    private void ShowValueError(string message)
    {
        if (valueError is not null) valueError.IsOpen = false;
        valueError = new ToolTip { Content = message, PlacementTarget = optionalValue };
        ToolTipService.SetToolTip(optionalValue, valueError); valueError.IsOpen = true;
    }
    private async Task ConfirmOptional()
    {
        if (optionalField is not { } field || busy || IsComposing || tailComposing()) return;
        if (string.IsNullOrWhiteSpace(optionalValue.Text)) { Close(false); selectField(field); return; }
        int request = generation, epoch = InputEpoch, textVersion = filterVersion; busy = true;
        try
        {
            var condition = await RustCore.BuildCondition(new() { field_name = field.name, text_value = optionalValue.Text, phrase = false, exclude = false, tag_id = null });
            var basis = original is { } task ? await task : await analyze(before);
            if (!Open || request != generation || epoch != InputEpoch || textVersion != filterVersion || IsComposing || tailComposing()) return;
            if (!condition.status.IsValid || !basis.status.IsValid || basis.document is null)
            { ShowValueError(SearchSession.Diagnostic(!condition.status.IsValid ? condition : basis)); return; }
            var result = await RustCore.EditQuery(new() { document = basis.document, action = QueryEditAction.AppendAnd, target = null, operand = condition.document });
            result.tag_revision ??= basis.tag_revision;
            if (result.document is { } document && document.nodes.Any(x => x.tag_id is not null)) result = await RustCore.RefreshQuery(document);
            if (!Open || request != generation || epoch != InputEpoch || textVersion != filterVersion || IsComposing || tailComposing()) return;
            if (!result.status.IsValid) { ShowValueError(SearchSession.Diagnostic(result)); return; }
            var restoreField = previousField;
            Close(false); selectField(restoreField); commit(result);
        }
        catch (Exception error) { if (Open && request == generation) ShowValueError(L10n.ErrorMessage(error)); }
        finally { if (request == generation) busy = false; }
    }

}
