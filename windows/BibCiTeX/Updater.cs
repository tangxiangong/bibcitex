using System.Reflection;
using System.Runtime.InteropServices;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Velopack;

namespace BibCiTeX;

internal static class Updater
{
    internal static string CurrentVersion => typeof(App).Assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion.Split('+')[0] ?? "0.6.0";
    internal static bool CanCheck => true;
    private static bool busy;
    private static long preferencesRevision;
    private static CancellationTokenSource? backgroundDownload;
    private static bool started;
    private static string? announced;
    private static readonly string Architecture = RuntimeInformation.ProcessArchitecture == System.Runtime.InteropServices.Architecture.Arm64 ? "arm64" : "x64";
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
    internal static bool AutomaticDownloads
    {
        get => NativeSettings.Values["automaticUpdates"] as string == "True";
        set { NativeSettings.Values["automaticUpdates"] = value.ToString(); PreferencesChanged(); }
    }
    private static void PreferencesChanged()
    {
        preferencesRevision++;
        backgroundDownload?.Cancel();
        NativeSettings.Values["pendingUpdate"] = "";
        announced = null;
    }
    internal static void ApplyPending()
    {
        var requested = NativeSettings.Values["pendingUpdate"] as string;
        if (string.IsNullOrEmpty(requested)) return;
        try
        {
            var manager = new UpdateManager(UpdatePolicy.BaseUrl);
            var pending = manager.IsInstalled ? manager.UpdatePendingRestart : null;
            if (UpdatePolicy.CanApplyPending(AutomaticDownloads, requested, pending?.Version.ToString(), Channel))
                manager.ApplyUpdatesAndRestart(pending);
        }
        catch (Exception error) { System.Diagnostics.Trace.TraceError("Pending update failed: {0}", error); }
        // A stale marker must never authorize a different cached package later.
        NativeSettings.Values["pendingUpdate"] = "";
    }
    internal static void AddSettings(MenuBarItem menu)
    {
        var channels = new MenuFlyoutSubItem(); Localized.Bind(channels, MenuFlyoutSubItem.TextProperty, "更新通道");
        foreach (var channel in new[] { "stable", "beta", "alpha" })
        {
            var item = new RadioMenuFlyoutItem { GroupName = "UpdateChannel", IsChecked = channel == Channel };
            Localized.Bind(item, MenuFlyoutItem.TextProperty, channel == "stable" ? "正式版" : channel == "beta" ? "Beta" : "Alpha");
            item.Click += (_, _) => Channel = channel; channels.Items.Add(item);
        }
        menu.Items.Add(channels);
        var automatic = new ToggleMenuFlyoutItem { IsChecked = AutomaticDownloads };
        Localized.Bind(automatic, MenuFlyoutItem.TextProperty, "自动下载并安装更新");
        automatic.Click += (_, _) => AutomaticDownloads = automatic.IsChecked;
        menu.Items.Add(automatic);
    }
    internal static async Task Start(FrameworkElement owner)
    {
        if (started) return;
        started = true;
        await Task.Delay(TimeSpan.FromSeconds(8));
        while (true)
        {
            try { await Check(owner, false); }
            catch (OperationCanceledException) { }
            catch (Exception error) { System.Diagnostics.Trace.TraceError("Update check failed: {0}", error); }
            await Task.Delay(TimeSpan.FromHours(6));
        }
    }
    internal static async Task Check(FrameworkElement owner, bool manual = true)
    {
        if (busy) return;
        busy = true;
        try
        {
            var requestedChannel = Channel;
            var revision = preferencesRevision;
            var manager = new UpdateManager(UpdatePolicy.BaseUrl, new UpdateOptions
            {
                ExplicitChannel = UpdatePolicy.FeedChannel(Architecture, requestedChannel),
                AllowVersionDowngrade = false,
                MaximumDeltasBeforeFallback = -1
            });
            if (!manager.IsInstalled)
            {
                if (manual) throw new LocalizedException("请先安装发布版本再检查更新");
                return;
            }
            var update = await manager.CheckForUpdatesAsync();
            if (revision != preferencesRevision) return;
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
                    if (UpdatePolicy.CanSchedule(revision, preferencesRevision, AutomaticDownloads))
                    {
                        NativeSettings.Values["pendingUpdate"] = version;
                        announced = version;
                    }
                }
                finally { backgroundDownload = null; }
                return;
            }
            var notes = update.TargetFullRelease.NotesMarkdown ?? "";
            var content = new StackPanel { Spacing = 12, MinWidth = 420 };
            content.Children.Add(new TextBlock { Text = version, FontSize = 20 });
            var scroll = new ScrollViewer { MaxHeight = 380, HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled };
            void RefreshNotes() => scroll.Content = UpdateMarkdown.Render(UpdatePolicy.Notes(notes, L10n.Language));
            RefreshNotes(); content.Children.Add(scroll);
            var progress = new ProgressBar { Minimum = 0, Maximum = 100, Visibility = Visibility.Collapsed };
            content.Children.Add(progress);
            var errorBar = new InfoBar { Severity = InfoBarSeverity.Error, IsClosable = false }; content.Children.Add(errorBar);
            var dialog = Localized.Dialog(new ContentDialog { XamlRoot = owner.XamlRoot, Content = content }, "检查更新");
            Localized.Bind(dialog, ContentDialog.PrimaryButtonTextProperty, "下载并安装");
            using var cancellation = new CancellationTokenSource();
            bool downloading = false;
            Task? installation = null;
            async Task Install()
            {
                downloading = true; dialog.IsPrimaryButtonEnabled = false;
                progress.Visibility = Visibility.Visible; errorBar.IsOpen = false;
                try
                {
                    await manager.DownloadUpdatesAsync(update, value => owner.DispatcherQueue.TryEnqueue(() => progress.Value = value), cancelToken: cancellation.Token);
                    cancellation.Token.ThrowIfCancellationRequested();
                    if (revision != preferencesRevision) throw new LocalizedException("更新设置已更改，请重新检查更新");
                    manager.ApplyUpdatesAndRestart(update);
                }
                catch (OperationCanceledException) { }
                catch (Exception error) { Localized.Error(errorBar, InfoBar.MessageProperty, error); errorBar.IsOpen = true; }
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
        finally { busy = false; }
    }
}
