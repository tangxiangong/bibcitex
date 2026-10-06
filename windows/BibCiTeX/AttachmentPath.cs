namespace BibCiTeX;

internal static class AttachmentPath
{
    internal static string Resolve(string value, string? bibliography)
    {
        var path = Extract(value);
        // Only explicit file URIs are decoded. Native paths may contain literal %, # and ?.
        if (path.StartsWith("file:", StringComparison.OrdinalIgnoreCase))
        {
            var uri = new Uri(path, UriKind.Absolute);
            if (!uri.IsFile) throw new ArgumentException(nameof(value));
            path = uri.LocalPath;
        }
        if (path == "~" || path.StartsWith("~/") || path.StartsWith("~\\"))
            path = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), path.Length > 1 ? path[2..] : "");
        if (!Path.IsPathRooted(path) && bibliography is not null)
            path = Path.Combine(Path.GetDirectoryName(bibliography)!, path);
        return Path.GetFullPath(path);
    }

    // A semicolon is a filename character unless it follows a complete label:path:format entry.
    // Keep this stage independent of the host OS so Windows spellings are tested on every host.
    internal static string Extract(string value)
    {
        if (value.StartsWith("file:", StringComparison.OrdinalIgnoreCase)) return value;
        for (var i = 0; i < value.Length; i++)
        {
            if (value[i] == ';' && (i == 0 || value[i - 1] != '\\') && TryEntry(value[..i], out var first)) return first;
        }
        return TryEntry(value, out var path) ? path : value;
    }

    private static bool TryEntry(string value, out string path)
    {
        path = "";
        var first = value.IndexOf(':');
        var last = value.LastIndexOf(':');
        if (first < 0 || first == last || last == value.Length - 1) return false;
        var format = value[(last + 1)..];
        if (!format.All(c => char.IsAsciiLetterOrDigit(c) || c is '_' or '-' or '+')) return false;
        var driveStart = value.StartsWith(@"\\?\", StringComparison.Ordinal) ? 4 : 0;
        var drive = first == driveStart + 1 && char.IsAsciiLetter(value[driveStart])
            && value.Length > driveStart + 2 && value[driveStart + 2] is '/' or '\\';
        var start = drive ? 0 : first + 1;
        if (start >= last) return false;
        path = value[start..last].Replace("\\;", ";").Replace("\\:", ":");
        return true;
    }
}
