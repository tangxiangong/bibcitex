using BibCiTeX.Core;

namespace BibCiTeX;

// Interoptopus generates the C# ABI and binary wire codecs. Only thread scheduling
// and view-model adaptation live here. Every returned wire buffer is disposed
// on its worker thread after Unwire has produced managed, owned DTOs.
internal static class RustCore
{
    private static void Check(string? error)
    {
        if (error is not null) throw new LocalizedException(error);
    }
    internal static Task Initialize() => Task.Run(() =>
    {
        using var wire = Interop.initialize().AsOk();
        Check(wire.Unwire().error);
    });
    internal static Task<List<Library>> Libraries() => Task.Run(() =>
    {
        using var wire = Interop.libraries().AsOk();
        var response = wire.Unwire(); Check(response.error);
        return response.payload.Select(Library.From).ToList();
    });
    internal static Task<List<Reference>> Search(string path, string query, string field = "all", string type = "all") => Task.Run(() =>
    {
        using var request = new SearchRequest { path = path, query = query, search_field = field, type_filter = type }.Wire();
        using var wire = Interop.search(request).AsOk();
        var response = wire.Unwire(); Check(response.error);
        return response.payload.Select(x => new Reference(x)).ToList();
    });
    /// <summary>Executes an explicit-mode request against one immutable library snapshot. Bound tag nodes require their expected revision; invalid or incomplete queries return diagnostics without actionable references.</summary>
    /// <returns>Owned managed data. Transport and storage failures throw a localized exception.</returns>
    internal static Task<AdvancedSearchResponse> AdvancedSearch(AdvancedSearchRequest value) => Task.Run(() =>
    {
        using var request = value.Wire(); using var wire = Interop.advanced_search(request).AsOk();
        var result = wire.Unwire(); Check(result.error); return result.payload ?? throw new InvalidOperationException("Missing search response");
    });
    /// <summary>Parses advanced query text in the shared Rust grammar and returns its status, source spans and editable tree. An incomplete query is data, not a storage exception.</summary>
    /// <returns>Owned managed data. Transport and storage failures throw a localized exception.</returns>
    internal static Task<QueryAnalysis> AnalyzeQuery(TextRequest value) => Task.Run(() =>
    {
        using var request = value.Wire(); using var wire = Interop.analyze_query(request).AsOk();
        var result = wire.Unwire(); Check(result.error); return result.payload ?? throw new InvalidOperationException("Missing service response");
    });
    /// <summary>Applies an actual UTF-8 source edit to a bound document, preserving untouched tag identities. Full replacements deliberately resolve names again.</summary>
    /// <returns>Owned managed data. Transport and storage failures throw a localized exception.</returns>
    internal static Task<QueryAnalysis> EditQueryText(QueryTextEdit value) => Task.Run(() =>
    {
        using var request = value.Wire(); using var wire = Interop.edit_query_text(request).AsOk();
        var result = wire.Unwire(); Check(result.error); return result.payload ?? throw new InvalidOperationException("Missing query response");
    });
    /// <summary>Serializes a validated ordered query tree without changing field provenance or tag identity. Invalid trees return diagnostics.</summary>
    /// <returns>Owned managed data. Transport and storage failures throw a localized exception.</returns>
    internal static Task<QueryAnalysis> RenderQuery(QueryDocumentRecord value) => Task.Run(() =>
    {
        using var request = value.Wire(); using var wire = Interop.render_query(request).AsOk();
        var result = wire.Unwire(); Check(result.error); return result.payload ?? throw new InvalidOperationException("Missing service response");
    });
    /// <summary>Applies a semantic operation to the requested node, or the root when target is absent. Append and replace operations require an operand.</summary>
    /// <returns>Owned managed data. Transport and storage failures throw a localized exception.</returns>
    internal static Task<QueryAnalysis> EditQuery(QueryEdit value) => Task.Run(() =>
    {
        using var request = value.Wire(); using var wire = Interop.edit_query(request).AsOk();
        var result = wire.Unwire(); Check(result.error); return result.payload ?? throw new InvalidOperationException("Missing service response");
    });
    /// <summary>Constructs and validates one typed condition. A tag ID binds a catalog identity; ordinary text and phrase matching remain distinct.</summary>
    /// <returns>Owned managed data. Transport and storage failures throw a localized exception.</returns>
    internal static Task<QueryAnalysis> BuildCondition(QueryCondition value) => Task.Run(() =>
    {
        using var request = value.Wire(); using var wire = Interop.build_condition(request).AsOk();
        var result = wire.Unwire(); Check(result.error); return result.payload ?? throw new InvalidOperationException("Missing service response");
    });
    /// <summary>Refreshes tag names and merge redirects while retaining bound identities. Deleted bindings return diagnostics and must be removed or reselected.</summary>
    /// <returns>Owned managed data. Transport and storage failures throw a localized exception.</returns>
    internal static Task<QueryAnalysis> RefreshQuery(QueryDocumentRecord value) => Task.Run(() =>
    {
        using var request = value.Wire(); using var wire = Interop.refresh_query(request).AsOk();
        var result = wire.Unwire(); Check(result.error); return result.payload ?? throw new InvalidOperationException("Missing service response");
    });
    /// <summary>Reads the global tag catalog and pending records; a path and exact cite key additionally request the selected reference associations.</summary>
    /// <returns>Owned managed data. Transport and storage failures throw a localized exception.</returns>
    internal static Task<TagSnapshot> TagSnapshot(TagSnapshotRequest value) => Task.Run(() =>
    {
        using var request = value.Wire(); using var wire = Interop.tag_snapshot(request).AsOk();
        var result = wire.Unwire(); Check(result.error); return result.payload ?? throw new InvalidOperationException("Missing service response");
    });
    /// <summary>Applies one metadata mutation only at its expected revision. Conflicts and storage failures throw and never silently overwrite newer metadata.</summary>
    /// <returns>Owned managed data. Transport and storage failures throw a localized exception.</returns>
    internal static Task<BatchTagSnapshot> BatchTagSnapshot(BatchTagRequest value) => Task.Run(() =>
    {
        using var request = value.Wire(); using var wire = Interop.batch_tag_snapshot(request).AsOk();
        var result = wire.Unwire(); Check(result.error); return result.payload ?? throw new InvalidOperationException("Missing service response");
    });
    internal static Task<BatchTagSnapshot> MutateBatchTag(BatchTagMutation value) => Task.Run(() =>
    {
        using var request = value.Wire(); using var wire = Interop.mutate_batch_tag(request).AsOk();
        var result = wire.Unwire(); Check(result.error); return result.payload ?? throw new InvalidOperationException("Missing service response");
    });
    internal static Task<TagSnapshot> MutateTag(TagMutation value) => Task.Run(() =>
    {
        using var request = value.Wire(); using var wire = Interop.mutate_tag(request).AsOk();
        var result = wire.Unwire(); Check(result.error); return result.payload ?? throw new InvalidOperationException("Missing service response");
    });
    /// <summary>Checks that a displayed result still belongs to the current file and tag snapshots before copy, paste or opening an attachment.</summary>
    /// <returns>Owned managed data. Transport and storage failures throw a localized exception.</returns>
    internal static Task<ContextValidity> ValidateSearchContext(SearchContext value) => Task.Run(() =>
    {
        using var request = value.Wire(); using var wire = Interop.validate_search_context(request).AsOk();
        var result = wire.Unwire(); Check(result.error); return result.payload ?? throw new InvalidOperationException("Missing service response");
    });
    /// <summary>Returns the supported field and type names used by native condition editors and helper candidates.</summary>
    /// <returns>Owned managed data. Transport and storage failures throw a localized exception.</returns>
    internal static Task<SearchCatalog> SearchCatalog() => Task.Run(() =>
    {
        using var wire = Interop.search_catalog().AsOk(); var result = wire.Unwire();
        Check(result.error); return result.payload ?? throw new InvalidOperationException("Missing catalog");
    });
    internal static Task AddLibrary(string name, string path, string? description) => Task.Run(() =>
    {
        using var request = new AddLibraryRequest { name = name, path = path, description = description }.Wire();
        using var wire = Interop.add_library(request).AsOk();
        Check(wire.Unwire().error);
    });
    internal static Task<Library> UpdateLibrary(string name, string newName, string? path, string? description) => Task.Run(() =>
    {
        using var request = new UpdateLibraryRequest { name = name, new_name = newName, path = path, description = description }.Wire();
        using var wire = Interop.update_library(request).AsOk();
        var response = wire.Unwire(); Check(response.error);
        return Library.From(response.payload ?? throw new LocalizedException("Bibliography not found"));
    });
    internal static Task SetLibraryPinned(string name, bool pinned) => Task.Run(() =>
    {
        using var request = new PinLibraryRequest { name = name, pinned = pinned }.Wire();
        using var wire = Interop.set_library_pinned(request).AsOk();
        Check(wire.Unwire().error);
    });
    internal static Task RemoveLibrary(string name) => Task.Run(() =>
    {
        using var request = new NameRequest { name = name }.Wire();
        using var wire = Interop.remove_library(request).AsOk();
        Check(wire.Unwire().error);
    });
    internal static Task<Library?> HelperCurrent() => Task.Run(() =>
    {
        using var wire = Interop.helper_current().AsOk();
        var response = wire.Unwire(); Check(response.error);
        return response.payload is { } library ? Library.From(library) : null;
    });
    internal static Task<Library> HelperSelect(string name, string path) => Task.Run(() =>
    {
        using var request = new SelectLibraryRequest { name = name, path = path }.Wire();
        using var wire = Interop.helper_select(request).AsOk();
        var response = wire.Unwire(); Check(response.error);
        return Library.From(response.payload ?? throw new LocalizedException("Missing bibliography response."));
    });
    internal static Task<string?> CapturePasteTarget() => Task.Run(() =>
    {
        using var wire = Interop.capture_paste_target().AsOk();
        var response = wire.Unwire(); Check(response.error);
        return response.application_name;
    });
    internal static Task<string> FormatCitation(IEnumerable<string> keys, string? format = null, string? mode = null)
    {
        // Capture the global preferences with the selection before yielding.
        var value = new CitationRequest { keys = keys.ToList(), format = format ?? CitationPreferences.Format, mode = mode ?? CitationPreferences.Mode };
        return Task.Run(() =>
        {
            using var request = value.Wire();
            using var wire = Interop.format_citation(request).AsOk();
            return wire.Unwire().text;
        });
    }
    internal static Task Copy(string text) => Task.Run(() =>
    {
        using var request = new TextRequest { text = text }.Wire();
        using var wire = Interop.copy(request).AsOk();
        Check(wire.Unwire().error);
    });
    internal static Task Paste(string text) => Task.Run(() =>
    {
        using var request = new TextRequest { text = text }.Wire();
        using var wire = Interop.paste(request).AsOk();
        Check(wire.Unwire().error);
    });
}
internal sealed record Library(string Uid, string Name, string Path, string Description, string CreatedAt, string UpdatedAt, bool Pinned = false, bool Available = true)
{
    internal static Library From(LibraryRecord x) => new(x.uid, x.name, x.path, x.description ?? "", x.created_at, x.updated_at, x.pinned, x.available);
    public override string ToString() => Name;
}
internal sealed record Chunk(string Kind, string Text);
internal sealed class Reference(ReferenceRecord data)
{
    internal ReferenceRecord Data { get; } = data;
    /// <summary>Matched positive fields from the service; never inferred from normalized display text.</summary>
    internal SearchEvidence? Evidence { get; set; }
    internal string Id => Data.id;
    internal string Key => Data.cite_key;
    internal string Type => Data.entry_type;
    internal string Title => Text("title") is { Length: > 0 } title ? title : L10n.Text("暂无标题");
    internal string Authors => Text("author");
    internal string Venue => new[] { "full_journal", "journal", "book_title", "publisher", "school", "institution", "organization", "how_published" }.Select(Text).FirstOrDefault(x => x.Length > 0) ?? "";
    internal string TypeLabel => L10n.Text(Types.FirstOrDefault(x => x.Value.Equals(Type, StringComparison.OrdinalIgnoreCase)).Label ?? (Type.ToLowerInvariant() is "phdthesis" or "mastersthesis" ? "学位论文" : Type));
    internal List<Chunk> Chunks(string key) => ChunkRecords(key).Select(x => new Chunk(x.kind.IsMath ? "math" : x.kind.IsVerbatim ? "verbatim" : "normal", x.text)).ToList();
    private IEnumerable<ChunkRecord> ChunkRecords(string key) => key switch { "title" => Data.title, "note" => Data.note, "abstract_" => Data.abstract_text, "book_title" => Data.book_title, "issue" => Data.issue, _ => [] };
    internal string Text(string key) => key switch
    {
        "cite_key" => Data.cite_key, "source" => Data.source,
        "title" or "note" or "abstract_" or "book_title" or "issue" => string.Concat(ChunkRecords(key).Select(x => x.text)),
        "author" => string.Join(", ", Data.author), "publisher" => string.Join(", ", Data.publisher), "organization" => string.Join(", ", Data.organization),
        "editor" => string.Join(", ", Data.editor.Select(x => string.IsNullOrEmpty(x.role) ? x.name : $"{x.name} ({x.role})")),
        "pages" => Data.pages is { } pages ? $"{pages.start}–{pages.end}" : "",
        "journal" => Data.journal ?? "", "full_journal" => Data.full_journal ?? "", "year" => Data.year?.ToString() ?? "", "volume" => Data.volume?.ToString() ?? "",
        "number" => Data.number ?? "", "doi" => Data.doi ?? "", "mrclass" => Data.mrclass ?? "", "isbn" => Data.isbn ?? "", "series" => Data.series ?? "",
        "url" => Data.url ?? "", "file" => Data.file ?? "", "edition" => Data.edition?.ToString() ?? "", "book_pages" => Data.book_pages ?? "",
        "school" => Data.school ?? "", "address" => Data.address ?? "", "month" => Data.month ?? "", "institution" => Data.institution ?? "",
        "eprint" => Data.eprint ?? "", "archive_prefix" => Data.archive_prefix ?? "", "arxiv_primary_class" => Data.arxiv_primary_class ?? "", "how_published" => Data.how_published ?? "", _ => ""
    };
    internal static (string Label, string Value)[] Types => [("全部类型", "all"), ("期刊论文", "Article"), ("图书", "Book"), ("学位论文", "Thesis"), ("技术报告", "TechReport"), ("其他", "Misc"), ("小册子", "Booklet"), ("书籍章节", "InBook"), ("文集章节", "InCollection"), ("会议论文", "InProceedings")];
}


