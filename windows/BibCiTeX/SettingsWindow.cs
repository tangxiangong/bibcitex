using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Windows.System;

namespace BibCiTeX;

internal sealed class SettingsWindow : Window
{
    private readonly StackPanel content = new() { Spacing = 20, Padding = new Thickness(20) };
    private bool closed;

    internal SettingsWindow()
    {
        Title = L10n.Text("设置");
        SystemBackdrop = Microsoft.UI.Composition.SystemBackdrops.MicaController.IsSupported() ? new MicaBackdrop() : new DesktopAcrylicBackdrop();
        Localized.BindValue(content, FrameworkElement.LanguageProperty, () => L10n.Language);

        var theme = Choice("主题", [("跟随系统", "Default"), ("浅色", "Light"), ("深色", "Dark")], App.Theme.ToString());
        theme.SelectionChanged += (_, _) =>
        {
            if (theme.SelectedItem is ComboBoxItem { Tag: string value }) App.SetTheme(Enum.Parse<ElementTheme>(value));
        };

        var language = Choice("语言", [("跟随系统", "system"), ("简体中文", "zh-Hans"), ("English", "en")], L10n.Selection);
        language.SelectionChanged += (_, _) =>
        {
            if (language.SelectedItem is ComboBoxItem { Tag: string value }) App.SetLanguage(value);
        };
        Section("外观", SettingRow("主题", theme), SettingRow("语言", language));

        var version = Views.Row(Views.LocalizedText("当前版本"), Views.Text(Updater.CurrentVersion));
        var channel = Choice("更新通道", [("正式版", "stable"), ("Beta", "beta"), ("Alpha", "alpha")], Updater.Channel);
        channel.SelectionChanged += (_, _) =>
        {
            if (channel.SelectedItem is ComboBoxItem { Tag: string value }) Updater.Channel = value;
        };
        var automatic = new ToggleSwitch { IsOn = Updater.AutomaticDownloads, OnContent = "", OffContent = "", MinWidth = 0, HorizontalAlignment = HorizontalAlignment.Right };
        Localized.Name(automatic, "自动下载并安装更新");
        automatic.Toggled += (_, _) => Updater.AutomaticDownloads = automatic.IsOn;
        var check = new Button { HorizontalAlignment = HorizontalAlignment.Right }; Localized.Bind(check, ContentControl.ContentProperty, "检查更新");
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
        Section("更新", SettingRow(version, check), SettingRow("更新通道", channel), SettingRow("自动下载并安装更新", automatic));
        Content = Views.AutoHideScrollbars(new ScrollViewer { Content = content, HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled });
        ApplyTheme();
        WindowInterop.ConfigureTitleBar(this);
        content.KeyDown += (_, args) => { if (args.Key == VirtualKey.Escape) { Close(); args.Handled = true; } };
        void Localize() { Title = L10n.Text("设置"); WindowInterop.LocalizeSystemMenu(this); }
        L10n.Changed += Localize;
        Closed += (_, _) => { closed = true; L10n.Changed -= Localize; };
        var scale = WindowInterop.GetDpiForWindow(WinRT.Interop.WindowNative.GetWindowHandle(this)) / 96.0;
        AppWindow.ResizeClient(new Windows.Graphics.SizeInt32((int)(480 * scale), (int)(400 * scale)));
        if (AppWindow.Presenter is Microsoft.UI.Windowing.OverlappedPresenter presenter) { presenter.IsResizable = false; presenter.IsMaximizable = false; }
        Localize();
    }

    internal void ApplyTheme() => ((FrameworkElement)Content).RequestedTheme = App.Theme;

    private void Section(string title, params UIElement[] controls)
    {
        var section = new StackPanel { Spacing = 8 };
        var heading = Views.LocalizedText(title, 13);
        heading.FontWeight = Microsoft.UI.Text.FontWeights.SemiBold;
        heading.Margin = new Thickness(4, 0, 0, 0);
        section.Children.Add(heading);
        var rows = new StackPanel();
        foreach (var control in controls)
        {
            if (rows.Children.Count > 0) rows.Children.Add(Views.Divider());
            rows.Children.Add(control);
        }
        var card = (Border)Microsoft.UI.Xaml.Markup.XamlReader.Load("""
            <Border xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                Background="{ThemeResource CardBackgroundFillColorDefaultBrush}"
                BorderBrush="{ThemeResource CardStrokeColorDefaultBrush}"
                BorderThickness="1" CornerRadius="8" />
            """);
        card.Child = rows;
        section.Children.Add(card);
        content.Children.Add(section);
    }

    private static Grid SettingRow(string label, FrameworkElement control)
    {
        var text = Views.LocalizedText(label);
        text.TextWrapping = TextWrapping.Wrap;
        return SettingRow(text, control);
    }

    private static Grid SettingRow(FrameworkElement label, FrameworkElement control)
    {
        var row = new Grid { MinHeight = 52, Padding = new Thickness(14, 8, 14, 8), ColumnSpacing = 16 };
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        label.VerticalAlignment = VerticalAlignment.Center;
        control.VerticalAlignment = VerticalAlignment.Center;
        row.Children.Add(label);
        Grid.SetColumn(control, 1);
        row.Children.Add(control);
        return row;
    }

    private static ComboBox Choice(string label, (string Label, string Value)[] options, string selected)
    {
        var choice = new ComboBox { Width = 156, HorizontalAlignment = HorizontalAlignment.Right };
        Localized.Name(choice, label);
        foreach (var (text, value) in options)
        {
            var item = Localized.ComboItem(text, value); choice.Items.Add(item);
            if (value == selected) choice.SelectedItem = item;
        }
        return choice;
    }
}
