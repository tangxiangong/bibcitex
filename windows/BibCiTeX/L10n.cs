using System.Globalization;
using System.Text.Json;

namespace BibCiTeX;

internal static class L10n
{
    internal static readonly string[] Languages = ["system", "zh-Hans", "en"];
    private static readonly Dictionary<string, Dictionary<string, string>> Catalogs =
        new[] { "zh-Hans", "en" }.ToDictionary(language => language, language =>
        {
            using var stream = typeof(L10n).Assembly.GetManifestResourceStream($"BibCiTeX.Localization.{language}.json")
                ?? throw new InvalidOperationException($"Missing localization resource: {language}");
            return JsonSerializer.Deserialize<Dictionary<string, string>>(stream)!;
        });
    internal static string Selection { get; private set; } = "system";
    internal static Func<string> SystemLanguage { get; set; } = () => CultureInfo.InstalledUICulture.Name;
    internal static string Language => Resolve(Selection, SystemLanguage());
    internal static event Action? Changed;
    internal static string Resolve(string selection, string preferred)
    {
        if (selection is "zh-Hans" or "en") return selection;
        var tag = preferred.Replace('_', '-').ToLowerInvariant();
        return tag is "zh" or "zh-cn" or "zh-sg" || tag.StartsWith("zh-hans", StringComparison.Ordinal) ? "zh-Hans" : "en";
    }
    internal static void Select(string selection)
    {
        selection = Languages.Contains(selection) ? selection : "system";
        if (Selection == selection) return;
        Selection = selection;
        Changed?.Invoke();
    }
    internal static string ErrorMessage(Exception error) => error is LocalizedException localized ? Text(localized.Key) : error.Message;
    internal static string Text(string key, params object[] arguments) => Translate(key, Language, arguments);
    internal static string Translate(string key, string language, params object[] arguments)
    {
        var value = Catalogs.GetValueOrDefault(language)?.GetValueOrDefault(key)
            ?? Catalogs["en"].GetValueOrDefault(key) ?? key;
        return arguments.Length == 0 ? value : string.Format(CultureInfo.GetCultureInfo(Catalogs.ContainsKey(language) ? language : "en"), value, arguments);
    }
    internal static string References(int count) => count == 1 ? Text("1 条文献") : Text("{0} 条文献", count);
}

/// Retain the source key so a visible error can follow later language changes.
internal sealed class LocalizedException(string key) : InvalidOperationException(L10n.Text(key))
{
    internal string Key { get; } = key;
}
