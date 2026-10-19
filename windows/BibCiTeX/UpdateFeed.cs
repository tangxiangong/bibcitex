using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using Velopack;

namespace BibCiTeX;

/// Authenticates the exact feed bytes before trusting package hashes or release notes.
internal static class UpdateFeed
{
    internal static ECDsa? ImportPublicKey(string? publicKey)
    {
        if (string.IsNullOrWhiteSpace(publicKey)) return null;
        var key = ECDsa.Create();
        try
        {
            var bytes = Convert.FromBase64String(publicKey.Trim());
            key.ImportSubjectPublicKeyInfo(bytes, out var read);
            if (read == bytes.Length && key.ExportParameters(false).Curve.Oid.Value == ECCurve.NamedCurves.nistP256.Oid.Value) return key;
        }
        catch (Exception error) when (error is FormatException or CryptographicException) { }
        key.Dispose();
        return null;
    }

    internal static string Architecture(string channel)
    {
        foreach (var arch in new[] { "x64", "arm64" })
            foreach (var audience in new[] { "stable", "beta", "alpha" })
                if (channel == UpdatePolicy.FeedChannel(arch, audience)) return arch;
        throw new InvalidDataException("Invalid update channel");
    }

    internal static VelopackAssetFeed Open(byte[] envelope, ECDsa key, string channel)
    {
        try
        {
            var architecture = Architecture(channel);
            using var document = JsonDocument.Parse(envelope);
            var feed = Convert.FromBase64String(document.RootElement.GetProperty("feed").GetString() ?? "");
            var signature = Convert.FromBase64String(document.RootElement.GetProperty("signature").GetString() ?? "");
            if (signature.Length != 64 || !key.VerifyData(feed, signature, HashAlgorithmName.SHA256, DSASignatureFormat.IeeeP1363FixedFieldConcatenation))
                throw new InvalidDataException("Invalid update feed signature");
            using var payload = JsonDocument.Parse(feed);
            if (payload.RootElement.GetProperty("Architecture").GetString() != architecture)
                throw new InvalidDataException("Update architecture mismatch");
            var parsed = VelopackAssetFeed.FromJson(Encoding.UTF8.GetString(feed));
            if (parsed?.Assets is not { Length: <= 1 } assets) throw new InvalidDataException("Invalid update package count");
            foreach (var asset in assets)
            {
                var version = asset.Version?.ToString() ?? "";
                if (asset.PackageId != "BibCiTeX" || asset.Type != VelopackAssetType.Full
                    || !UpdatePolicy.AcceptsVersion(version, channel.Split('-')[2]))
                    throw new InvalidDataException("Invalid update package identity or version");
                RequireSafeFileName(asset.FileName);
                if (asset.FileName != $"BibCiTeX-{version}-win-{architecture}-{UpdatePolicy.DefaultChannel(version)}-full.nupkg")
                    throw new InvalidDataException("Update package filename mismatch");
                if (asset.Size <= 0 || asset.SHA256 is not { Length: 64 } hash || !hash.All(Uri.IsHexDigit))
                    throw new InvalidDataException("Update package size or SHA-256 is missing");
            }
            return parsed;
        }
        catch (Exception error) when (error is JsonException or FormatException or KeyNotFoundException or InvalidOperationException or CryptographicException)
        {
            throw new InvalidDataException("Invalid signed update feed", error);
        }
    }

    internal static void RequireSafeFileName(string? name)
    {
        if (string.IsNullOrEmpty(name) || name.IndexOfAny(['/', '\\', ':', '?', '#', '%']) >= 0
            || name.Any(char.IsControl) || !name.EndsWith(".nupkg", StringComparison.OrdinalIgnoreCase))
            throw new InvalidDataException("Invalid update package filename");
    }
}
