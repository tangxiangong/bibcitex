namespace BibCiTeX;

// Presentation only. Keep canonical paths unchanged for storage, identity and file access.
internal static class PathDisplay
{
    internal static string Format(string path)
    {
        const string networkPrefix = @"\\?\UNC\";
        if (path.StartsWith(networkPrefix, StringComparison.OrdinalIgnoreCase))
            return @"\\" + path[networkPrefix.Length..];
        if (path.StartsWith(@"\\?\", StringComparison.Ordinal) && path.Length >= 7
            && char.IsAsciiLetter(path[4]) && path[5] == ':' && path[6] == '\\')
            return path[4..];
        // Volume GUID and device namespaces have no equivalent ordinary display spelling.
        return path;
    }
}
