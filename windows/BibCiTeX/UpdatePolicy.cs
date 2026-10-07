namespace BibCiTeX;

internal static class UpdatePolicy
{
    internal const string BaseUrl = "https://app-release-1302963684.cos.ap-guangzhou.myqcloud.com/bibcitex/update-feed";
    internal const string FallbackUrl = "https://github.com/tangxiangong/bibcitex/releases/download/update-feed";
    internal static string DefaultChannel(string version) => version.Contains("-alpha.", StringComparison.Ordinal) ? "alpha"
        : version.Contains("-beta.", StringComparison.Ordinal) ? "beta" : "stable";
    internal static string Channel(string? saved, string version) => saved is "stable" or "beta" or "alpha" ? saved : DefaultChannel(version);
    internal static string FeedChannel(string architecture, string channel) => architecture is "x64" or "arm64"
        && channel is "stable" or "beta" or "alpha" ? $"win-{architecture}-{channel}" : throw new ArgumentException("Invalid update target");
    internal static bool CanSchedule(long startedRevision, long currentRevision, bool enabled) => enabled && startedRevision == currentRevision;
    internal static bool CanApplyPending(bool enabled, string? requestedVersion, string? downloadedVersion, string channel)
    {
        if (!enabled || string.IsNullOrEmpty(requestedVersion) || requestedVersion != downloadedVersion) return false;
        var preview = DefaultChannel(requestedVersion);
        return channel switch { "alpha" => true, "beta" => preview != "alpha", "stable" => preview == "stable", _ => false };
    }
    internal static string Notes(string markdown, string language)
    {
        var marker = $"<!-- locale:{language} -->";
        var start = markdown.IndexOf(marker, StringComparison.Ordinal);
        if (start < 0 && language != "en") return Notes(markdown, "en");
        if (start < 0) return markdown.Contains("<!-- locale:", StringComparison.Ordinal) ? "" : markdown;
        start += marker.Length;
        var end = markdown.IndexOf("<!-- /locale -->", start, StringComparison.Ordinal);
        return end < 0 ? "" : markdown[start..end].Trim();
    }
}
