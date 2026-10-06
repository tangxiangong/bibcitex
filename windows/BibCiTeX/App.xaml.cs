using Microsoft.UI.Xaml;
using Microsoft.Windows.AppLifecycle;

namespace BibCiTeX;

public partial class App : Application
{
    private MainWindow? main = null;
#if !LOCALIZATION_TESTS
    private AppInstance? instance;
    private Microsoft.UI.Dispatching.DispatcherQueueTimer? languageTimer;
#endif
    internal static TrayWindow? Tray { get; private set; }
    internal static HelperWindow? Helper { get; private set; }
    internal static ElementTheme Theme { get; private set; }
    public App()
    {
        // Read the user's language list, independently of our persisted native override.
        L10n.SystemLanguages = () => Windows.System.UserProfile.GlobalizationPreferences.Languages;
#if LOCALIZATION_TESTS
        var selection = "en";
#else
        var settings = NativeSettings.Values;
        var selection = settings.TryGetValue("language", out var value) ? value as string ?? "system" : "system";
#endif
        L10n.Changing += ApplyNativeLanguage;
        ApplyNativeLanguage(L10n.Resolve(selection, L10n.SystemLanguages()));
        L10n.Select(selection);
        InitializeComponent();
    }
    private static void ApplyNativeLanguage(string language)
    {
        // This must precede XAML resource loading, including at application startup.
        Microsoft.Windows.Globalization.ApplicationLanguages.PrimaryLanguageOverride = language == "zh-Hans" ? "zh-CN" : "en-US";
    }
    protected override async void OnLaunched(LaunchActivatedEventArgs args)
    {
#if LOCALIZATION_TESTS
        try
        {
            await NativeLocalizationTests.Run();
            System.IO.File.WriteAllText(System.IO.Path.Combine(AppContext.BaseDirectory, "localization-test-results.txt"), "PASS native WinUI localization bindings after GC and language round trips");
            Environment.ExitCode = 0;
        }
        catch (Exception error)
        {
            System.IO.File.WriteAllText(System.IO.Path.Combine(AppContext.BaseDirectory, "localization-test-results.txt"), error.ToString());
            Environment.ExitCode = 1;
        }
        Exit();
#else
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
        L10n.Changed += RefreshWindowLanguages;
        languageTimer = dispatcher.CreateTimer();
        languageTimer.Interval = TimeSpan.FromSeconds(2);
        languageTimer.Tick += (_, _) => L10n.RefreshSystemLanguage();
        languageTimer.Start();
        main.Closed += (_, _) => { languageTimer.Stop(); L10n.Changed -= RefreshWindowLanguages; };
        main.Activate(); main.InstallShortcut();
        if (initializationError is { } startupError)
            ((FrameworkElement)main.Content).Loaded += async (_, _) => await Views.Error((FrameworkElement)main.Content, startupError);
#endif
    }
    internal static void SetLanguage(string language)
    {
        NativeSettings.Values["language"] = language;
        L10n.Select(language);
    }
    private static void RefreshWindowLanguages()
    {
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
