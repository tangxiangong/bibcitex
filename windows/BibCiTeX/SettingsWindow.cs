using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Windows.System;

namespace BibCiTeX;

internal sealed class SettingsWindow : Window
{
    private readonly StackPanel content = new() { Spacing = 20, Padding = new Thickness(24) };
    private bool closed;

    internal SettingsWindow()
    {
        Title = L10n.Text("设置");
        SystemBackdrop = Microsoft.UI.Composition.SystemBackdrops.MicaController.IsSupported() ? new MicaBackdrop() : new DesktopAcrylicBackdrop();
        content.RequestedTheme = App.Theme;
        Localized.BindValue(content, FrameworkElement.LanguageProperty, () => L10n.Language);

        var theme = Choice("主题", [("跟随系统", "Default"), ("浅色", "Light"), ("深色", "Dark")], App.Theme.ToString());
        theme.SelectionChanged += (_, _) =>
        {
            if (theme.SelectedItem is ComboBoxItem { Tag: string value }) App.SetTheme(Enum.Parse<ElementTheme>(value));
        };
        Section("外观", theme);

        var language = Choice("语言", [("跟随系统", "system"), ("简体中文", "zh-Hans"), ("English", "en")], L10n.Selection);
        language.SelectionChanged += (_, _) =>
        {
            if (language.SelectedItem is ComboBoxItem { Tag: string value }) App.SetLanguage(value);
        };
        Section("语言", language);

        var version = Views.Row(Views.LocalizedText("当前版本"), Views.Text(Updater.CurrentVersion));
        var channel = Choice("更新通道", [("正式版", "stable"), ("Beta", "beta"), ("Alpha", "alpha")], Updater.Channel);
        channel.SelectionChanged += (_, _) =>
        {
            if (channel.SelectedItem is ComboBoxItem { Tag: string value }) Updater.Channel = value;
        };
        var automatic = new ToggleSwitch { IsOn = Updater.AutomaticDownloads };
        Localized.Bind(automatic, ToggleSwitch.HeaderProperty, "自动下载并安装更新");
        automatic.Toggled += (_, _) => Updater.AutomaticDownloads = automatic.IsOn;
        var check = new Button(); Localized.Bind(check, ContentControl.ContentProperty, "检查更新");
        check.Click += async (_, _) =>
        {
            check.IsEnabled = false;
            try { await Updater.Check(content); }
            catch (Exception error)
            {
                if (!closed)
                {
                    try { await Views.Error(content, error); }
                    catch (Exception) when (closed) { }
                }
            }
            finally { if (!closed) check.IsEnabled = true; }
        };
        Section("更新", version, channel, automatic, check);
        Content = Views.AutoHideScrollbars(new ScrollViewer { Content = content, HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled });
        content.KeyDown += (_, args) => { if (args.Key == VirtualKey.Escape) { Close(); args.Handled = true; } };
        void Localize() { Title = L10n.Text("设置"); WindowInterop.LocalizeSystemMenu(this); }
        L10n.Changed += Localize;
        Closed += (_, _) => { closed = true; L10n.Changed -= Localize; };
        var scale = WindowInterop.GetDpiForWindow(WinRT.Interop.WindowNative.GetWindowHandle(this)) / 96.0;
        AppWindow.ResizeClient(new Windows.Graphics.SizeInt32((int)(480 * scale), (int)(640 * scale)));
        if (AppWindow.Presenter is Microsoft.UI.Windowing.OverlappedPresenter presenter) { presenter.IsResizable = false; presenter.IsMaximizable = false; }
        Localize();
    }

    internal void ApplyTheme() => content.RequestedTheme = App.Theme;

    private void Section(string title, params UIElement[] controls)
    {
        var section = new StackPanel { Spacing = 12 };
        section.Children.Add(Views.LocalizedText(title, 18));
        foreach (var control in controls) section.Children.Add(control);
        content.Children.Add(section);
    }

    private static ComboBox Choice(string label, (string Label, string Value)[] options, string selected)
    {
        var choice = new ComboBox { HorizontalAlignment = HorizontalAlignment.Stretch };
        Localized.Bind(choice, ComboBox.HeaderProperty, label);
        Localized.Name(choice, label);
        foreach (var (text, value) in options)
        {
            var item = Localized.ComboItem(text, value); choice.Items.Add(item);
            if (value == selected) choice.SelectedItem = item;
        }
        return choice;
    }
}
