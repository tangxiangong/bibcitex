using System.Reflection;
using System.Runtime.InteropServices;
using BibCiTeX;
using BibCiTeX.Core;

UpdatePolicyTests.Run();
await UpdateSourceTests.Run();
WindowsRuntimeTests.Run();

if (args.Length != 1) throw new ArgumentException("Pass the built bibcitex_csharp native library path.");
var library = Path.GetFullPath(args[0]);
if (!File.Exists(library)) throw new FileNotFoundException(library);
NativeLibrary.SetDllImportResolver(Assembly.GetExecutingAssembly(), (name, assembly, searchPath) =>
    name == "bibcitex_csharp" ? NativeLibrary.Load(library) : nint.Zero);

var directory = Path.Combine(Path.GetTempPath(), "BibCiTeX-interop-" + Guid.NewGuid());
Directory.CreateDirectory(directory);
TestEnvironment.Isolate(directory);
var bibliography = Path.Combine(directory, "文献 β.bib");
var assertions = 0;
void Assert(bool condition, string name)
{
    if (!condition) throw new InvalidOperationException("FAIL: " + name);
    assertions++;
}
async Task Throws(Func<Task> call, string expected)
{
    try { await call(); }
    catch (InvalidOperationException error) { Assert(!string.IsNullOrWhiteSpace(error.Message) && (error is LocalizedException localized ? localized.Key : error.Message).Contains(expected, StringComparison.OrdinalIgnoreCase), "error preserves message: " + expected); return; }
    throw new InvalidOperationException("Expected error: " + expected);
}

