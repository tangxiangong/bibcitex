using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Windows.ApplicationModel;
using Windows.Management.Deployment;

namespace BibCiTeX;

internal static class Updater
{
    internal static async Task Check(FrameworkElement owner)
    {
        var manager = new PackageManager();
        // Query a PackageManager-owned Package; Package.Current cannot run this check.
        var package = manager.FindPackageForUser(string.Empty, Package.Current.Id.FullName);
        var installer = package.GetAppInstallerInfo();
        if (installer is null) throw new LocalizedException("未配置 App Installer 更新源");
        var result = await package.CheckUpdateAvailabilityAsync();
        if (result.Availability is PackageUpdateAvailability.Available or PackageUpdateAvailability.Required)
        {
            // The Windows deployment service owns consent, signature validation and installation.
            var deployment = await manager.RequestAddPackageByAppInstallerFileAsync(installer.Uri, AddPackageByAppInstallerOptions.None, null);
            if (deployment.ExtendedErrorCode is { } error && error.HResult < 0) throw new InvalidOperationException(deployment.ErrorText, error);
        }
        else if (result.Availability == PackageUpdateAvailability.NoUpdates)
        {
            var dialog = Localized.Dialog(new ContentDialog { XamlRoot = owner.XamlRoot }, "检查更新");
            Localized.Bind(dialog, ContentControl.ContentProperty, "当前已是最新版本");
            await Views.ShowDialog(dialog);
        }
        else throw result.ExtendedError ?? new LocalizedException("无法检查更新");
    }
}
