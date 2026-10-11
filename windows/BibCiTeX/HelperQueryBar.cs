using BibCiTeX.Core;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Windows.Foundation;

namespace BibCiTeX;

/// <summary>Projects one ordered query document as editable capsules followed by its uncommitted text tail.</summary>
internal sealed class HelperQueryBar
{
    private readonly TextBox tail;
    private readonly SearchSession session;
    private readonly Action changed;
    private readonly Action<Exception> report;
    private readonly Action<bool, bool> editing;
    private readonly Func<bool> composing;
    private readonly Func<int> inputEpoch;
    private int focusEpoch;
    private readonly StackPanel row = new() { Orientation = Orientation.Horizontal, Spacing = 6, VerticalAlignment = VerticalAlignment.Center };
    private readonly StackPanel chips = new() { Orientation = Orientation.Horizontal, Spacing = 5, VerticalAlignment = VerticalAlignment.Center };
    private readonly ContentControl inline = new() { VerticalAlignment = VerticalAlignment.Center };
    private QueryAnalysis? prefix;
    private SearchFieldDescriptor? selectedField;
    private readonly Action requestField;
    private int generation;
    private bool changing;
    private bool deletingPrevious;
    private int pendingBackspaces;
    /// <summary>The ID-bound capsule document saved independently of the trailing text while choosing a library.</summary>
    internal QueryAnalysis? Prefix => prefix;
    /// <summary>The current keyword scope is separate from the unscoped query document.</summary>
    internal SearchFieldDescriptor? Field => selectedField;
    /// <summary>Whether the confirmed query owns a nonempty tree.</summary>
    internal bool HasPrefix => prefix?.document?.root is not null;
    /// <summary>The horizontally scrolling search surface; no query condition is executed outside its composed document.</summary>
    internal ScrollViewer View { get; }
    /// <summary>Suppresses reactions to an atomic visual projection change.</summary>
    internal bool Updating { get; private set; }
    /// <summary>Creates a native query projection. The existing search box becomes only its trailing text editor.</summary>
    internal HelperQueryBar(TextBox tail, SearchSession session, Action changed, Action<Exception> report, Action<bool, bool> editing, Func<bool> composing, Func<int> inputEpoch, Action requestField)
    {
        this.tail = tail; this.session = session; this.changed = changed; this.report = report; this.editing = editing; this.composing = composing; this.inputEpoch = inputEpoch; this.requestField = requestField;
        tail.MinWidth = 140;
        row.Children.Add(chips); row.Children.Add(tail); row.Children.Add(inline);
        View = new ScrollViewer { Content = row, HorizontalScrollMode = ScrollMode.Enabled, VerticalScrollMode = ScrollMode.Disabled,
            HorizontalScrollBarVisibility = ScrollBarVisibility.Hidden, VerticalScrollBarVisibility = ScrollBarVisibility.Disabled, ZoomMode = ZoomMode.Disabled };
        row.SizeChanged += (_, _) => { if (pendingTailReveal) RevealTail(); };
    }
    private bool pendingTailReveal;
    /// <summary>Places the temporary field-value editor after existing text, preserving the visible query order.</summary>
    internal void SetInlineEditor(UIElement editor) => inline.Content = editor;
    /// <summary>Resolves the visible prefix and supplied tail once, retaining their original AND operand order and all tag IDs.</summary>
    internal async Task<QueryAnalysis> AnalyzeBase(string text)
    {
        var saved = prefix;
        if (saved is null) return await RustCore.AnalyzeQuery(new() { text = text });
        if (saved.document is { } boundDocument && boundDocument.nodes.Any(x => x.tag_id is not null))
        {
            var refreshed = await RustCore.RefreshQuery(boundDocument);
            if (!refreshed.status.IsValid) return refreshed;
            if (ReferenceEquals(prefix, saved)) { prefix = refreshed; if (saved.text != refreshed.text && !composing()) Render(); }
            saved = refreshed;
        }
        if (text.Trim().Length == 0) return saved;
        var operand = await RustCore.AnalyzeQuery(new() { text = text });
        if (!operand.status.IsValid || operand.document is null) return operand;
        if (!saved.status.IsValid || saved.document is null) return saved;
        var combined = await RustCore.EditQuery(new() { document = saved.document, action = QueryEditAction.AppendAnd, target = null, operand = operand.document });
        combined.tag_revision ??= saved.tag_revision;
        return combined;
    }
    /// <summary>Applies the selected UI scope once around the base tree; explicit type, tag and handwritten fields retain their own scopes.</summary>
    internal async Task<QueryAnalysis> Analyze(string text)
    {
        var field = selectedField;
        var basis = await AnalyzeBase(text);
        if (field is null || !basis.status.IsValid || basis.document?.root is not { } root) return basis;
        var nodes = basis.document.nodes.ToList();
        nodes.Add(new QueryNodeRecord { kind = QueryNodeKind.Scope, text_value = field.name, children = [root], tag_id = null, start = 0, end = 0 });
        var scoped = await RustCore.RenderQuery(new() { nodes = nodes, root = (uint)(nodes.Count - 1) });
        scoped.tag_revision ??= basis.tag_revision;
        return scoped;
    }
    /// <summary>Replaces the current keyword scope without modifying the base document or accumulating nested UI scopes.</summary>
    internal void SelectField(SearchFieldDescriptor? field)
    {
        if (composing()) return;
        selectedField = field; Render(); FocusTail(); changed();
    }
    /// <summary>Confirms the complete document, clears only its text tail and moves focus after the final capsule.</summary>
    internal void Accept(QueryAnalysis query)
    {
        if (!query.status.IsValid || composing()) return;
        prefix = query.document?.root is null ? null : query;
        session.Accept(query);
        Updating = true;
        try { tail.Text = ""; }
        finally { Updating = false; }
        Render(); FocusTail(); changed();
    }
    /// <summary>Restores a helper draft without serializing its ID-bound prefix through names.</summary>
    internal void Restore(QueryAnalysis? query, string text = "", SearchFieldDescriptor? field = null)
    {
        if (composing()) return;
        prefix = query; selectedField = field;
        Updating = true;
        try { tail.Text = text; }
        finally { Updating = false; }
        Render(); FocusTail();
    }
    /// <summary>Resets a new helper session and drops both the capsule projection and its trailing text.</summary>
    internal void Reset() { prefix = null; selectedField = null; generation++; Updating = true; try { tail.Text = ""; } finally { Updating = false; } chips.Children.Clear(); }
    /// <summary>Refreshes a matching bound projection after tag renames, without changing an actively edited text tail.</summary>
    internal void RefreshProjection(QueryAnalysis? query)
    {
        if (selectedField is not null || composing() || prefix is null || query is null || tail.Text.Length != 0 || ReferenceEquals(prefix, query)) return;
        prefix = query; Render();
    }
    /// <summary>Moves the caret only at explicit confirmation/cancellation boundaries and reveals the trailing input after overflow.</summary>
    internal void FocusTail()
    {
        if (composing()) return;
        int focus = ++focusEpoch, epoch = inputEpoch();
        tail.Focus(FocusState.Programmatic); tail.Select(tail.Text.Length, 0); pendingTailReveal = true;
        tail.DispatcherQueue.TryEnqueue(() => { if (composing() || focus != focusEpoch || epoch != inputEpoch() || tail.FocusState == FocusState.Unfocused) return; tail.StartBringIntoView(new BringIntoViewOptions { AnimationDesired = false, HorizontalAlignmentRatio = 1 }); RevealTail(); });
    }
    /// <summary>Cancels queued focus and scrolling before native marked text begins.</summary>
    internal void BeginComposition() { focusEpoch++; pendingTailReveal = false; }
    private void RevealTail()
    {
        if (composing()) { pendingTailReveal = false; return; }
        if (View.ViewportWidth <= 0) return;
        View.ChangeView(View.ScrollableWidth, null, null, disableAnimation: true);
        pendingTailReveal = false;
    }
    /// <summary>Allows only repeated Backspace to join an in-flight capsule deletion, not unrelated busy operations.</summary>
    internal bool DeletingPrevious => deletingPrevious;
    /// <summary>Whether an unmodified Backspace at the native tail boundary can remove a preceding capsule.</summary>
    internal bool CanDeletePrevious => !composing() && (!changing || deletingPrevious) && (HasPrefix || selectedField is not null);
    /// <summary>Removes preceding capsules in visual order. Repeated key presses queue; native text, selections and IME always take priority.</summary>
    internal async Task DeletePrevious()
    {
        if (!CanDeletePrevious || tail.SelectionStart != 0 || tail.SelectionLength != 0 || tail.FocusState == FocusState.Unfocused) return;
        pendingBackspaces++;
        if (deletingPrevious) return;
        deletingPrevious = changing = true;
        editing(true, false);
        try
        {
            while (pendingBackspaces > 0 && !composing() && tail.SelectionStart == 0 && tail.SelectionLength == 0 && tail.FocusState != FocusState.Unfocused)
            {
                pendingBackspaces--;
                if (prefix?.document is { } document && document.root is { } root)
                {
                    var top = document.nodes[(int)root];
                    uint target = top.kind.IsAnd ? top.children[^1] : root;
                    int request = generation, epoch = inputEpoch(); string text = tail.Text;
                    var updated = await RustCore.EditQuery(new() { document = document, action = QueryEditAction.Remove, target = target, operand = null });
                    if (request != generation || epoch != inputEpoch() || composing() || tail.Text != text || tail.SelectionStart != 0 || tail.SelectionLength != 0 || tail.FocusState == FocusState.Unfocused) break;
                    if (!updated.status.IsValid) { report(new InvalidOperationException(SearchSession.Diagnostic(updated))); break; }
                    updated.tag_revision ??= prefix?.tag_revision;
                    prefix = updated.document?.root is null ? null : updated;
                }
                else if (selectedField is not null) selectedField = null;
                else break;
                Render();
                // Keep a nonempty tail at its beginning so the next Backspace
                // removes the next capsule instead of a character at its end.
                tail.Focus(FocusState.Programmatic); tail.Select(0, 0);
                tail.StartBringIntoView(new BringIntoViewOptions { AnimationDesired = false });
            }
        }
        catch (Exception error) { report(error); }
        finally { pendingBackspaces = 0; deletingPrevious = changing = false; editing(false, false); }
    }
    private void Render()
    {
        int request = ++generation;
        chips.Children.Clear();
        if (selectedField is { } field)
        {
            var choose = new Button { Content = L10n.Text(field.label_key), Padding = new Thickness(8, 3, 3, 3), BorderThickness = new Thickness(0) };
            choose.Click += (_, _) => { if (!composing()) requestField(); };
            var clear = Views.Button("x", "移除", () => { if (composing()) return; selectedField = null; Render(); FocusTail(); changed(); });
            clear.Padding = new Thickness(3); clear.MinWidth = 24; clear.MinHeight = 24;
            chips.Children.Add(Views.Capsule(Views.Row(choose, clear)));
        }
        if (prefix?.document is not { } document || document.root is not { } root) return;
        var node = document.nodes[(int)root];
        // Only expose top-level AND children: OR and NOT subtrees remain intact capsules.
        var ids = node.kind.IsAnd ? node.children : new List<uint> { root };
        foreach (uint id in ids)
        {
            var label = new TextBlock { Text = Label(document, id), MaxWidth = 220, TextTrimming = TextTrimming.CharacterEllipsis, VerticalAlignment = VerticalAlignment.Center };
            var edit = new Button { Content = label, Padding = new Thickness(9, 3, 3, 3), BorderThickness = new Thickness(0), Background = new Microsoft.UI.Xaml.Media.SolidColorBrush(Microsoft.UI.Colors.Transparent) };
            var remove = Views.Button("x", "移除", async () =>
            {
                if (changing || composing()) return; changing = true;
                int epoch = inputEpoch();
                editing(true, false);
                try
                {
                    var updated = await RustCore.EditQuery(new() { document = document, action = QueryEditAction.Remove, target = id, operand = null });
                    if (request == generation && !updated.status.IsValid) report(new InvalidOperationException(SearchSession.Diagnostic(updated)));
                    if (request == generation && epoch == inputEpoch() && !composing() && updated.status.IsValid) { updated.tag_revision ??= prefix?.tag_revision; prefix = updated.document?.root is null ? null : updated; Render(); FocusTail(); }
                }
                catch (Exception ex) { if (request == generation) report(ex); }
                finally { changing = false; editing(false, false); }
            });
            remove.Padding = new Thickness(3); remove.MinWidth = 24; remove.MinHeight = 24;
            edit.Click += async (_, _) =>
            {
                if (changing || composing()) return; changing = true;
                int epoch = inputEpoch();
                editing(true, true);
                try
                {
                    var updated = await AdvancedSearchEditor.Show(tail.XamlRoot, prefix?.text ?? "", prefix, selectedNode: id);
                    if (request == generation && epoch == inputEpoch() && !composing() && updated is not null) { updated.tag_revision ??= prefix?.tag_revision; prefix = updated; Render(); FocusTail(); }
                }
                catch (Exception ex) { if (request == generation) report(ex); }
                finally { changing = false; editing(false, false); }
            };
            ToolTipService.SetToolTip(edit, Label(document, id));
            chips.Children.Add(Views.Capsule(Views.Row(edit, remove)));
        }
    }
    private static string Label(QueryDocumentRecord document, uint id)
    {
        var node = document.nodes[(int)id];
        if (node.kind.IsWord) return node.text_value;
        if (node.kind.IsPhrase) return "\"" + node.text_value + "\"";
        if (node.kind.IsScope) return node.text_value + ":" + Label(document, node.children[0]);
        if (node.kind.IsNot) return "−(" + Label(document, node.children[0]) + ")";
        return "(" + string.Join(node.kind.IsOr ? " OR " : " ", node.children.Select(child => Label(document, child))) + ")";
    }
}
