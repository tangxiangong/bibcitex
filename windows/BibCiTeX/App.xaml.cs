using Microsoft.UI.Xaml;
using Microsoft.Windows.AppLifecycle;

namespace BibCiTeX;

public partial class App : Application
{
    private MainWindow? main;
    private AppInstance? instance;
    internal static TrayWindow? Tray { get; private set; }
    internal static HelperWindow? Helper { get; private set; }
    internal static ElementTheme Theme { get; private set; }
    public App()
    {
        // Read the user's language list, independently of our persisted native override.
        L10n.SystemLanguage = () => Windows.System.UserProfile.GlobalizationPreferences.Languages.FirstOrDefault() ?? System.Globalization.CultureInfo.InstalledUICulture.Name;
        var settings = NativeSettings.Values;
        var selection = settings.TryGetValue("language", out var value) ? value as string ?? "system" : "system";
        ApplyNativeLanguage(selection);
        L10n.Select(selection);
        InitializeComponent();
    }
    private static void ApplyNativeLanguage(string selection)
    {
        var language = L10n.Resolve(selection, L10n.SystemLanguage());
        // This must precede XAML resource loading, including at application startup.
        Microsoft.Windows.Globalization.ApplicationLanguages.PrimaryLanguageOverride = language == "zh-Hans" ? "zh-CN" : "en-US";
    }
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
        Theme = NativeSettings.Values.TryGetValue("theme", out var value)
            && Enum.TryParse<ElementTheme>(value as string, out var theme) ? theme : ElementTheme.Default;
        Exception? initializationError = null;
        try { await RustCore.Initialize(); } catch (Exception error) { initializationError = error; }
        main = new MainWindow(); Helper = new HelperWindow();
        Tray = new TrayWindow(() => { main.AppWindow.Show(); main.Activate(); });
        main.Closed += (_, _) => { Tray.Shutdown(); Helper.Dispose(); Helper.Close(); };
        ((FrameworkElement)main.Content).Loaded += (_, _) => _ = Updater.Start((FrameworkElement)main.Content);
        main.Activate(); main.InstallShortcut();
        if (initializationError is { } startupError)
            ((FrameworkElement)main.Content).Loaded += async (_, _) => await Views.Error((FrameworkElement)main.Content, startupError);
    }
    internal static void SetLanguage(string language)
    {
        NativeSettings.Values["language"] = language;
        ApplyNativeLanguage(language);
        L10n.Select(language);
        if (Current is App { main: { } window }) WindowInterop.LocalizeSystemMenu(window);
        if (Tray is { } tray) WindowInterop.LocalizeSystemMenu(tray);
        if (Helper is { } helper) WindowInterop.LocalizeSystemMenu(helper);
    }
    internal static void SetTheme(ElementTheme theme)
    {
        Theme = theme;
        NativeSettings.Values["theme"] = theme.ToString();
        if (Current is App { main: { } window }) ((FrameworkElement)window.Content).RequestedTheme = theme;
        if (Tray is { } tray) ((FrameworkElement)tray.Content).RequestedTheme = theme;
        if (Helper is { } helper) ((FrameworkElement)helper.Content).RequestedTheme = theme;
    }
}
