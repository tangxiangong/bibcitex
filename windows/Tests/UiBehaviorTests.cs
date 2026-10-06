using BibCiTeX;

internal static class UiBehaviorTests
{
    internal static void Run()
    {
        void Assert(bool value, string message) { if (!value) throw new InvalidOperationException(message); }
        var canonical = @"\\?\C:\Users\xiaoyu\Desktop\test.bib";
        Assert(PathDisplay.Format(canonical) == @"C:\Users\xiaoyu\Desktop\test.bib", "Hide extended drive prefix in UI");
        var longSuffix = new string('x', 300) + @"\文献.bib";
        Assert(PathDisplay.Format(@"\\?\C:\" + longSuffix) == @"C:\" + longSuffix, "Display preserves long path contents");
        Assert(PathDisplay.Format(@"\\?\UNC\server\share\文献.bib") == @"\\server\share\文献.bib", "Present extended UNC as a network path");
        Assert(PathDisplay.Format(@"\\?\unc\server\share\test.bib") == @"\\server\share\test.bib", "UNC namespace is case insensitive");
        foreach (var ordinary in new[] { @"C:\papers\test.bib", @"\\server\share\test.bib", "relative.bib", "", @"\\?\Volume{123}\test.bib", @"\\.\PhysicalDrive0" })
            Assert(PathDisplay.Format(ordinary) == ordinary, "Preserve ordinary and device path spellings: " + ordinary);
        foreach (var width in new[] { 800d, 940d, 1050d, 1200d, 1800d })
        foreach (var left in new[] { false, true })
        foreach (var right in new[] { false, true })
        foreach (var desired in new[] { (180d, 280d), (220d, 350d), (300d, 520d) })
        {
            var columns = WorkbenchLayout.Columns(width, left, right, desired.Item1, desired.Item2);
            Assert(left ? columns.Left is >= 180 and <= 300 : columns.Left == 0, "Sidebar collapse/resize bounds");
            Assert(right ? columns.Right is >= 280 and <= 520 : columns.Right == 0, "Inspector collapse/resize bounds");
            Assert(width - columns.Left - columns.Right - (left ? 5 : 0) - (right ? 5 : 0) >= 330, "Resizing must preserve usable center width");
        }
        Assert(TextElision.Middle("abcdef", x => x.Length <= 5) == "ab…ef", "Keep path prefix and filename suffix");
        Assert(TextElision.Middle("short", x => x.Length <= 10) == "short", "Keep fitting text intact");
        Assert(TextElision.Middle("字", _ => false) == "…", "Single grapheme overflow");
        Assert(TextElision.Middle("", _ => true) == "", "Empty path");
        Assert(TextElision.Middle("😀abc😀", x => x.Length <= 5) == "😀…😀", "Do not split Unicode text elements");
        var directory = Path.Combine(Path.GetTempPath(), "BibCiTeX 路径");
        var library = Path.Combine(directory, "library.bib");
        var expected = Path.Combine(directory, "paper.pdf");
        foreach (var input in new[] { "paper.pdf", ":paper.pdf:PDF", "attachment:paper.pdf:PDF", "a:paper.pdf:PDF", "attachment:paper.pdf:PDF;other:second.pdf:PDF" })
            Assert(AttachmentPath.Resolve(input, library) == expected, "Relative/manager attachment resolves against its library: " + input);
        foreach (var name in new[] { "paper;v2.pdf", "paper#part.pdf", "paper%20literal.pdf" })
        {
            var literal = Path.Combine(directory, name);
            Assert(AttachmentPath.Resolve(name, library) == literal, "Preserve literal relative filename: " + name);
            Assert(AttachmentPath.Resolve(literal, library) == literal, "Preserve literal absolute filename: " + name);
            Assert(AttachmentPath.Resolve(new Uri(expected).AbsoluteUri.Replace("paper.pdf", Uri.EscapeDataString(name)), library) == literal, "Decode explicit file URI once: " + name);
        }
        Assert(AttachmentPath.Resolve(new Uri(expected).AbsoluteUri, library) == expected, "File URI decoding");
        Assert(AttachmentPath.Resolve("~/paper.pdf", library) == Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), "paper.pdf"), "Home expansion");
        foreach (var spelling in new[] { @"C:\papers\paper.pdf", @"C:\papers\paper;v2.pdf", @"\\server\share\paper.pdf", @"\\server\share\paper;v2.pdf", @"\\?\C:\papers\paper.pdf" })
        {
            Assert(AttachmentPath.Extract(spelling) == spelling, "Preserve Windows native path on every test host: " + spelling);
            Assert(AttachmentPath.Extract("label:" + spelling + ":PDF;other:second.pdf:PDF") == spelling, "Extract Windows attachment on every test host: " + spelling);
        }
        Assert(AttachmentPath.Extract(@"\\?\C:\papers\paper.pdf:PDF") == @"\\?\C:\papers\paper.pdf", "Retain extended drive prefix in unlabelled attachment");
        Assert(AttachmentPath.Extract(@":paper\;v2.pdf:PDF;other:second.pdf:PDF") == "paper;v2.pdf", "Escaped semicolon in structured attachment");
        Assert(AttachmentPath.Extract(":paper;v2.pdf:PDF;other:second.pdf:PDF") == "paper;v2.pdf", "Filename semicolon before attachment format");
        Assert(AttachmentPath.Resolve("label:" + new Uri(expected).AbsoluteUri + ":PDF", library) == expected, "File URI inside a labeled attachment");
        if (OperatingSystem.IsWindows())
        {
            foreach (var input in new[] { @"C:\papers\paper.pdf", @":C:\papers\paper.pdf:PDF", @"label:C:\papers\paper.pdf:PDF", @"a:C:\papers\paper.pdf:PDF" })
                Assert(AttachmentPath.Resolve(input, library) == @"C:\papers\paper.pdf", "Preserve Windows drive colon: " + input);
        }
        Console.WriteLine("UI behavior: pane combinations, narrow windows and attachment actions passed.");
    }
}
