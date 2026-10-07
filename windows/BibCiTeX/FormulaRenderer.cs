using CSharpMath.SkiaSharp;
using CSharpMath.Atom;
using System.Text;
using SkiaSharp;

namespace BibCiTeX;

internal readonly record struct RenderedFormula(byte[] Bytes, double Width, double Height, double Baseline);

// CPU-only rendering shared by WinUI and the no-window integration tests.
internal static class FormulaRenderer
{
    private static readonly HashSet<string> RegisteredCommands = new(LaTeXSettings.Commands.Select(x => x.Key), StringComparer.Ordinal);

    internal static RenderedFormula Render(string latex, float fontSize, float scale, double availableWidth, bool dark)
    {
        if (!float.IsFinite(fontSize) || fontSize <= 0) throw new ArgumentOutOfRangeException(nameof(fontSize));
        if (!float.IsFinite(scale) || scale <= 0) throw new ArgumentOutOfRangeException(nameof(scale));
        if (!double.IsFinite(availableWidth) || availableWidth <= 8) throw new ArgumentOutOfRangeException(nameof(availableWidth));
        var painter = new MathPainter
        {
            LaTeX = NormalizeControlWords(latex),
            // WinUI uses DIPs (96/inch); CSharpMath uses points (72/inch).
            FontSize = fontSize * (72f / 96f) * scale,
            LineStyle = LineStyle.Text,
            TextColor = dark ? SKColors.White : SKColors.Black,
            DisplayErrorInline = false
        };
        var bounds = painter.Measure();
        if (painter.ErrorMessage is not null) throw new FormatException(painter.ErrorMessage);
        if (!float.IsFinite(bounds.Width) || !float.IsFinite(bounds.Height)) throw new FormatException(latex);

        // Padding is in logical units, so changing monitor DPI preserves layout.
        var padding = scale;
        var widthBudget = Math.Max(1, Math.Min(8192, availableWidth * scale) - 2 * padding);
        var heightBudget = Math.Max(1, 8192 - 2 * padding);
        var fit = (float)Math.Min(1, Math.Min(widthBudget / Math.Max(1, bounds.Width), heightBudget / Math.Max(1, bounds.Height)));
        if (fit < 1) { painter.FontSize *= fit; bounds = painter.Measure(); }
        var width = Math.Max(1, (int)Math.Ceiling(bounds.Width + 2 * padding));
        var height = Math.Max(1, (int)Math.Ceiling(bounds.Height + 2 * padding));
        using var surface = SKSurface.Create(new SKImageInfo(width, height)) ?? throw new InvalidOperationException("Cannot allocate formula surface.");
        surface.Canvas.Clear(SKColors.Transparent);
        // MathPainter.Draw(x, y) positions the baseline; Measure.Y is -ascent.
        painter.Draw(surface.Canvas, padding, -bounds.Y + padding);
        using var image = surface.Snapshot();
        using var data = image.Encode(SKEncodedImageFormat.Png, 100);
        return new RenderedFormula(data.ToArray(), width / scale, height / scale, (-bounds.Y + padding) / scale);
    }

    private static string NormalizeControlWords(string latex)
    {
        // CSharpMath 0.5.1 SplitCommand absorbs a following =, * or prime into
        // every control word. Keep any registered suffix commands
        // intact, but delimit ordinary words so \beta= and \alpha' remain valid TeX.
        var result = new StringBuilder(latex.Length);
        for (var index = 0; index < latex.Length;)
        {
            if (latex[index] != '\\') { result.Append(latex[index++]); continue; }
            var start = index++;
            if (index == latex.Length) { result.Append('\\'); break; }
            if (!IsControlLetter(latex[index]))
            {
                result.Append(latex, start, 2); index++; continue;
            }
            while (index < latex.Length && IsControlLetter(latex[index])) index++;
            result.Append(latex, start, index - start);
            if (index < latex.Length && latex[index] is '=' or '*' or '\'' &&
                !RegisteredCommands.Contains(latex.Substring(start, index - start + 1)))
                result.Append(' ');
        }
        return result.ToString();
    }

    private static bool IsControlLetter(char value) => value is >= 'a' and <= 'z' or >= 'A' and <= 'Z' or '@';
}