const string contents = """
@article{unicode2026,
  title = {{NASA}$x^2$研究 与希腊字母 β},
  author = {张, 小明 and Doe, Jane},
  journal = {测试期刊}, fjournal = {完整测试期刊},
  year = {2026}, volume = {12}, number = {3}, pages = {101--109},
  note = {说明 $y_1$}, doi = {10.1234/example}, mrclass = {68T01},
  publisher = {出版机构}, isbn = {9780000000001}, series = {测试丛书},
  url = {https://example.org/article}, file = {paper.pdf},
  abstract = {摘要 $z=1$}, edition = {2}, issue = {Spring},
  school = {测试大学}, address = {北京}, booktitle = {合集 $a+b$},
  editor = {Smith, John}, month = {March}, organization = {测试组织},
  institution = {研究机构}, eprint = {2601.00001}, archiveprefix = {arXiv},
  primaryclass = {cs.AI}, howpublished = {Online}
}
@misc{empty,
  title = {No optional metadata}
}
@book{book2025,
  title = {A book}, author = {Doe, Jane}, year = {2025}, publisher = {Publisher}
}
""";
try
{
    await File.WriteAllTextAsync(bibliography, contents);
    var rows = await RustCore.Search(bibliography, "");
    Assert(rows.Count == 3, "all records cross the actual Rust/C# boundary");
    var reference = rows.Single(x => x.Key == "unicode2026");
    Assert(reference.Title.Contains("研究") && reference.Title.Contains("β"), "UTF-8 title");
    Assert(reference.Authors.Contains("张") && reference.Authors.Contains("Jane"), "UTF-8 authors");
    Assert(reference.Chunks("title").Any(x => x.Kind == "math" && x.Text == "x^2"), "math chunk and enum conversion");
    Assert(reference.Chunks("title").Any(x => x.Kind == "normal" && x.Text == "NASA"), "protected title text follows core normalization");
    Assert(reference.Type == "Article" && reference.TypeLabel == L10n.Text("期刊论文"), "entry type");
    var languageBefore = L10n.Selection;
    try
    {
        var titleBefore = reference.Title; var authorsBefore = reference.Authors;
        L10n.Select("en"); Assert(reference.TypeLabel == "Journal article", "existing reference switches to English");
        L10n.Select("zh-Hans"); Assert(reference.TypeLabel == "期刊论文", "existing reference switches to Chinese");
        Assert(reference.Title == titleBefore && reference.Authors == authorsBefore && reference.Key == "unicode2026", "language switching preserves bibliography data");
    }
    finally { L10n.Select(languageBefore); }
    Assert(reference.Id.Length > 0, "stable reference identity");
    Assert(reference.Text("pages") == "101–109", "inclusive page endpoint");
    Assert(reference.Text("source").Contains("unicode2026") && reference.Text("source").Contains("研究"), "full BibTeX source");
    var expected = new Dictionary<string, string>
    {
        ["journal"] = "测试期刊", ["full_journal"] = "完整测试期刊", ["year"] = "2026",
        ["volume"] = "12", ["number"] = "3", ["doi"] = "10.1234/example", ["mrclass"] = "68T01",
        ["publisher"] = "出版机构", ["isbn"] = "9780000000001", ["series"] = "测试丛书",
        ["url"] = "https://example.org/article", ["file"] = "paper.pdf", ["edition"] = "2",
        ["issue"] = "Spring", ["school"] = "测试大学", ["address"] = "北京", ["month"] = "March",
        ["organization"] = "测试组织", ["institution"] = "研究机构", ["eprint"] = "2601.00001",
        ["archive_prefix"] = "arXiv", ["arxiv_primary_class"] = "cs.AI", ["how_published"] = "Online"
    };
    foreach (var (key, value) in expected) Assert(reference.Text(key) == value, "metadata field " + key + ": " + reference.Text(key));
    Assert(reference.Text("editor").Contains("John") && reference.Text("editor").Contains("Smith"), "editor names and roles");
    Assert(reference.Text("book_pages").Length > 0, "raw pages metadata");
    foreach (var field in new[] { "note", "abstract_", "book_title" }) Assert(reference.Chunks(field).Any(x => x.Kind == "math"), "math in " + field);
    var empty = rows.Single(x => x.Key == "empty");
    Assert(empty.Data.year is null && empty.Data.pages is null, "nullable numeric/record metadata");
    Assert(empty.Data.author.Count == 0 && empty.Data.editor.Count == 0, "empty managed lists");
    Assert((await RustCore.Search(bibliography, "研究", "title")).Single().Key == reference.Key, "Unicode per-field search");
    Assert((await RustCore.Search(bibliography, "2026", "year")).Single().Key == reference.Key, "year search");
    Assert((await RustCore.Search(bibliography, "Jane", "author")).Count == 2, "author search");
    Assert((await RustCore.Search(bibliography, "", "all", "Book")).Single().Key == "book2025", "type filter");
    Assert((await RustCore.Search(bibliography, "", "author")).Count == 3, "empty field search includes missing fields");
    Assert((await RustCore.Search(bibliography, "Jane 研究", "all", "Article")).Single().Key == reference.Key, "AND search combines ordinary fields");
    Assert((await RustCore.Search(bibliography, "Jane 研究", "title")).Count == 0, "ordinary field selector bounds all terms");
    var ordinaryFile = Path.Combine(directory, "ordinary-search.bib");
    await File.WriteAllTextAsync(ordinaryFile, "@misc{accent,title={Gödel machine-learning network}}\n@misc{godel,title={Unrelated}}\n@misc{plain,title={Godel}}");
    Assert((await RustCore.Search(ordinaryFile, "godel")).Select(x => x.Key).SequenceEqual(new[] { "godel", "plain", "accent" }), "shared relevance ordering crosses generated C# bindings");
    Assert((await RustCore.Search(ordinaryFile, "machine learning", "title")).Single().Key == "accent", "hyphen tolerance and AND title search");
    Assert((await RustCore.Search(ordinaryFile, "netwrok", "title")).Single().Key == "accent", "limited ordinary spelling tolerance");
    await File.WriteAllTextAsync(ordinaryFile, "@misc{changed,title={New graph methods}}");
    Assert((await RustCore.Search(ordinaryFile, "new graph", "title")).Single().Key == "changed", "search text cache refresh crosses C# bindings");
    Assert((await RustCore.Search(bibliography, "no-such-title")).Count == 0, "empty search response");
    var identity = (await RustCore.Search(bibliography, ""))[0];
    Assert(Guid.TryParse(identity.Id, out _) && Guid.TryParse(identity.Data.library_uid, out _), "UUIDs are UIDs");
    var tagState = await RustCore.TagSnapshot(new() { library_uid = identity.Data.library_uid, record_uid = identity.Id, path = "", cite_key = "" });
    var tagged = await RustCore.MutateTag(new() { library_uid = identity.Data.library_uid, record_uid = identity.Id, color = "#3b82f6", action = TagAction.Create, id = "", name = "Interop", target_id = "", path = "", cite_key = "", expected_revision = tagState.revision });
    Assert(tagged.tags.Single().color == "#3B82F6" && tagged.assigned_ids.Count == 1, "tag UUID and color roundtrip");
    Assert((await RustCore.Search(bibliography, "")).Single(r => r.Id == identity.Id).Data.tags.Single().id == tagged.assigned_ids[0], "tag assignment survives generated wire transport");
    await Throws(() => RustCore.Search(bibliography, "", "invalid"), "Unknown search field");
    await Throws(() => RustCore.Search(bibliography, "", "all", "invalid"), "Unknown bibliography type filter");
    var invalid = Path.Combine(directory, "invalid.bib");
    await File.WriteAllTextAsync(invalid, "@article{unfinished,");
    await Throws(() => RustCore.Search(invalid, ""), "");
    await Throws(() => RustCore.Search(Path.Combine(directory, "missing.bib"), ""), "");
    var incompleteQuery = await RustCore.AnalyzeQuery(new TextRequest { text = "author:" });
    Assert(incompleteQuery.status.IsIncomplete && incompleteQuery.document is null, "typed incomplete query");
    var invalidQuery = await RustCore.AnalyzeQuery(new TextRequest { text = "year:abc author:" });
    Assert(invalidQuery.status.IsInvalid && invalidQuery.diagnostics.Count > 0, "definite errors dominate incomplete suffix");
    var batchRows = await RustCore.Search(bibliography, "");
    var batchTargets = batchRows.Select(r => new TagTarget { library_uid = r.Data.library_uid, record_uid = r.Id }).ToList();
    var batchState = await RustCore.BatchTagSnapshot(new() { targets = batchTargets });
    Assert(batchState.count == (ulong)batchRows.Count && batchState.partial_ids.Contains(tagged.assigned_ids[0]), "batch partial state crosses C# bindings");
    var batchAdded = await RustCore.MutateBatchTag(new() { targets = batchTargets, action = TagAction.Assign, id = tagged.assigned_ids[0], name = "", expected_revision = batchState.revision });
    Assert(batchAdded.assigned_ids.Contains(tagged.assigned_ids[0]) && batchAdded.partial_ids.Count == 0, "batch assign is shared across selected records");
    var batchRemoved = await RustCore.MutateBatchTag(new() { targets = batchTargets, action = TagAction.Unassign, id = tagged.assigned_ids[0], name = "", expected_revision = batchAdded.revision });
    Assert(batchRemoved.assigned_ids.Count == 0 && batchRemoved.partial_ids.Count == 0 && batchRemoved.tags.Count == batchAdded.tags.Count, "batch remove preserves catalog");
    var condition = await RustCore.BuildCondition(new QueryCondition { field_name = "title", text_value = "A book", phrase = false, exclude = false, tag_id = null });
    Assert(condition.status.IsValid && condition.document is not null, "typed condition construction");
    var rendered = await RustCore.RenderQuery(condition.document!);
    Assert(rendered.text == condition.text, "typed ordered arena serialization");
    var advanced = await RustCore.AdvancedSearch(new AdvancedSearchRequest { path = bibliography, mode = SearchMode.Advanced, input_kind = QueryInputKind.Bound, text = condition.text, document = condition.document, expected_tag_revision = null, default_field = "all", type_filter = "all", query_version = 42 });
    Assert(advanced.references.Single().cite_key == "book2025" && advanced.context?.query_version == 42, "bound query result and generation");
    Assert(advanced.evidence.Single().ranges.Count > 0, "original source match ranges cross C# bindings");
    Assert((await RustCore.ValidateSearchContext(advanced.context!)).valid, "result snapshot validation");
    var fuzzyFile = Path.Combine(directory, "fuzzy-evidence.bib");
    await File.WriteAllTextAsync(fuzzyFile, "@article{A,title={netwrok graph}}");
    var supplemented = await RustCore.AdvancedSearch(new AdvancedSearchRequest { path = fuzzyFile, mode = SearchMode.Advanced, input_kind = QueryInputKind.Text, text = "network graph", document = null, expected_tag_revision = null, default_field = "title", type_filter = "all", query_version = 43 });
    Assert(supplemented.evidence.Single().fuzzy_terms.SequenceEqual(new[] { "network" }), "actual spelling terms cross C# bindings");
    for (var i = 0; i < 32; i++) {
        var result = await RustCore.AdvancedSearch(new AdvancedSearchRequest { path = bibliography, mode = SearchMode.Advanced, input_kind = QueryInputKind.Text, text = "year:2025 OR type:article", document = null, expected_tag_revision = null, default_field = "all", type_filter = "all", query_version = (ulong)i });
        Assert(result.analysis.status.IsValid && result.references.Count == 2, "repeated advanced wire ownership " + i);
    }
    var parallel = await Task.WhenAll(Enumerable.Range(0, 32).Select(_ => RustCore.Search(bibliography, "研究", "title")));
    Assert(parallel.All(x => x.Single().Id == reference.Id), "parallel owned DTOs and stable identity");
    for (var i = 0; i < 64; i++) Assert((await RustCore.Search(bibliography, "")).Count == 3, "repeated wire ownership " + i);
    await File.AppendAllTextAsync(bibliography, "\n@misc{fresh, title={Newly added}}\n");
    Assert((await RustCore.Search(bibliography, "")).Count == 4, "file cache invalidates after content changes");
    if (Environment.GetEnvironmentVariable("BIBCITEX_SEARCH_BENCHMARK") == "1") {
        foreach (var count in new[] { 1_000, 10_000, 50_000 }) {
            var fixture = Path.Combine(directory, $"payload-{count}.bib");
            await File.WriteAllTextAsync(fixture, string.Join("\n", Enumerable.Range(0, count).Select(i => $"@article{{item{i},title={{Graph neural network}},author={{Smith, Jane}},year={{2026}}}}")));
            int Payload() { using var input = new SearchRequest { path = fixture, query = "", search_field = "all", type_filter = "all" }.Wire(); using var wire = Interop.search(input).AsOk(); var response = wire.Unwire(); if (response.error is not null) throw new InvalidOperationException(response.error); return response.payload.Count; }
            _ = Payload(); var samples = new List<double>();
            for (var i = 0; i < 10; i++) { var timer = System.Diagnostics.Stopwatch.StartNew(); if (Payload() != count) throw new InvalidOperationException("Incomplete benchmark payload"); samples.Add(timer.Elapsed.TotalMilliseconds); }
            samples.Sort(); Console.WriteLine(FormattableString.Invariant($"BINDING,csharp,{count},{samples[5]},{samples[9]}"));
        }
    }
    Console.WriteLine($"PASS: {assertions} assertions against the actual Rust dynamic library. Isolated tag metadata; no clipboard, paste, or UI calls.");
}
finally { Directory.Delete(directory, recursive: true); }

internal static class TestEnvironment
{
    [DllImport("libc", EntryPoint = "setenv")]
    private static extern int SetNative([MarshalAs(UnmanagedType.LPUTF8Str)] string name, [MarshalAs(UnmanagedType.LPUTF8Str)] string value, int overwrite);
    internal static void Isolate(string directory)
    {
        Environment.SetEnvironmentVariable("BIBCITEX_CONFIG_DIRECTORY", directory);
        if (!OperatingSystem.IsWindows() && SetNative("BIBCITEX_CONFIG_DIRECTORY", directory, 1) != 0)
            throw new InvalidOperationException("Could not isolate native test settings");
    }
}
