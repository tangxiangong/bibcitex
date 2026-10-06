using System.Globalization;

namespace BibCiTeX;

internal static class TextElision
{
    internal static string Middle(string text, Func<string, bool> fits)
    {
        if (fits(text)) return text;
        var boundaries = StringInfo.ParseCombiningCharacters(text);
        var low = 0; var high = Math.Max(0, boundaries.Length - 1);
        string Candidate(int count)
        {
            var left = (count + 1) / 2; var right = count / 2;
            return text[..(left == 0 ? 0 : boundaries[left])] + "…" + (right == 0 ? "" : text[boundaries[^right]..]);
        }
        while (low < high)
        {
            var mid = (low + high + 1) / 2;
            if (fits(Candidate(mid))) low = mid; else high = mid - 1;
        }
        return Candidate(low);
    }
}
