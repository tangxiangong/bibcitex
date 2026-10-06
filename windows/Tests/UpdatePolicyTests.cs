using BibCiTeX;

internal static class UpdatePolicyTests
{
    internal static void Run()
    {
        static void Equal(string expected, string actual) { if (expected != actual) throw new InvalidOperationException($"Expected {expected}, got {actual}"); }
        Equal("alpha", UpdatePolicy.Channel(null, "0.7.0-alpha.10"));
        Equal("beta", UpdatePolicy.Channel("bad", "0.7.0-beta.1"));
        Equal("stable", UpdatePolicy.Channel("stable", "0.7.0-alpha.1"));
        Equal("win-arm64-beta", UpdatePolicy.FeedChannel("arm64", "beta"));
        static void Assert(bool value) { if (!value) throw new InvalidOperationException("Update policy regression"); }
        Assert(UpdatePolicy.CanSchedule(1, 1, true));
        Assert(!UpdatePolicy.CanSchedule(1, 2, true));
        Assert(!UpdatePolicy.CanSchedule(1, 1, false));
        Assert(UpdatePolicy.CanApplyPending(true, "1.1.0-beta.1", "1.1.0-beta.1", "beta"));
        Assert(!UpdatePolicy.CanApplyPending(true, "1.1.0-beta.1", "1.2.0-alpha.1", "alpha"));
        Assert(!UpdatePolicy.CanApplyPending(false, "1.1.0", "1.1.0", "stable"));
        Assert(!UpdatePolicy.CanApplyPending(true, "1.1.0-alpha.1", "1.1.0-alpha.1", "beta"));
        Assert(!UpdatePolicy.CanApplyPending(true, "1.1.0-beta.1", "1.1.0-beta.1", "stable"));
        var directory = Path.Combine(Path.GetTempPath(), "bibcitex-update-settings-" + Guid.NewGuid());
        Directory.CreateDirectory(directory);
        try
        {
            var path = Path.Combine(directory, "settings.json");
            File.WriteAllText(path, "{broken");
            var settings = new NativeSettings.Store(path);
            Assert(settings["language"] is null);
            Assert(File.ReadAllText(Directory.GetFiles(directory, "*.corrupt-*").Single()) == "{broken");
            settings["language"] = "en";
            settings["pendingUpdate"] = "1.1.0";
            var reloaded = new NativeSettings.Store(path);
            Equal("en", (string)reloaded["language"]!);
            Equal("1.1.0", (string)reloaded["pendingUpdate"]!);
        }
        finally { Directory.Delete(directory, true); }
        const string markdown = "<!-- locale:zh-Hans -->\n# 中文\n<!-- /locale -->\n<!-- locale:en -->\n# English\n<!-- /locale -->";
        Equal("# 中文", UpdatePolicy.Notes(markdown, "zh-Hans"));
        Equal("# English", UpdatePolicy.Notes(markdown, "en"));
        Equal("# English", UpdatePolicy.Notes(markdown, "fr"));
        Equal("", UpdatePolicy.Notes("<!-- locale:en -->truncated", "en"));
        try { UpdatePolicy.FeedChannel("../x64", "stable"); throw new InvalidOperationException("Unsafe channel accepted"); }
        catch (ArgumentException) { }
    }
}
