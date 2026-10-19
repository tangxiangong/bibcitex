using System.Security.Cryptography;
using System.Text.Json;
using BibCiTeX;
using Velopack;
using Velopack.Logging;
using Velopack.Sources;

internal static class UpdateFeedTests
{
    private static byte[] Feed(string version = "1.1.0-beta.1", string arch = "x64", string appId = "BibCiTeX", string type = "Full", int count = 1)
        => JsonSerializer.SerializeToUtf8Bytes(new
        {
            Architecture = arch,
            Assets = Enumerable.Repeat(new
            {
                PackageId = appId, Version = version, Type = type,
                FileName = $"BibCiTeX-{version}-win-{arch}-{UpdatePolicy.DefaultChannel(version)}-full.nupkg",
                Size = 4, SHA256 = Convert.ToHexString(SHA256.HashData(new byte[] { 1, 2, 3, 4 })),
                NotesMarkdown = "<!-- locale:zh-Hans -->\n中文\n<!-- /locale -->\n<!-- locale:en -->\nEnglish\n<!-- /locale -->"
            }, count)
        });

    private static byte[] Seal(byte[] bytes, ECDsa key) => JsonSerializer.SerializeToUtf8Bytes(new
    {
        feed = Convert.ToBase64String(bytes),
        signature = Convert.ToBase64String(key.SignData(bytes, HashAlgorithmName.SHA256, DSASignatureFormat.IeeeP1363FixedFieldConcatenation))
    });

    internal static void Run()
    {
        static void Assert(bool condition, string name) { if (!condition) throw new InvalidOperationException(name); }
        static void Reject(Action action)
        {
            try { action(); }
            catch (InvalidDataException) { return; }
            throw new InvalidOperationException("Untrusted update metadata was accepted");
        }
        using var signer = ECDsa.Create(ECCurve.NamedCurves.nistP256);
        using var publicKey = UpdateFeed.ImportPublicKey(Convert.ToBase64String(signer.ExportSubjectPublicKeyInfo()))!;
        Assert(UpdateFeed.ImportPublicKey(null) is null && UpdateFeed.ImportPublicKey("invalid") is null, "Unconfigured keys must disable updates");
        using var wrongCurve = ECDsa.Create(ECCurve.NamedCurves.nistP384);
        Assert(UpdateFeed.ImportPublicKey(Convert.ToBase64String(wrongCurve.ExportSubjectPublicKeyInfo())) is null, "Only P-256 is supported");
        var envelope = Seal(Feed(), signer);
        using var original = JsonDocument.Parse(envelope);
        Reject(() => UpdateFeed.Open(JsonSerializer.SerializeToUtf8Bytes(new
        {
            feed = Convert.ToBase64String(Feed(version: "1.2.0")),
            signature = original.RootElement.GetProperty("signature").GetString()
        }), publicKey, "win-x64-beta"));
        Reject(() => UpdateFeed.Open(JsonSerializer.SerializeToUtf8Bytes(new
        {
            feed = original.RootElement.GetProperty("feed").GetString(), signature = "AA=="
        }), publicKey, "win-x64-beta"));
        var feed = UpdateFeed.Open(envelope, publicKey, "win-x64-beta");
        Assert(feed.Assets.Single().Version.ToString() == "1.1.0-beta.1", "Signed previews must preserve their version");
        Assert(UpdatePolicy.Notes(feed.Assets[0].NotesMarkdown!, "zh-Hans") == "中文", "Signing must preserve localized notes");
        Reject(() => UpdateFeed.Open(envelope, publicKey, "win-x64-stable"));
        Reject(() => UpdateFeed.Open(envelope, publicKey, "win-arm64-beta"));
        Reject(() => UpdateFeed.Open(Seal(Feed(appId: "Other"), signer), publicKey, "win-x64-beta"));
        Reject(() => UpdateFeed.Open(Seal(Feed(type: "Delta"), signer), publicKey, "win-x64-beta"));
        Reject(() => UpdateFeed.Open(Seal(Feed(count: 2), signer), publicKey, "win-x64-beta"));
        Reject(() => UpdateFeed.Open(Seal(Feed(version: "1.0.0-rc.1"), signer), publicKey, "win-x64-alpha"));
        Reject(() => UpdateFeed.Open("{}"u8.ToArray(), publicKey, "win-x64-beta"));
        using var other = ECDsa.Create(ECCurve.NamedCurves.nistP256);
        Reject(() => UpdateFeed.Open(envelope, other, "win-x64-beta"));
        var stable = Seal(Feed(version: "1.1.0"), signer);
        foreach (var channel in new[] { "stable", "beta", "alpha" })
            Assert(UpdateFeed.Open(stable, publicKey, $"win-x64-{channel}").Assets.Length == 1, "Stable releases must promote to all audiences");
        Assert(UpdateFeed.Open(Seal(Feed(count: 0), signer), publicKey, "win-x64-beta").Assets.Length == 0, "A signed empty feed remains valid");
        foreach (var name in new[] { "../evil.nupkg", "https://host/evil.nupkg", "%2f.nupkg", "bad?x.nupkg", "setup.exe" })
            Reject(() => UpdateFeed.RequireSafeFileName(name));
        var primary = new Downloader(envelope);
        var backup = new Downloader(envelope);
        var sources = new IUpdateSource[] { new SignedWebSource("https://primary", publicKey, primary), new SignedWebSource("https://backup", publicKey, backup) };
        var source = new UpdateSource(sources, sources);
        source.GetReleaseFeed(null!, "BibCiTeX", "win-x64-beta").GetAwaiter().GetResult();
        Assert(primary.LastUrl == "https://primary/releases.win-x64-beta.signed.json" && backup.Checks == 0, "Primary signed feed is preferred");
        primary.Bytes = "{}"u8.ToArray();
        source.GetReleaseFeed(null!, "BibCiTeX", "win-x64-beta").GetAwaiter().GetResult();
        Assert(backup.Checks == 1, "Primary failure must use a verified fallback");
        backup.Bytes = Seal(Feed(), other);
        Reject(() => source.GetReleaseFeed(null!, "BibCiTeX", "win-x64-beta").GetAwaiter().GetResult());
        Console.WriteLine("Signed feed checks passed.");
    }

