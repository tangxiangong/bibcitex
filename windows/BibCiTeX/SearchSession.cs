using Microsoft.UI.Xaml.Controls;
using BibCiTeX.Core;
using Windows.System;
using System.Text;

namespace BibCiTeX;

// Window-local drafts are deliberately never written to preferences.
/// <summary>Owns one window’s independent in-memory mode drafts and typed search context; only the selected mode is persisted.</summary>
internal sealed class SearchSession
{
    private sealed record Draft(string Text, int Field, int Type, QueryAnalysis? Bound = null, Task<QueryAnalysis>? Edit = null, string? EditedText = null);
    private readonly string? preference;
    private readonly Draft[] drafts = [new("", 0, 0), new("", 0, 0)];
    /// <summary>Whether this window currently interprets the query with the advanced grammar.</summary>
    internal bool Advanced { get; private set; }
    /// <summary>The confirmed typed draft, including stable tag bindings and its text projection.</summary>
    internal QueryAnalysis? Bound { get; private set; }
    /// <summary>The last published immutable snapshot; null means external result actions are unavailable.</summary>
    internal SearchContext? Context { get; private set; }
    /// <summary>Installs a confirmed service-owned query and invalidates the previous executable result context.</summary>
    internal void Accept(QueryAnalysis query) { Bound = query; Context = null; pendingEdit = null; pendingText = null; }
    private Task<QueryAnalysis>? pendingEdit;
    private string? pendingText;
    private bool composing;
    private bool candidateInput;
    /// <summary>Tracks actual selection edits and IME composition so unchanged bound tag nodes retain their identities.</summary>
    internal void Attach(TextBox input, bool helper = false)
    {
        int start = 0, length = 0, deletionDirection = 0;
        bool direct = false, fullReplace = false;
        string compositionText = ""; int compositionStart = 0, compositionLength = 0;
        input.TextCompositionStarted += (_, _) => { composing = true; Context = null; compositionText = input.Text; compositionStart = input.SelectionStart; compositionLength = input.SelectionLength; };
        input.TextCompositionEnded += (_, _) =>
        {
            composing = false;
            if (Bound?.document is not { } document) return;
            var count = input.Text.Length - compositionText.Length + compositionLength;
            if (count < 0 || compositionStart + count > input.Text.Length) { Bound = null; return; }
            var previous = pendingText == compositionText ? pendingEdit : null;
            pendingText = input.Text;
            pendingEdit = EditAfter(previous, new() { document = document, text = compositionText,
                start = (uint)Encoding.UTF8.GetByteCount(compositionText[..compositionStart]),
                end = (uint)Encoding.UTF8.GetByteCount(compositionText[..(compositionStart + compositionLength)]),
                replacement = input.Text.Substring(compositionStart, count), full_replace = false });
        };
        input.Paste += (_, _) => { direct = true; start = input.SelectionStart; length = input.SelectionLength; deletionDirection = 0; fullReplace = length == input.Text.Length; };
        input.CuttingToClipboard += (_, _) => { direct = true; start = input.SelectionStart; length = input.SelectionLength; deletionDirection = 0; fullReplace = false; };
        input.PreviewKeyDown += (_, e) =>
        {
            direct = true; fullReplace = false; start = input.SelectionStart; length = input.SelectionLength;
            deletionDirection = length == 0 ? e.Key == VirtualKey.Back ? -1 : e.Key == VirtualKey.Delete ? 1 : 0 : 0;
            if ((Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(VirtualKey.Control) & Windows.UI.Core.CoreVirtualKeyStates.Down) != 0 && e.Key is not (VirtualKey.V or VirtualKey.X or VirtualKey.Back or VirtualKey.Delete)) direct = false;
        };
        input.BeforeTextChanging += (_, e) =>
        {
            if (helper && direct && !fullReplace && !composing && HelperCandidates.IsTrigger(input.Text, e.NewText, e.NewText.Length)) { direct = false; return; }
            if (Restoring || composing || candidateInput || Bound?.document is not { } document || Bound.text == e.NewText) { direct = false; return; }
            if (!direct) { Bound = null; return; }
            direct = false;
            var old = input.Text;
            // Backspace/Delete (including Ctrl-word deletion) identify the side of
            // the caret. The native edit supplies the exact removed UTF-16 count.
            if (deletionDirection != 0 && e.NewText.Length < old.Length)
            {
                length = old.Length - e.NewText.Length;
                if (deletionDirection < 0) start -= length;
            }
            deletionDirection = 0;
            int replacementLength = e.NewText.Length - (old.Length - length);
            // Verify the actual selection edit, never search/diff the text for an ID.
            if (start < 0 || start + length > old.Length || replacementLength < 0 || start + replacementLength > e.NewText.Length ||
                old[..start] != e.NewText[..start] || old[(start + length)..] != e.NewText[(start + replacementLength)..]) { Bound = null; return; }
            var previous = pendingText == old ? pendingEdit : null;
            pendingText = e.NewText;
            pendingEdit = EditAfter(previous, new() { document = document, text = old,
                start = (uint)Encoding.UTF8.GetByteCount(old[..start]), end = (uint)Encoding.UTF8.GetByteCount(old[..(start + length)]),
                replacement = e.NewText.Substring(start, replacementLength), full_replace = fullReplace });
        };
    }
    /// <summary>Suspends binding edits while the helper owns an uncommitted trigger token.</summary>
    internal void CandidateInput(bool active) { candidateInput = active; Context = null; }
    /// <summary>Commits a cancelled helper trigger and its typed suffix as one recorded insertion at the original caret.</summary>
    internal void ContinueCandidateText(string prefix, string text, Task<QueryAnalysis> original)
    {
        if (!text.StartsWith(prefix, StringComparison.Ordinal)) { Bound = null; pendingEdit = null; pendingText = null; return; }
        pendingText = text;
        pendingEdit = Append();
        async Task<QueryAnalysis> Append()
        {
            var query = await original;
            if (query.document is null) return await RustCore.AnalyzeQuery(new() { text = text });
            uint point = (uint)Encoding.UTF8.GetByteCount(prefix);
            return await RustCore.EditQueryText(new() { document = query.document, text = prefix, start = point, end = point, replacement = text[prefix.Length..], full_replace = false });
        }
    }
    private static async Task<QueryAnalysis> EditAfter(Task<QueryAnalysis>? previous, QueryTextEdit edit)
    {
        if (previous is not null)
        {
            var prior = await previous;
            if (prior.document is { } tree) edit.document = tree;
        }
        return await RustCore.EditQueryText(edit);
    }
    /// <summary>Invalidates displayed results and clears bindings only when text is replaced outside a tracked semantic edit.</summary>
    internal void TextEdited(string text) { if (Restoring || composing || candidateInput) return; if (Bound?.text != text && pendingText != text) Bound = null; Context = null; }
    private async Task ResolveEdit(string text)
    {
        if (pendingEdit is { } edit && pendingText == text)
        {
            var edited = await edit;
            if (ReferenceEquals(pendingEdit, edit)) { Bound = edited; pendingEdit = null; pendingText = null; }
        }
    }
    /// <summary>Returns the exact current draft tree, finishing queued native edits before opening a visual editor.</summary>
    internal async Task<QueryAnalysis> AnalyzeDraft(string text)
    {
        await ResolveEdit(text);
        return Bound?.text == text ? Bound : await RustCore.AnalyzeQuery(new() { text = text });
    }
    /// <summary>Executes the current mode and draft, refreshing stale bound tag projections before one retry.</summary>
    internal async Task<AdvancedSearchResponse> Search(string path, string text, string field, string type, int version)
    {
        await ResolveEdit(text);
        var bound = Advanced && Bound?.text == text ? Bound : null;
        if (bound is not null && !bound.status.IsValid) return new() { analysis = bound, references = [], evidence = [], context = null };
        var request = new AdvancedSearchRequest {
            path = path, mode = Advanced ? SearchMode.Advanced : SearchMode.Ordinary,
            input_kind = bound is null ? QueryInputKind.Text : QueryInputKind.Bound,
            text = text, document = bound?.document, expected_tag_revision = bound?.tag_revision,
            default_field = field, type_filter = type, query_version = (ulong)version
        };
        var response = await RustCore.AdvancedSearch(request);
        if (response.analysis.status.IsStaleQuery && bound?.document is { } document)
        {
            var refreshed = await RustCore.RefreshQuery(document);
            if (!refreshed.status.IsValid) return new() { analysis = refreshed, references = [], evidence = [], context = null };
            request.document = refreshed.document; request.text = refreshed.text; request.expected_tag_revision = refreshed.tag_revision;
            response = await RustCore.AdvancedSearch(request);
            if (ReferenceEquals(Bound, bound)) Bound = refreshed;
        }
        return response;
    }
    /// <summary>Refreshes only the visible projection after a bound tag rename without treating it as a user text replacement.</summary>
    internal void SyncProjection(TextBox input)
    {
        if (!Advanced || Bound is null || input.Text == Bound.text) return;
        Restoring = true;
        try { input.Text = Bound.text; }
        finally { Restoring = false; }
    }
    /// <summary>Publishes a result context only after the owner has checked its request and window version.</summary>
    internal void Publish(AdvancedSearchResponse response) => Context = response.context;
    /// <summary>Revalidates the current result snapshot before an external action; absent contexts are not actionable.</summary>
    internal async Task<bool> Validate()
    {
        var context = Context;
        return context is not null && (await RustCore.ValidateSearchContext(context)).valid && ReferenceEquals(Context, context);
    }
    /// <summary>Immediately makes existing results unavailable when any query or library input changes.</summary>
    internal void Invalidate() => Context = null;
    /// <summary>Formats service diagnostic codes and original UTF-8 ranges for the native status area.</summary>
    internal static string Diagnostic(QueryAnalysis analysis) => string.Join(" · ", analysis.diagnostics.Select(x => $"{L10n.Text(x.code)} [{x.start}–{x.end}]"));
    /// <summary>Suppresses search and text-edit reactions during an atomic draft or projection restore.</summary>
    internal bool Restoring { get; private set; }
    /// <summary>Creates a window-local session. A null preference selects the always-advanced helper default.</summary>
    internal SearchSession(string? preference = null)
    {
        this.preference = preference;
        Advanced = preference is null || bool.TryParse(NativeSettings.Values[preference] as string, out var saved) && saved;
    }
    /// <summary>Saves the outgoing draft and restores the other mode without moving the selected library or merging conditions.</summary>
    internal void Switch(bool advanced, TextBox text, ComboBox field, ComboBox type)
    {
        if (Advanced == advanced) return;
        // Persist before changing the visible state, so a storage failure is atomic.
        if (preference is not null) NativeSettings.Values[preference] = advanced;
        drafts[Advanced ? 1 : 0] = new(text.Text, field.SelectedIndex, type.SelectedIndex, Bound, pendingEdit, pendingText);
        Advanced = advanced;
        var next = drafts[advanced ? 1 : 0];
        Bound = next.Bound; pendingEdit = next.Edit; pendingText = next.EditedText; Context = null;
        Restoring = true;
        try { text.Text = next.Text; field.SelectedIndex = next.Field; type.SelectedIndex = next.Type; }
        finally { Restoring = false; }
    }
}
