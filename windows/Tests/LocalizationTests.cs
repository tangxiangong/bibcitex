using System.Text.Json;
using System.Globalization;
using System.Text.RegularExpressions;

namespace BibCiTeX;

internal static class LocalizationTests
{
    internal static void Run()
    {
        void Check(bool value) { if (!value) throw new InvalidOperationException("Localization regression failed"); }
        Check(L10n.Resolve("system", "zh-Hans-CN") == "zh-Hans");
        Check(L10n.Resolve("system", "zh_CN") == "zh-Hans");
        Check(L10n.Resolve("system", "zh-Hant-TW") == "en");
        Check(L10n.Resolve("system", "en-GB") == "en");
        Check(L10n.Resolve("invalid", "fr-FR") == "en");
        Check(L10n.Resolve("en", "zh-CN") == "en");
        Check(L10n.Resolve("zh-Hans", "en-US") == "zh-Hans");
        Check(L10n.Translate("复制引用键", "en") == "Copy citation key");
        Check(L10n.Translate("复制引用键", "zh-Hans") == "复制引用键");
        Check(L10n.Translate("1 条文献", "en") == "1 reference");
        Check(L10n.Translate("{0} 条文献", "en", 42) == "42 references");
        Check(L10n.Translate("复制引用键 {0}", "en", "文献 {0} β") == "Copy citation key 文献 {0} β");
        Check(L10n.Translate("metadata.editor", "en") == "Editor");
        Check(L10n.Translate("metadata.editor", "zh-Hans") == "编辑");
        Check(L10n.Translate("unknown key", "en") == "unknown key");
        Check(L10n.Translate("/tmp/文献.bib", "en") == "/tmp/文献.bib");
        Check(L10n.Translate("搜索", "unsupported") == "Search");
        var previous = L10n.Selection;
        var preferred = L10n.SystemLanguage;
        var culture = CultureInfo.CurrentUICulture;
        var changes = 0;
        void Changed() => changes++;
        L10n.Changed += Changed;
        try
        {
            L10n.SystemLanguage = () => "zh-CN";
            CultureInfo.CurrentUICulture = CultureInfo.GetCultureInfo("en-US");
            L10n.Select("system"); Check(L10n.Language == "zh-Hans");
            L10n.Select("en");
            var visibleError = new LocalizedException("文献库名称不能为空");
            Check(L10n.ErrorMessage(visibleError) == "Library name cannot be empty");
            L10n.Select("zh-Hans"); Check(L10n.ErrorMessage(visibleError) == "文献库名称不能为空");
            Check(L10n.ErrorMessage(new IOException("/tmp/文献.bib {0}")) == "/tmp/文献.bib {0}");
            L10n.Select("system"); Check(L10n.Language == "zh-Hans");
            L10n.SystemLanguage = () => "en-US"; Check(L10n.Language == "en");
            L10n.Select("en"); Check(L10n.References(1) == "1 reference");
            L10n.Select("zh-Hans"); Check(L10n.References(1) == "1 条文献");
            Check(L10n.References(42) == "42 条文献");
            L10n.Select("en"); Check(L10n.References(42) == "42 references");
            var count = changes; L10n.Select("en"); Check(changes == count);
        }
        finally { L10n.Changed -= Changed; L10n.SystemLanguage = preferred; CultureInfo.CurrentUICulture = culture; L10n.Select(previous); }
        Dictionary<string, string> Catalog(string code)
        {
            using var stream = typeof(L10n).Assembly.GetManifestResourceStream($"BibCiTeX.Localization.{code}.json")!;
            return JsonSerializer.Deserialize<Dictionary<string, string>>(stream)!;
        }
        var en = Catalog("en"); var zh = Catalog("zh-Hans");
        Check(en.Count > 100 && en.Keys.Order().SequenceEqual(zh.Keys.Order()));
        foreach (var (key, value) in en)
        {
            Check(value.Length > 0 && zh[key].Length > 0);
            string[] Placeholders(string text) => Regex.Matches(text, @"\{[0-9]+\}").Select(match => match.Value).Order().ToArray();
            Check(Placeholders(value).SequenceEqual(Placeholders(zh[key])));
        }
        var textCommands = new Dictionary<string, (string En, string Zh)>
        {
            ["menu.system.Undo"] = ("Undo", "撤销"), ["menu.system.Redo"] = ("Redo", "重做"),
            ["menu.cut"] = ("Cut", "剪切"), ["menu.copy"] = ("Copy", "拷贝"),
            ["menu.paste"] = ("Paste", "粘贴"), ["menu.delete"] = ("Delete", "删除"), ["menu.selectAll"] = ("Select All", "全选")
        };
        foreach (var (key, labels) in textCommands) Check(en[key] == labels.En && zh[key] == labels.Zh);
        Console.WriteLine("PASS Windows native text command labels, retained error keys and system-language restoration");
        Console.WriteLine("PASS C# localization: embedded catalogs, language switching, fallback, placeholders and singular/plural counts");
    }
}
