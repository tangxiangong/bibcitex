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
    internal static Task CapturePasteTarget() => Task.Run(() =>
    {
        using var wire = Interop.capture_paste_target().AsOk();
        Check(wire.Unwire().error);
    });
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
internal sealed record Library(string Name, string Path, string Description, string CreatedAt, string UpdatedAt, bool Pinned = false, bool Available = true)
{
    internal static Library From(LibraryRecord x) => new(x.name, x.path, x.description ?? "", x.created_at, x.updated_at, x.pinned, x.available);
    public override string ToString() => Name;
}
internal sealed record Chunk(string Kind, string Text);
internal sealed class Reference(ReferenceRecord data)
{
    internal ReferenceRecord Data { get; } = data;
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
