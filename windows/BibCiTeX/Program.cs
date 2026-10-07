using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Velopack;
using System.Runtime.CompilerServices;

namespace BibCiTeX;

internal static class Program
{
    [STAThread]
    private static void Main(string[] args)
    {
        // Installer hooks must finish before WinUI, single-instance registration or Rust startup.
        VelopackApp.Build().SetAutoApplyOnStartup(false).Run();
        var prepareRuntimeOnly = args.Length == 1 && args[0] == "--prepare-runtime";
        try { WindowsRuntime.EnsureInstalled(); }
        catch (Exception error)
        {
            if (prepareRuntimeOnly) Console.Error.WriteLine(error);
            else WindowsRuntime.ShowError(error);
            Environment.ExitCode = 1;
            return;
        }
        try { if (!prepareRuntimeOnly) StartApplication(); }
        finally { WindowsRuntime.Shutdown(); }
    }

    // Do not resolve WinUI types while JIT-compiling the runtime bootstrap entry point.
    [MethodImpl(MethodImplOptions.NoInlining)]
    private static void StartApplication()
    {
        Updater.ApplyPending();
        WinRT.ComWrappersSupport.InitializeComWrappers();
        Application.Start(parameters =>
        {
            SynchronizationContext.SetSynchronizationContext(
                new DispatcherQueueSynchronizationContext(DispatcherQueue.GetForCurrentThread()));
            _ = new App();
        });
    }
}
