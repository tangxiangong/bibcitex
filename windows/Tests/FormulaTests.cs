using BibCiTeX;
using SkiaSharp;

internal static class FormulaTests
{
    internal static void Run(Action<bool, string> assert)
    {
        var samples = new[]
        {
            @"x^2+y_1",
            @"\frac{a+b}{\sqrt{x^2+y^2}}",
            @"\int_0^\infty e^{-x^2}\,dx=\frac{\sqrt{\pi}}{2}",
            @"\alpha+\beta=\gamma",
            "α+β=γ",
            @"\alpha' + \beta*2",
            @"\operatorname{lim}_{x \to 0} x",
            @"\left(\begin{matrix}a&b\\c&d\end{matrix}\right)"
        };
        foreach (var sample in samples)
        {
            var light = FormulaRenderer.Render(sample, 22, 1, 640, false);
            CheckInkAndMargins(light, assert, sample);
            var dark = FormulaRenderer.Render(sample, 22, 1, 640, true);
            using var lightBitmap = SKBitmap.Decode(light.Bytes);
            using var darkBitmap = SKBitmap.Decode(dark.Bytes);
            assert(lightBitmap.Width == darkBitmap.Width && lightBitmap.Height == darkBitmap.Height, "formula theme keeps geometry");
            var lightPixels = lightBitmap.Pixels.Where(x => x.Alpha > 100).ToArray();
            var darkPixels = darkBitmap.Pixels.Where(x => x.Alpha > 100).ToArray();
            assert(lightPixels.Length > 0 && lightPixels.All(x => x.Red < 10 && x.Green < 10 && x.Blue < 10), "light formula foreground");
            assert(darkPixels.Length > 0 && darkPixels.All(x => x.Red > 245 && x.Green > 245 && x.Blue > 245), "dark formula foreground");
            foreach (var dpi in new[] { 1.25f, 1.5f, 2f, 3f })
            {
                var scaled = FormulaRenderer.Render(sample, 22, dpi, 640, false);
                assert(Math.Abs(scaled.Width - light.Width) <= 1.05, "formula logical width survives DPI " + dpi);
                assert(Math.Abs(scaled.Height - light.Height) <= 1.05, "formula logical height survives DPI " + dpi);
                CheckInkAndMargins(scaled, assert, "DPI " + dpi + " " + sample);
            }
        }
        var longFormula = string.Join("+", Enumerable.Repeat(@"\frac{x_i^2}{1+y_i}", 36)) + @"=\sqrt{z}";
        foreach (var dpi in new[] { 1f, 1.5f, 2f })
        {
            var fitted = FormulaRenderer.Render(longFormula, 22, dpi, 300, false);
            assert(fitted.Width <= 300.01, "long equation fits without cropping at DPI " + dpi);
            CheckInkAndMargins(fitted, assert, "long equation DPI " + dpi);
        }
        var empty = FormulaRenderer.Render("", 22, 1, 640, false);
        using (var bitmap = SKBitmap.Decode(empty.Bytes)) assert(bitmap.Pixels.All(x => x.Alpha == 0), "empty formula remains transparent");
        var rejected = false;
        try { FormulaRenderer.Render(@"\definitelyUnsupportedMacro{x}", 22, 1, 640, false); }
        catch (FormatException) { rejected = true; }
        assert(rejected, "unsupported LaTeX reports failure for native text fallback");
        rejected = false;
        try { FormulaRenderer.Render("x", 22, 0, 640, false); }
        catch (ArgumentOutOfRangeException) { rejected = true; }
        assert(rejected, "invalid DPI is rejected before allocating a surface");
    }

    private static void CheckInkAndMargins(RenderedFormula formula, Action<bool, string> assert, string name)
    {
        using var bitmap = SKBitmap.Decode(formula.Bytes);
        assert(bitmap is not null && bitmap.Width > 2 && bitmap.Height > 2, "formula decodes: " + name);
        if (bitmap is null) return;
        assert(bitmap.Pixels.Any(x => x.Alpha > 0), "formula contains rendered ink: " + name);
        var marginsClear = true;
        for (var x = 0; x < bitmap.Width; x++)
            marginsClear &= bitmap.GetPixel(x, 0).Alpha == 0 && bitmap.GetPixel(x, bitmap.Height - 1).Alpha == 0;
        for (var y = 0; y < bitmap.Height; y++)
            marginsClear &= bitmap.GetPixel(0, y).Alpha == 0 && bitmap.GetPixel(bitmap.Width - 1, y).Alpha == 0;
        assert(marginsClear, "formula has transparent margins on all four sides: " + name);
    }
}
