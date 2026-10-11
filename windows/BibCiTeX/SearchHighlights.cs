using System.Text;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Documents;

namespace BibCiTeX;

/// <summary>Maps service-provided original UTF-8 match ranges to native UTF-16 text without re-running matching.</summary>
internal static class SearchHighlights
{
    /// <summary>Returns valid character ranges in one original field entry. Invalid or split-code-point offsets are ignored.</summary>
    internal static List<(int Start, int End)> Ranges(Reference reference, string field, string original, uint entry = 0)
    {
        var bytes = Encoding.UTF8.GetBytes(original);
        var strict = new UTF8Encoding(false, true);
        var result = new List<(int Start, int End)>();
        foreach (var range in reference.Evidence?.ranges.Where(x => x.field_name == field && x.entry_index == entry) ?? [])
        {
            if (range.start >= range.end || range.end > bytes.Length) continue;
            try { result.Add((strict.GetCharCount(bytes.AsSpan(0, (int)range.start)), strict.GetCharCount(bytes.AsSpan(0, (int)range.end)))); }
            catch (DecoderFallbackException) { /* Never slice a native surrogate or a partial UTF-8 scalar. */ }
        }
        return result;
    }

    /// <summary>Adds normal and emphasized runs using original-text character ranges; offsets may span successive BibTeX chunks.</summary>
    internal static void Append(InlineCollection inlines, string original, IReadOnlyList<(int Start, int End)> ranges, int offset = 0)
    {
        var boundaries = new SortedSet<int> { 0, original.Length };
        foreach (var (start, end) in ranges)
        {
            if (end <= offset || start >= offset + original.Length) continue;
            boundaries.Add(Math.Clamp(start - offset, 0, original.Length));
            boundaries.Add(Math.Clamp(end - offset, 0, original.Length));
        }
        var points = boundaries.ToArray();
        for (int i = 1; i < points.Length; i++)
        {
            int start = points[i - 1], end = points[i];
            var run = new Run { Text = original[start..end] };
            if (ranges.Any(x => x.Start < end + offset && x.End > start + offset)) run.FontWeight = Microsoft.UI.Text.FontWeights.Bold;
            inlines.Add(run);
        }
    }

    /// <summary>Builds the author display while preserving each service entry index, including separators between distinct authors.</summary>
    internal static TextBlock Authors(Reference reference, double size)
    {
        var result = new TextBlock { FontSize = size, TextTrimming = Microsoft.UI.Xaml.TextTrimming.CharacterEllipsis, Opacity = .75 };
        for (int i = 0; i < reference.Data.author.Count; i++)
        {
            if (i > 0) result.Inlines.Add(new Run { Text = ", " });
            var author = reference.Data.author[i];
            Append(result.Inlines, author, Ranges(reference, "author", author, (uint)i));
        }
        return result;
    }
}
