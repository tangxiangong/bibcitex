using BibCiTeX;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Xml.Linq;

internal static class WindowsRuntimeTests
{
    internal static void Run()
    {
        static void Assert(bool value) { if (!value) throw new InvalidOperationException("Runtime bootstrap regression"); }
        using var project = typeof(WindowsRuntimeTests).Assembly.GetManifestResourceStream("BibCiTeX.Tests.AppProject")
            ?? throw new InvalidOperationException("Missing application dependency manifest");
        var packageVersion = XDocument.Load(project).Descendants("PackageReference")
            .Single(package => (string?)package.Attribute("Include") == "Microsoft.WindowsAppSDK.Runtime")
            .Attribute("Version")!.Value;
        if (WindowsRuntime.Version != packageVersion)
            throw new InvalidOperationException("Runtime bootstrap version must match Microsoft.WindowsAppSDK.Runtime");
        var version = System.Version.Parse(packageVersion);
        var majorMinor = (uint)typeof(WindowsRuntime).GetField("MajorMinor", BindingFlags.Static | BindingFlags.NonPublic)!.GetRawConstantValue()!;
        var minimumVersion = (ulong)typeof(WindowsRuntime).GetField("MinimumVersion", BindingFlags.Static | BindingFlags.NonPublic)!.GetRawConstantValue()!;
        Assert(majorMinor == ((uint)version.Major << 16 | (uint)version.Minor));
        Assert(minimumVersion == ((ulong)version.Major << 48 | (ulong)version.Minor << 32 | (ulong)version.Build << 16));
        var installs = 0;
        WindowsRuntime.Ensure(() => 0, () => installs++);
        Assert(installs == 0);
        var probes = 0;
        WindowsRuntime.Ensure(() => ++probes == 1 ? unchecked((int)0x80670016) : 0, () => installs++);
        Assert(probes == 2 && installs == 1);
        probes = 0;
        try
        {
            WindowsRuntime.Ensure(() => { probes++; return unchecked((int)0x80670016); }, () => throw new IOException("offline"));
            throw new InvalidOperationException("Installation failure was ignored");
        }
        catch (IOException) { Assert(probes == 1); }
        try
        {
            WindowsRuntime.Ensure(() => unchecked((int)0x80670016), () => { });
            throw new InvalidOperationException("Failed runtime initialization was ignored");
        }
        catch (COMException) { }
        installs = 0;
        probes = 0;
        try
        {
            WindowsRuntime.Ensure(() => { probes++; return unchecked((int)0x80070005); }, () => installs++);
            throw new InvalidOperationException("Access denied was ignored");
        }
        catch (UnauthorizedAccessException) { Assert(probes == 1 && installs == 0); }
        Assert(WindowsRuntime.Installer(Architecture.X64).Url.EndsWith("-x64.exe"));
        Assert(WindowsRuntime.Installer(Architecture.Arm64).Url.EndsWith("-arm64.exe"));
        try { WindowsRuntime.Installer(Architecture.X86); throw new InvalidOperationException("Unsupported architecture accepted"); }
        catch (PlatformNotSupportedException) { }
        var path = Path.GetTempFileName();
        try
        {
            File.WriteAllText(path, "installer fixture");
            var hash = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(path)));
            WindowsRuntime.VerifyInstaller(path, hash);
            File.AppendAllText(path, "tampered");
            try { WindowsRuntime.VerifyInstaller(path, hash); throw new InvalidOperationException("Corrupt installer accepted"); }
            catch (InvalidDataException) { }
        }
        finally { File.Delete(path); }
        Console.WriteLine("PASS runtime bootstrap: reuse, install, failure, architecture and integrity");
    }
}
