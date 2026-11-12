namespace BibCiTeX;

internal sealed record HelperShortcut(uint Modifiers, uint Key, string Value)
{
    internal const string DefaultValue = "Win+Shift+K";
    internal string DisplayValue => Value.Replace("+", " + ");

    internal static HelperShortcut? FromKey(uint key, uint modifiers)
    {
        var name = key switch
        {
            >= 0x41 and <= 0x5A or >= 0x30 and <= 0x39 => ((char)key).ToString(),
            0x20 => "Space",
            >= 0x70 and <= 0x7B => "F" + (key - 0x6F),
            _ => ""
        };
        var names = new List<string>();
        foreach (var (bit, label) in new (uint, string)[] { (8, "Win"), (2, "Ctrl"), (1, "Alt"), (4, "Shift") })
            if ((modifiers & bit) != 0) names.Add(label);
        names.Add(name);
        return Parse(string.Join('+', names));
    }

    internal static HelperShortcut? Parse(string text)
    {
        var parts = text.ToUpperInvariant().Split('+', StringSplitOptions.TrimEntries);
        if (parts.Length < 2) return null;
        uint modifiers = 0;
        foreach (var part in parts[..^1])
        {
            uint bit = part switch
            {
                "CTRL" or "CONTROL" => 2,
                "ALT" => 1,
                "SHIFT" => 4,
                "WIN" or "SUPER" => 8,
                _ => 0
            };
            if (bit == 0 || (modifiers & bit) != 0) return null;
            modifiers |= bit;
        }
        if ((modifiers & (1 | 2 | 8)) == 0) return null;
        var name = parts[^1];
        uint key;
        if (name.Length == 1 && name[0] is >= 'A' and <= 'Z' or >= '0' and <= '9') key = name[0];
        else if (name == "SPACE") key = 0x20;
        else if (name.StartsWith('F') && int.TryParse(name.AsSpan(1), out var number) && number is >= 1 and <= 12)
        { key = (uint)(0x6F + number); name = "F" + number; }
        else return null;
        var names = new List<string>();
        foreach (var (bit, label) in new (uint, string)[] { (8, "Win"), (2, "Ctrl"), (1, "Alt"), (4, "Shift") })
            if ((modifiers & bit) != 0) names.Add(label);
        names.Add(name == "SPACE" ? "Space" : name);
        return new(modifiers, key, string.Join('+', names));
    }
}
