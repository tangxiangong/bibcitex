using System.Security.Cryptography;
using Velopack;
using Velopack.Logging;
using Velopack.Sources;

namespace BibCiTeX;

// Each check starts at COS. Downloads keep the selected feed's asset identity and hash.
internal sealed class UpdateSource : IUpdateSource
{
    private readonly IUpdateSource[] feeds;
    private readonly IUpdateSource[] downloads;
    private int selected;

    internal UpdateSource() : this(
        [new SimpleWebSource(UpdatePolicy.BaseUrl, timeout: 0.5), new SimpleWebSource(UpdatePolicy.FallbackUrl, timeout: 0.5)],
        [new SimpleWebSource(UpdatePolicy.BaseUrl), new SimpleWebSource(UpdatePolicy.FallbackUrl)]) { }

    internal UpdateSource(IUpdateSource[] feeds, IUpdateSource[] downloads)
    {
        this.feeds = feeds;
        this.downloads = downloads;
    }

    public async Task<VelopackAssetFeed> GetReleaseFeed(IVelopackLogger logger, string? appId, string channel,
        Guid? stagingId = null, VelopackAsset? latestLocalRelease = null)
    {
        selected = 0;
        try { return await feeds[0].GetReleaseFeed(logger, appId, channel, stagingId, latestLocalRelease); }
        catch (Exception error)
        {
            System.Diagnostics.Trace.TraceWarning("COS update feed failed; trying GitHub: {0}", error.Message);
            selected = 1;
            return await feeds[1].GetReleaseFeed(logger, appId, channel, stagingId, latestLocalRelease);
        }
    }

    public async Task DownloadReleaseEntry(IVelopackLogger logger, VelopackAsset releaseEntry,
        string localFile, Action<int> progress, CancellationToken cancelToken = default)
    {
        // Generated feeds use basenames. An absolute URL would bypass mirror selection.
        if (string.IsNullOrEmpty(releaseEntry.FileName) || releaseEntry.FileName is "." or ".."
            || releaseEntry.FileName.IndexOfAny(['/', '\\', ':']) >= 0)
            throw new InvalidDataException("Invalid update package filename");
        for (var index = selected; index < downloads.Length; index++)
        {
            cancelToken.ThrowIfCancellationRequested();
            try
            {
                await downloads[index].DownloadReleaseEntry(logger, releaseEntry, localFile, progress, cancelToken);
                using var file = File.OpenRead(localFile);
                var checksum = Convert.ToHexString(await SHA256.HashDataAsync(file, cancelToken));
                if (file.Length != releaseEntry.Size || !checksum.Equals(releaseEntry.SHA256, StringComparison.OrdinalIgnoreCase))
                    throw new InvalidDataException("Update package checksum mismatch");
                return;
            }
            catch (Exception error) when (index == 0 && !cancelToken.IsCancellationRequested)
            {
                System.Diagnostics.Trace.TraceWarning("COS update download failed; trying GitHub: {0}", error.Message);
                File.Delete(localFile);
                progress(0);
            }
        }
    }
}