internal static class CitationPreferences
{
    internal static readonly (string Label, string Value)[] Formats = [
        ("key", "raw"), ("\\cite", "cite"),
        ("\\citep (natbib)", "citep"), ("\\citet (natbib)", "citet"),
        ("\\autocite (biblatex)", "autocite"), ("\\parencite (biblatex)", "parencite"),
        ("\\textcite (biblatex)", "textcite"), ("\\footcite (biblatex)", "footcite"),
        ("@key", "typst"), ("#cite", "typst-cite"), ("#cite (正文引用)", "typst-prose")
    ];
    internal static string Family(string format) => format == "raw" ? "raw" : format.StartsWith("typst", StringComparison.Ordinal) ? "typst" : "latex";
    internal static (string Label, string Value)[] Commands(string family) => Formats.Where(item => item.Value != "raw" && Family(item.Value) == family).ToArray();
    internal static string Format
    {
        get => NativeSettings.Values["citationFormat"] is string value && Formats.Any(item => item.Value == value) ? value : "raw";
        set => NativeSettings.Values["citationFormat"] = value;
    }
    internal static string Mode
    {
        get => NativeSettings.Values["citationMode"] is "individual" ? "individual" : "merged";
        set => NativeSettings.Values["citationMode"] = value;
    }
}
