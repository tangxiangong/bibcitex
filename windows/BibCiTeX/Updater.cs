using System.Net;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Velopack;

namespace BibCiTeX;

internal static class Updater
{
    internal static string CurrentVersion => typeof(Updater).Assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion.Split('+')[0] ?? "0.8.2";
    private static readonly Lazy<ECDsa?> PublicKey = new(() => UpdateFeed.ImportPublicKey(typeof(Updater).Assembly
        .GetCustomAttributes<AssemblyMetadataAttribute>().FirstOrDefault(attribute => attribute.Key == "UpdatePublicKey")?.Value));
    private static readonly string Architecture = RuntimeInformation.ProcessArchitecture == System.Runtime.InteropServices.Architecture.Arm64 ? "arm64" : "x64";
    private static bool busy, started;
    private static long preferencesRevision;
    private static CancellationTokenSource? backgroundDownload;
    private static string? announced;
    internal static event Action? Changed;
    internal static string? UnavailableReason { get; private set; }
    internal static bool CanCheck => started && UnavailableReason is null && !busy && !UpdateActivity.IsBusy;
    internal static string? PendingVersion => NativeSettings.Values["pendingUpdate"] as string is { Length: > 0 } value ? value : null;
    internal static string Channel
    {
        get
        {
            var saved = NativeSettings.Values["updateChannel"] as string;
            var channel = UpdatePolicy.Channel(saved, CurrentVersion);
            if (saved != channel) NativeSettings.Values["updateChannel"] = channel;
            return channel;
        }
        set { NativeSettings.Values["updateChannel"] = UpdatePolicy.Channel(value, CurrentVersion); PreferencesChanged(); }
    }
    internal static bool AutomaticChecks
    {
        get => NativeSettings.Values["automaticUpdateChecks"] as string != "False";
        set { NativeSettings.Values["automaticUpdateChecks"] = value.ToString(); PreferencesChanged(); }
    }
    internal static bool AutomaticDownloads
    {
        get => NativeSettings.Values["automaticUpdates"] as string == "True";
        set { NativeSettings.Values["automaticUpdates"] = value.ToString(); PreferencesChanged(); }
    }
    private static DateTimeOffset? LastCheck
    {
        get => DateTimeOffset.TryParse(NativeSettings.Values["lastUpdateCheck"] as string, out var value) ? value : null;
        set => NativeSettings.Values["lastUpdateCheck"] = value?.ToString("O", System.Globalization.CultureInfo.InvariantCulture) ?? "";
    }
    private static void PreferencesChanged()
    {
        preferencesRevision++;
        backgroundDownload?.Cancel();
        NativeSettings.Values["pendingUpdate"] = "";
        announced = null;
        Changed?.Invoke();
    }
    private static UpdateManager? CreateManager(out string? unavailable)
    {
        if (PublicKey.Value is not { } key) { unavailable = "此构建未配置更新验证公钥"; return null; }
        try
        {
            var manager = new UpdateManager(new UpdateSource(key), new UpdateOptions
            {
                ExplicitChannel = UpdatePolicy.FeedChannel(Architecture, Channel),
                AllowVersionDowngrade = false,
                MaximumDeltasBeforeFallback = -1
            });
            unavailable = manager.IsInstalled ? null : "请先安装发布版本再检查更新";
            return manager.IsInstalled ? manager : null;
        }
        catch (Exception error)
        {
            System.Diagnostics.Trace.TraceError("Update manager unavailable: {0}", error);
            unavailable = "自动更新不可用";
            return null;
        }
    }
    internal static void ApplyPending()
    {
        var requested = PendingVersion;
        if (requested is null) return;
        NativeSettings.Values["pendingUpdate"] = "";
        try
        {
            var manager = CreateManager(out _);
            var pending = manager?.UpdatePendingRestart;
            if (pending is not null && manager!.CurrentVersion is { } current && pending.Version.CompareTo(current) > 0
                && UpdatePolicy.CanApplyPending(AutomaticDownloads, requested, pending.Version.ToString(), Channel))
                manager!.ApplyUpdatesAndRestart(pending, Environment.GetCommandLineArgs()[1..]);
        }
        catch (Exception error) { System.Diagnostics.Trace.TraceError("Pending update failed: {0}", error); }
    }
    internal static void ApplyOnExit()
    {
        if (UpdateActivity.IsBusy || PendingVersion is not { } requested) return;
        try
        {
            var manager = CreateManager(out _);
            var pending = manager?.UpdatePendingRestart;
            if (pending is null || manager!.CurrentVersion is not { } current || pending.Version.CompareTo(current) <= 0) return;
            if (!UpdatePolicy.CanApplyPending(AutomaticDownloads, requested, pending?.Version.ToString(), Channel)) return;
            manager!.WaitExitThenApplyUpdates(pending, silent: true, restart: false);
            // Installation finishes after this process exits. The next launch consumes the marker,
            // retrying a still-pending package if the external installer failed.
        }
        catch (Exception error) { System.Diagnostics.Trace.TraceError("Update on exit failed: {0}", error); }
    }
    internal static async Task Start(FrameworkElement owner)
    {
        if (started) return;
        started = true;
        CreateManager(out var reason); UnavailableReason = reason;
        var dispatcher = owner.DispatcherQueue;
        UpdateActivity.Changed += () => dispatcher.TryEnqueue(() => Changed?.Invoke());
        Changed?.Invoke();
        if (reason is not null) return;
        await Task.Delay(TimeSpan.FromSeconds(8));
        while (true)
        {
            var delay = UpdatePolicy.CheckDelay(DateTimeOffset.UtcNow, LastCheck);
            if (delay > TimeSpan.Zero) await Task.Delay(delay);
            else if (!AutomaticChecks || !CanCheck) await Task.Delay(TimeSpan.FromMinutes(1));
            else
            {
                try { await Check(owner, false); }
                catch (OperationCanceledException) { }
                catch (Exception error) { System.Diagnostics.Trace.TraceError("Update check failed: {0}", error); }
            }
        }
    }
    internal static async Task Check(FrameworkElement owner, bool manual = true)
    {
        if (busy || UpdateActivity.IsBusy || (!manual && !AutomaticChecks)) return;
        var manager = CreateManager(out var reason); UnavailableReason = reason;
        if (manager is null) { Changed?.Invoke(); return; }
        busy = true; Changed?.Invoke();
        LastCheck = DateTimeOffset.UtcNow;
        try
        {
            var revision = preferencesRevision;
            var update = await manager.CheckForUpdatesAsync();
            if ((manual && !owner.IsLoaded) || revision != preferencesRevision) return;
            if (update is null)
            {
                if (manual)
                {
                    var latest = Localized.Dialog(new ContentDialog { XamlRoot = owner.XamlRoot }, "检查更新");
                    Localized.Bind(latest, ContentControl.ContentProperty, "当前已是最新版本"); await Views.ShowDialog(latest);
                }
                return;
            }
            var version = update.TargetFullRelease.Version.ToString();
            if (!manual && announced == version) return;
            if (!manual && AutomaticDownloads)
            {
                using var download = new CancellationTokenSource();
                backgroundDownload = download;
                try
                {
                    await manager.DownloadUpdatesAsync(update, cancelToken: download.Token);
                    if (!download.IsCancellationRequested && AutomaticChecks
                        && UpdatePolicy.CanSchedule(revision, preferencesRevision, AutomaticDownloads))
                    {
                        NativeSettings.Values["pendingUpdate"] = version;
                        announced = version; Changed?.Invoke();
                    }
                }
                finally { backgroundDownload = null; }
                return;
            }
            if (!owner.IsLoaded) return;
            var notes = update.TargetFullRelease.NotesMarkdown ?? "";
            var content = new StackPanel { Spacing = 12, MinWidth = 420 };
            var identity = new TextBlock { TextWrapping = TextWrapping.Wrap };
            Localized.Bind(identity, TextBlock.TextProperty, "BibCiTeX {0} 现已推出，当前版本为 {1}。", version, CurrentVersion);
            content.Children.Add(identity);
            var scroll = new ScrollViewer { MaxHeight = 380, HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled };
            void RefreshNotes() => scroll.Content = UpdateMarkdown.Render(UpdatePolicy.Notes(notes, L10n.Language));
            RefreshNotes(); content.Children.Add(scroll);
            var releaseLink = new HyperlinkButton { NavigateUri = new Uri($"https://github.com/tangxiangong/bibcitex/releases/tag/v{version}"), Padding = new Thickness(0) };
            Localized.Bind(releaseLink, ContentControl.ContentProperty, "在 GitHub 查看发布说明"); content.Children.Add(releaseLink);
            var progress = new ProgressBar { Minimum = 0, Maximum = 100, Visibility = Visibility.Collapsed };
            content.Children.Add(progress);
            var status = new TextBlock { TextWrapping = TextWrapping.Wrap, Visibility = Visibility.Collapsed }; content.Children.Add(status);
            var errorBar = new InfoBar { Severity = InfoBarSeverity.Error, IsClosable = false }; content.Children.Add(errorBar);
            var dialog = Localized.Dialog(new ContentDialog { XamlRoot = owner.XamlRoot, Content = content }, "检查更新");
            Localized.Bind(dialog, ContentDialog.PrimaryButtonTextProperty, "下载并安装");
            using var cancellation = new CancellationTokenSource();
            bool downloading = false;
            Task? installation = null;
            async Task Install()
            {
                downloading = true; dialog.IsPrimaryButtonEnabled = false;
                progress.Value = 0; progress.IsIndeterminate = false;
                progress.Visibility = status.Visibility = Visibility.Visible; errorBar.IsOpen = false;
                Localized.Bind(status, TextBlock.TextProperty, "正在下载…");
                try
                {
                    await manager.DownloadUpdatesAsync(update, value => owner.DispatcherQueue.TryEnqueue(() => progress.Value = value), cancelToken: cancellation.Token);
                    if (UpdateActivity.IsBusy)
                    {
                        Localized.Bind(status, TextBlock.TextProperty, "将在当前操作完成后重新启动…"); progress.IsIndeterminate = true;
                    }
                    while (UpdateActivity.IsBusy) await Task.Delay(TimeSpan.FromSeconds(1), cancellation.Token);
                    cancellation.Token.ThrowIfCancellationRequested();
                    if (revision != preferencesRevision) throw new LocalizedException("更新设置已更改，请重新检查更新");
                    Localized.Bind(status, TextBlock.TextProperty, "正在重新启动…");
                    NativeSettings.Values["pendingUpdate"] = "";
                    manager.ApplyUpdatesAndRestart(update);
                }
                catch (OperationCanceledException) when (cancellation.IsCancellationRequested) { }
                catch (Exception error)
                {
                    progress.Visibility = status.Visibility = Visibility.Collapsed;
                    Localized.Error(errorBar, InfoBar.MessageProperty, Describe(error)); errorBar.IsOpen = true;
                }
                finally { downloading = false; dialog.IsPrimaryButtonEnabled = true; }
            }
            dialog.PrimaryButtonClick += (_, args) =>
            {
                args.Cancel = true;
                if (!downloading) installation = Install();
            };
            dialog.CloseButtonClick += (_, _) => cancellation.Cancel();
            L10n.Changed += RefreshNotes;
            try { announced = version; await Views.ShowDialog(dialog); }
            finally
            {
                L10n.Changed -= RefreshNotes; cancellation.Cancel();
                if (installation is not null) await installation;
            }
        }
        catch (Exception error) when (manual)
        {
            if (owner.IsLoaded) await Views.Error(owner, Describe(error));
        }
        finally { busy = false; Changed?.Invoke(); }
    }
    private static Exception Describe(Exception error)
    {
        System.Diagnostics.Trace.TraceError("Update failed: {0}", error);
        return error switch
        {
            HttpRequestException { StatusCode: HttpStatusCode.NotFound } => new LocalizedException("暂时无法获取更新信息，请稍后重试。"),
            HttpRequestException or TaskCanceledException => new LocalizedException("无法连接到更新服务器，请检查网络后重试。"),
            InvalidDataException or CryptographicException => new LocalizedException("更新文件验证失败，请稍后重试。"),
            _ => error
        };
    }
}
