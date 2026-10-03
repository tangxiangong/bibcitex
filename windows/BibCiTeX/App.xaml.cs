using Microsoft.UI.Xaml;
using Microsoft.Windows.AppLifecycle;

namespace BibCiTeX;

public partial class App : Application
{
    private MainWindow? main;
    private AppInstance? instance;
    internal static HelperWindow? Helper { get; private set; }
    internal static ElementTheme Theme { get; private set; }
    public App() => InitializeComponent();
    protected override async void OnLaunched(LaunchActivatedEventArgs args)
    {
        instance = AppInstance.FindOrRegisterForKey("BibCiTeX");
        if (!instance.IsCurrent)
        {
            await instance.RedirectActivationToAsync(AppInstance.GetCurrent().GetActivatedEventArgs());
            Exit();
            return;
        }
        var dispatcher = Microsoft.UI.Dispatching.DispatcherQueue.GetForCurrentThread();
        instance.Activated += (_, _) => dispatcher.TryEnqueue(() => { main?.AppWindow.Show(); main?.Activate(); });
        Theme = Windows.Storage.ApplicationData.Current.LocalSettings.Values.TryGetValue("theme", out var value)
            && Enum.TryParse<ElementTheme>(value as string, out var theme) ? theme : ElementTheme.Default;
        Exception? initializationError = null;
        try { await RustCore.Initialize(); } catch (Exception error) { initializationError = error; }
        main = new MainWindow(); Helper = new HelperWindow();
        main.Closed += (_, _) => { Helper.Dispose(); Helper.Close(); };
        main.Activate(); main.InstallShortcut();
        if (initializationError is { } startupError)
            ((FrameworkElement)main.Content).Loaded += async (_, _) => await Views.Error((FrameworkElement)main.Content, startupError);
    }
    internal static void SetTheme(ElementTheme theme)
    {
        Theme = theme;
        Windows.Storage.ApplicationData.Current.LocalSettings.Values["theme"] = theme.ToString();
        if (Current is App { main: { } window }) ((FrameworkElement)window.Content).RequestedTheme = theme;
        if (Helper is { } helper) ((FrameworkElement)helper.Content).RequestedTheme = theme;
    }
}
