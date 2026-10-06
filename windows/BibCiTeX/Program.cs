using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Velopack;

namespace BibCiTeX;

internal static class Program
{
    [STAThread]
    private static void Main(string[] args)
    {
        // Installer hooks must finish before WinUI, single-instance registration or Rust startup.
#if !LOCALIZATION_TESTS
        VelopackApp.Build().SetAutoApplyOnStartup(false).Run();
        Updater.ApplyPending();
#endif
        WinRT.ComWrappersSupport.InitializeComWrappers();
        Application.Start(parameters =>
        {
            SynchronizationContext.SetSynchronizationContext(
                new DispatcherQueueSynchronizationContext(DispatcherQueue.GetForCurrentThread()));
            _ = new App();
        });
    }
}
