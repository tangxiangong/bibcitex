using System.Security.Cryptography;
using BibCiTeX;
using Velopack;
using Velopack.Logging;
using Velopack.Sources;

internal static class UpdateSourceTests
{
    internal static async Task Run()
    {
        var bytes = new byte[] { 1, 2, 3, 4 };
        var asset = new VelopackAsset { FileName = "BibCiTeX-win-x64-full.nupkg", Size = bytes.Length,
            SHA256 = Convert.ToHexString(SHA256.HashData(bytes)) };
        var feed = new VelopackAssetFeed { Assets = [asset] };
        var primary = new FakeSource(feed, bytes);
        var fallback = new FakeSource(feed, bytes);
        var source = new UpdateSource([primary, fallback], [primary, fallback]);
        void Assert(bool value, string name) { if (!value) throw new InvalidOperationException(name); }
        Assert(ReferenceEquals(await source.GetReleaseFeed(null!, "BibCiTeX", "win-x64-stable"), feed)
            && fallback.Checks == 0, "COS feed must be preferred");
        var empty = new FakeSource(new VelopackAssetFeed { Assets = [] }, bytes);
        await new UpdateSource([empty, fallback], [empty, fallback]).GetReleaseFeed(null!, "BibCiTeX", "win-x64-stable");
        Assert(fallback.Checks == 0, "Valid empty COS feed must not query GitHub");
        primary.FeedError = true;
        await source.GetReleaseFeed(null!, "BibCiTeX", "win-x64-stable");
        Assert(fallback.Checks == 1, "Feed error must select GitHub");
        var path = Path.Combine(Path.GetTempPath(), Guid.NewGuid() + ".nupkg");
        try
        {
            await source.DownloadReleaseEntry(null!, asset, path, _ => { });
            Assert(primary.Downloads == 0 && fallback.Downloads == 1, "GitHub feed must download from GitHub");
            primary.FeedError = false;
            await source.GetReleaseFeed(null!, "BibCiTeX", "win-x64-stable");
            primary.DownloadError = true;
            await source.DownloadReleaseEntry(null!, asset, path, _ => { });
            Assert(File.ReadAllBytes(path).SequenceEqual(bytes) && fallback.Downloads == 2,
                "Partial COS download must be replaced by identical GitHub package");
            primary.DownloadError = false;
            primary.Bytes = [9, 9, 9, 9];
            await source.DownloadReleaseEntry(null!, asset, path, _ => { });
            Assert(fallback.Downloads == 3, "Corrupted primary package must use verified fallback");
            using var cancellation = new CancellationTokenSource();
            primary.OnDownload = cancellation.Cancel;
            try { await source.DownloadReleaseEntry(null!, asset, path, _ => { }, cancellation.Token); throw new InvalidOperationException("Cancellation ignored"); }
            catch (OperationCanceledException) { }
            Assert(fallback.Downloads == 3, "Cancellation must not start fallback download");
            primary.OnDownload = null;
            fallback.Bytes = [8];
            try { await source.DownloadReleaseEntry(null!, asset, path, _ => { }); throw new InvalidOperationException("Corrupt fallback accepted"); }
            catch (InvalidDataException) { }
            fallback.FeedError = true;
            primary.FeedError = true;
            try { await source.GetReleaseFeed(null!, "BibCiTeX", "win-x64-stable"); throw new InvalidOperationException("Both-source failure was hidden"); }
            catch (HttpRequestException) { }
        }
        finally { File.Delete(path); }
        Console.WriteLine("Update source transport checks passed.");
    }

    private sealed class FakeSource(VelopackAssetFeed feed, byte[] bytes) : IUpdateSource
    {
        internal bool FeedError;
        internal bool DownloadError;
        internal int Checks;
        internal int Downloads;
        internal byte[] Bytes = bytes;
        internal Action? OnDownload;
        public Task<VelopackAssetFeed> GetReleaseFeed(IVelopackLogger logger, string? appId, string channel,
            Guid? stagingId = null, VelopackAsset? latestLocalRelease = null)
        {
            Checks++;
            return FeedError ? Task.FromException<VelopackAssetFeed>(new HttpRequestException("offline")) : Task.FromResult(feed);
        }
        public Task DownloadReleaseEntry(IVelopackLogger logger, VelopackAsset releaseEntry,
            string localFile, Action<int> progress, CancellationToken cancelToken = default)
        {
            Downloads++;
            File.WriteAllBytes(localFile, Bytes);
            OnDownload?.Invoke();
            cancelToken.ThrowIfCancellationRequested();
            return DownloadError ? Task.FromException(new HttpRequestException("interrupted")) : Task.CompletedTask;
        }
    }
}
