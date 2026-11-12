using System.Diagnostics;
using System.Net.Http;
using System.Runtime.InteropServices;
using System.Security.Cryptography;

namespace BibCiTeX;

// Runs before loading WinUI. .NET itself is provisioned by the Velopack installer.
internal static class WindowsRuntime
{
    // Keep in sync with Microsoft.WindowsAppSDK.Runtime in the project file.
    internal const string Version = "2.6.5";
    private const uint MajorMinor = 0x00020006;
    private const ulong MinimumVersion = 0x0002000600050000;

    internal static (string Url, string Hash) Installer(Architecture architecture) => architecture switch
    {
        Architecture.X64 => (
            "https://download.microsoft.com/download/81f35163-9c63-4b4d-a293-6dcf1b32320a/WindowsAppRuntimeInstall-x64.exe",
            "908e9a8ed8ad2ac765d6679543b0758812437e3ad6e3223f08d1730635cec8ec"),
        Architecture.Arm64 => (
            "https://download.microsoft.com/download/1346f044-de9e-4d14-826c-98215cf54e6e/WindowsAppRuntimeInstall-arm64.exe",
            "4cafe57b5e03384a39cca9c6c0dbbbf837cf76ac828e66238cf6160ed2575b13"),
        _ => throw new PlatformNotSupportedException($"Unsupported architecture: {architecture}")
    };

    internal static void EnsureInstalled()
    {
        using var mutex = new Mutex(false, @"Local\BibCiTeX.WindowsAppRuntime.Install");
        var acquired = false;
        try
        {
            try { acquired = mutex.WaitOne(TimeSpan.FromMinutes(10)); }
            catch (AbandonedMutexException) { acquired = true; }
            if (!acquired) throw new TimeoutException("Windows App Runtime installation is still in progress.");
            Ensure(() => Initialize(MajorMinor, "", MinimumVersion, 0), Install);
        }
        finally { if (acquired) mutex.ReleaseMutex(); }
    }

    // An existing compatible runtime needs neither a download nor an installer invocation.
    internal static void Ensure(Func<int> initialize, Action install)
    {
        var result = initialize();
        if (result >= 0) return;
        // MddBootstrap reports this HRESULT when no compatible framework is registered.
        // Permission, corruption and unsupported-process failures must not trigger installation.
        if (result != unchecked((int)0x80670016)) Marshal.ThrowExceptionForHR(result);
        install();
        result = initialize();
        if (result < 0) Marshal.ThrowExceptionForHR(result);
    }

    private static void Install()
    {
        // The ARM64 installer also registers x64 framework packages for emulated apps.
        var installer = Installer(RuntimeInformation.OSArchitecture);
        var directory = Directory.CreateTempSubdirectory("BibCiTeX-runtime-");
        try
        {
            var path = Path.Combine(directory.FullName, "WindowsAppRuntimeInstall.exe");
            using (var client = new HttpClient { Timeout = TimeSpan.FromMinutes(5) })
            using (var response = client.GetAsync(installer.Url, HttpCompletionOption.ResponseHeadersRead).GetAwaiter().GetResult())
            {
                response.EnsureSuccessStatusCode();
                using var cancellation = new CancellationTokenSource(TimeSpan.FromMinutes(5));
                using var output = File.Create(path);
                response.Content.CopyToAsync(output, cancellation.Token).GetAwaiter().GetResult();
            }
            VerifyInstaller(path, installer.Hash);
            using var process = Process.Start(new ProcessStartInfo(path, "--quiet") { UseShellExecute = false, CreateNoWindow = true })
                ?? throw new IOException("Cannot start Windows App Runtime installer.");
            // Do not kill an in-flight system installation: wait for its final result.
            process.WaitForExit();
            if (process.ExitCode != 0)
                throw new InvalidOperationException($"Windows App Runtime installation failed (0x{process.ExitCode:X8}).");
        }
        finally
        {
            try { directory.Delete(true); }
            catch (IOException) { }
            catch (UnauthorizedAccessException) { }
        }
    }

    internal static void VerifyInstaller(string path, string expectedHash)
    {
        using var stream = File.OpenRead(path);
        if (!CryptographicOperations.FixedTimeEquals(SHA256.HashData(stream), Convert.FromHexString(expectedHash)))
            throw new InvalidDataException("Windows App Runtime download failed integrity verification.");
    }

    internal static void ShowError(Exception error)
    {
        var chinese = System.Globalization.CultureInfo.CurrentUICulture.Name.StartsWith("zh", StringComparison.OrdinalIgnoreCase);
        var message = chinese
            ? "无法安装或初始化 Windows App Runtime。请检查网络后重新启动 BibCiTeX。\n\n"
            : "Could not install or initialize Windows App Runtime. Check your connection and restart BibCiTeX.\n\n";
        MessageBox(0, message + error.Message, "BibCiTeX", 0x10);
    }

    [DllImport("Microsoft.WindowsAppRuntime.Bootstrap.dll", EntryPoint = "MddBootstrapInitialize2", CharSet = CharSet.Unicode, ExactSpelling = true)]
    private static extern int Initialize(uint majorMinor, string versionTag, ulong minimumVersion, uint options);

    [DllImport("Microsoft.WindowsAppRuntime.Bootstrap.dll", EntryPoint = "MddBootstrapShutdown", ExactSpelling = true)]
    internal static extern void Shutdown();

    [DllImport("user32.dll", EntryPoint = "MessageBoxW", CharSet = CharSet.Unicode, ExactSpelling = true)]
    private static extern int MessageBox(nint owner, string text, string caption, uint type);
}
