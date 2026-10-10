using System.Reflection;
using System.Runtime.InteropServices;
using BibCiTeX;

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
    await Throws(() => RustCore.Search(bibliography, "", "invalid"), "Unknown search field");
    await Throws(() => RustCore.Search(bibliography, "", "all", "invalid"), "Unknown bibliography type filter");
    var invalid = Path.Combine(directory, "invalid.bib");
    await File.WriteAllTextAsync(invalid, "@article{unfinished,");
    await Throws(() => RustCore.Search(invalid, ""), "");
    await Throws(() => RustCore.Search(Path.Combine(directory, "missing.bib"), ""), "");
    var parallel = await Task.WhenAll(Enumerable.Range(0, 32).Select(_ => RustCore.Search(bibliography, "研究", "title")));
    Assert(parallel.All(x => x.Single().Id == reference.Id), "parallel owned DTOs and stable identity");
    for (var i = 0; i < 64; i++) Assert((await RustCore.Search(bibliography, "")).Count == 3, "repeated wire ownership " + i);
    await File.AppendAllTextAsync(bibliography, "\n@misc{fresh, title={Newly added}}\n");
    Assert((await RustCore.Search(bibliography, "")).Count == 4, "file cache invalidates after content changes");
    Console.WriteLine($"PASS: {assertions} assertions against the actual Rust dynamic library. No registry, clipboard, paste, or UI calls.");
}
finally { Directory.Delete(directory, recursive: true); }
