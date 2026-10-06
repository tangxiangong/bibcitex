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
        VelopackApp.Build().SetAutoApplyOnStartup(false).Run();
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
