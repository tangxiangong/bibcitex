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
    internal static Func<IEnumerable<string>> SystemLanguages { get; set; } = () => [CultureInfo.InstalledUICulture.Name];
    internal static string Language { get; private set; } = "en";
    internal static event Action<string>? Changing;
    internal static event Action? Changed;
    internal static string Resolve(string selection, string preferred) => Resolve(selection, [preferred]);
    internal static string Resolve(string selection, IEnumerable<string> preferred)
    {
        if (selection is "zh-Hans" or "en") return selection;
        foreach (var language in preferred)
        {
            var tag = language.Replace('_', '-').ToLowerInvariant();
            if (tag is "zh" or "zh-cn" or "zh-sg" || tag.StartsWith("zh-hans", StringComparison.Ordinal)) return "zh-Hans";
            if (tag == "en" || tag.StartsWith("en-", StringComparison.Ordinal)) return "en";
        }
        return "en";
    }
    internal static void Select(string selection)
    {
        selection = Languages.Contains(selection) ? selection : "system";
        var language = Resolve(selection, SystemLanguages());
        if (Selection == selection && Language == language) return;
        Changing?.Invoke(language);
        Selection = selection;
        Language = language;
        Changed?.Invoke();
    }
    internal static void RefreshSystemLanguage() { if (Selection == "system") Select(Selection); }
    internal static string ErrorMessage(Exception error)
    {
        if (error is LocalizedException localized)
        {
            if (Catalogs["en"].ContainsKey(localized.Key)) return Text(localized.Key, localized.Arguments);
            // Compatibility with service errors that include diagnostic details in a string.
            const string prefix = "Invalid settings: ";
            if (localized.Key.StartsWith(prefix, StringComparison.Ordinal))
                return Text("error.invalidSettings", localized.Key[prefix.Length..]);
        }
        // OS/library messages are diagnostics, never UI keys. Keep the details intact.
        return Text("error.operationFailed", error is LocalizedException core ? core.Key : error.Message);
    }
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
internal sealed class LocalizedException(string key, params object[] arguments) : InvalidOperationException(L10n.Text(key, arguments))
{
    internal string Key { get; } = key;
    internal object[] Arguments { get; } = arguments;
}