    // Ephemeral fixtures let the host verify the Rust publisher against the actual C# reader.
    internal static void ExportFixture(string directory)
    {
        Directory.CreateDirectory(directory);
        using var key = ECDsa.Create(ECCurve.NamedCurves.nistP256);
        File.WriteAllText(Path.Combine(directory, "public.txt"), Convert.ToBase64String(key.ExportSubjectPublicKeyInfo()));
        File.WriteAllText(Path.Combine(directory, "private.txt"), Convert.ToBase64String(key.ExportPkcs8PrivateKey()));
        foreach (var arch in new[] { "x64", "arm64" }) File.WriteAllBytes(Path.Combine(directory, $"releases.win-{arch}-beta.json"), Feed(arch: arch));
    }
    internal static void VerifyFixture(string directory)
    {
        using var key = UpdateFeed.ImportPublicKey(File.ReadAllText(Path.Combine(directory, "public.txt")))!;
        foreach (var arch in new[] { "x64", "arm64" })
            UpdateFeed.Open(File.ReadAllBytes(Path.Combine(directory, $"releases.win-{arch}-beta.signed.json")), key, $"win-{arch}-beta");
        Console.WriteLine("Rust publisher / C# reader interoperability passed for both architectures.");
    }

    private sealed class Downloader(byte[] bytes) : IFileDownloader
    {
        internal byte[] Bytes = bytes;
        internal string? LastUrl;
        internal int Checks;
        public Task<byte[]> DownloadBytes(string url, IDictionary<string, string>? headers = null, double timeout = 30)
        { LastUrl = url; Checks++; return Task.FromResult(Bytes); }
        public Task<string> DownloadString(string url, IDictionary<string, string>? headers = null, double timeout = 30) => throw new NotSupportedException();
        public Task DownloadFile(string url, string targetFile, Action<int> progress, IDictionary<string, string>? headers = null, double timeout = 30, CancellationToken cancelToken = default) => throw new NotSupportedException();
    }
}
