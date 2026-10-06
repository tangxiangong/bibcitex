namespace BibCiTeX;

// Pure layout constraints shared by pointer/keyboard resizing and window resizing.
internal static class WorkbenchLayout
{
    internal static (double Left, double Right) Columns(double width, bool left, bool right, double preferredLeft, double preferredRight)
    {
        var available = Math.Max(0, width - 330 - (left ? 5 : 0) - (right ? 5 : 0));
        var l = left ? Math.Clamp(preferredLeft, 180, 300) : 0;
        var r = right ? Math.Clamp(preferredRight, 280, 520) : 0;
        var overflow = Math.Max(0, l + r - available);
        var shrink = Math.Min(overflow, Math.Max(0, r - (right ? 280 : 0)));
        r -= shrink; overflow -= shrink;
        l -= Math.Min(overflow, Math.Max(0, l - (left ? 180 : 0)));
        return (l, r);
    }
}
