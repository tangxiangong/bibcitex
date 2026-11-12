using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Media;
using Windows.System;

namespace BibCiTeX;

internal sealed class SettingsWindow : Window
{
    private readonly StackPanel content = new() { Spacing = 20, Padding = new Thickness(20) };
    private bool closed;
    private readonly ToggleButton shortcutRecorder = new() { MinWidth = 150 };
    private Action<string>? captureShortcut;

    internal bool CaptureCurrentShortcut()
    {
        if (shortcutRecorder.IsChecked != true) return false;
        captureShortcut?.Invoke(MainWindow.HelperShortcutValue);
        return true;
    }

    internal SettingsWindow(MainWindow owner)
    {
        Title = L10n.Text("设置");
        SystemBackdrop = Microsoft.UI.Composition.SystemBackdrops.MicaController.IsSupported() ? new MicaBackdrop() : new DesktopAcrylicBackdrop();
        Localized.BindValue(content, FrameworkElement.LanguageProperty, () => L10n.Language);

        var tray = new ToggleSwitch { IsOn = MainWindow.TrayIconEnabled, OnContent = "", OffContent = "", MinWidth = 0, HorizontalAlignment = HorizontalAlignment.Right };
        Localized.Name(tray, "显示托盘图标");
        Localized.Name(shortcutRecorder, "助手快捷键");
        Localized.Tooltip(shortcutRecorder, "录制快捷键");
        var resetShortcut = new Button(); Localized.Bind(resetShortcut, ContentControl.ContentProperty, "恢复默认");
        var desktopError = (TextBlock)Microsoft.UI.Xaml.Markup.XamlReader.Load("""
            <TextBlock xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                TextWrapping="Wrap" Margin="14,8" Visibility="Collapsed"
                Foreground="{ThemeResource SystemFillColorCriticalBrush}" />
            """);
        Exception? desktopFailure = null;
        void RefreshDesktopError()
        {
            desktopError.Text = desktopFailure is null ? "" : L10n.ErrorMessage(desktopFailure);
            desktopError.Visibility = desktopFailure is null ? Visibility.Collapsed : Visibility.Visible;
        }
        L10n.Changed += RefreshDesktopError;
        Closed += (_, _) => L10n.Changed -= RefreshDesktopError;
        var changingTray = false;
        tray.Toggled += (_, _) =>
        {
            if (changingTray) return;
            try { owner.SetTrayIconEnabled(tray.IsOn); desktopFailure = null; }
            catch (Exception error)
            {
                desktopFailure = error;
                changingTray = true;
                tray.IsOn = MainWindow.TrayIconEnabled;
                changingTray = false;
            }
            RefreshDesktopError();
        };
        void ApplyShortcut(string value)
        {
            shortcutRecorder.IsChecked = false;
            try { owner.SetHelperShortcut(value); desktopFailure = null; }
            catch (Exception error) { desktopFailure = error; }
            RefreshRecorder();
            RefreshDesktopError();
        }
        void RefreshRecorder() => shortcutRecorder.Content = shortcutRecorder.IsChecked == true
            ? L10n.Text("请按快捷键…") : HelperShortcut.Parse(MainWindow.HelperShortcutValue)!.DisplayValue;
        captureShortcut = ApplyShortcut;
        shortcutRecorder.Checked += (_, _) => RefreshRecorder();
        shortcutRecorder.Unchecked += (_, _) => RefreshRecorder();
        shortcutRecorder.LostFocus += (_, _) => shortcutRecorder.IsChecked = false;
        Activated += (_, args) => { if (args.WindowActivationState == WindowActivationState.Deactivated) shortcutRecorder.IsChecked = false; };
        L10n.Changed += RefreshRecorder;
        Closed += (_, _) => { shortcutRecorder.IsChecked = false; captureShortcut = null; L10n.Changed -= RefreshRecorder; };
        shortcutRecorder.PreviewKeyDown += (_, args) =>
        {
            if (shortcutRecorder.IsChecked != true) return;
            if (args.Key == VirtualKey.Tab) { shortcutRecorder.IsChecked = false; return; }
            args.Handled = true;
            if (args.Key == VirtualKey.Escape) { shortcutRecorder.IsChecked = false; return; }
            if (args.Key is VirtualKey.Control or VirtualKey.LeftControl or VirtualKey.RightControl
                or VirtualKey.Shift or VirtualKey.LeftShift or VirtualKey.RightShift
                or VirtualKey.Menu or VirtualKey.LeftMenu or VirtualKey.RightMenu
                or VirtualKey.LeftWindows or VirtualKey.RightWindows) return;
            bool Down(VirtualKey key) => Microsoft.UI.Input.InputKeyboardSource.GetKeyStateForCurrentThread(key)
                .HasFlag(Windows.UI.Core.CoreVirtualKeyStates.Down);
            uint modifiers = (Down(VirtualKey.Control) ? 2u : 0) | (Down(VirtualKey.Menu) ? 1u : 0)
                | (Down(VirtualKey.Shift) ? 4u : 0) | (Down(VirtualKey.LeftWindows) || Down(VirtualKey.RightWindows) ? 8u : 0);
            if (HelperShortcut.FromKey((uint)args.Key, modifiers) is { } shortcut) ApplyShortcut(shortcut.Value);
            else { desktopFailure = new LocalizedException("error.helperShortcutFormatWindows"); RefreshDesktopError(); }
        };
        RefreshRecorder();
        resetShortcut.Click += (_, _) => ApplyShortcut(HelperShortcut.DefaultValue);
        var shortcutControls = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8 };
        shortcutControls.Children.Add(shortcutRecorder); shortcutControls.Children.Add(resetShortcut);
        Section("通用", SettingRow("显示托盘图标", tray), SettingRow("助手快捷键", shortcutControls), desktopError);

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

        var family = CitationPreferences.Family(CitationPreferences.Format);
        var format = Choice("引用格式", [("key", "raw"), ("LaTeX", "latex"), ("Typst", "typst")], family);
        var command = Choice("引用指令", CitationPreferences.Commands(family), CitationPreferences.Format, CommandItem);
        var commandRow = SettingRow("引用指令", command);
        commandRow.Visibility = family == "raw" ? Visibility.Collapsed : Visibility.Visible;
        var mode = Choice("多选输出方式", [("合并", "merged"), ("逐项", "individual")], CitationPreferences.Mode);
        mode.IsEnabled = family != "raw";
        var updatingCommand = false;
        format.SelectionChanged += (_, _) =>
        {
            if (format.SelectedItem is not ComboBoxItem { Tag: string value } || value == CitationPreferences.Family(CitationPreferences.Format)) return;
            CitationPreferences.Format = value switch { "latex" => "cite", "typst" => "typst", _ => "raw" };
            updatingCommand = true;
            try
            {
                command.Items.Clear();
                foreach (var (label, id) in CitationPreferences.Commands(value))
                {
                    var item = CommandItem(label, id); command.Items.Add(item);
                    if (id == CitationPreferences.Format) command.SelectedItem = item;
                }
            }
            finally { updatingCommand = false; }
            commandRow.Visibility = value == "raw" ? Visibility.Collapsed : Visibility.Visible;
            mode.IsEnabled = value != "raw";
        };
        command.SelectionChanged += (_, _) => { if (!updatingCommand && command.SelectedItem is ComboBoxItem { Tag: string value }) CitationPreferences.Format = value; };
        mode.SelectionChanged += (_, _) => { if (mode.SelectedItem is ComboBoxItem { Tag: string value }) CitationPreferences.Mode = value; };
        Section("引用", SettingRow("引用格式", format), commandRow, SettingRow("多选输出方式", mode));

        var version = Views.Row(Views.LocalizedText("当前版本"), Views.Text(Updater.CurrentVersion));
        var channel = Choice("更新通道", [("正式版", "stable"), ("Beta", "beta"), ("Alpha", "alpha")], Updater.Channel);
        channel.SelectionChanged += (_, _) =>
        {
            if (channel.SelectedItem is ComboBoxItem { Tag: string value }) Updater.Channel = value;
        };
        var automatic = new ToggleSwitch { IsOn = Updater.AutomaticDownloads, OnContent = "", OffContent = "", MinWidth = 0, HorizontalAlignment = HorizontalAlignment.Right };
        Localized.Name(automatic, "自动下载并安装更新");
        automatic.Toggled += (_, _) => Updater.AutomaticDownloads = automatic.IsOn;
        var automaticChecks = new ToggleSwitch { IsOn = Updater.AutomaticChecks, OnContent = "", OffContent = "", MinWidth = 0, HorizontalAlignment = HorizontalAlignment.Right };
        Localized.Name(automaticChecks, "自动检查更新");
        automaticChecks.Toggled += (_, _) => Updater.AutomaticChecks = automaticChecks.IsOn;
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
            finally { if (!closed) check.IsEnabled = Updater.CanCheck; }
        };
        var updateStatus = new TextBlock { TextWrapping = TextWrapping.Wrap, Margin = new Thickness(14, 8, 14, 8) };
        void RefreshUpdates()
        {
            check.IsEnabled = Updater.CanCheck;
            channel.IsEnabled = automaticChecks.IsEnabled = Updater.UnavailableReason is null;
            automatic.IsEnabled = automaticChecks.IsEnabled && Updater.AutomaticChecks;
            updateStatus.Text = Updater.UnavailableReason is { } reason ? L10n.Text(reason)
                : Updater.PendingVersion is { } pending ? L10n.Text("BibCiTeX {0} 已下载，将在退出时安装。", pending) : "";
            updateStatus.Visibility = updateStatus.Text.Length == 0 ? Visibility.Collapsed : Visibility.Visible;
        }
        Updater.Changed += RefreshUpdates;
        L10n.Changed += RefreshUpdates;
        Closed += (_, _) => { Updater.Changed -= RefreshUpdates; L10n.Changed -= RefreshUpdates; };
        RefreshUpdates();
        Section("更新", SettingRow(version, check), SettingRow("更新通道", channel), SettingRow("自动检查更新", automaticChecks), SettingRow("自动下载并安装更新", automatic), updateStatus);
        Content = Views.AutoHideScrollbars(new ScrollViewer { Content = content, HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled });
        ApplyTheme();
        WindowInterop.ConfigureTitleBar(this);
        content.KeyDown += (_, args) => { if (args.Key == VirtualKey.Escape) { Close(); args.Handled = true; } };
        void Localize() { Title = L10n.Text("设置"); WindowInterop.LocalizeSystemMenu(this); }
        L10n.Changed += Localize;
        Closed += (_, _) => { closed = true; L10n.Changed -= Localize; };
        var scale = WindowInterop.GetDpiForWindow(WinRT.Interop.WindowNative.GetWindowHandle(this)) / 96.0;
        AppWindow.ResizeClient(new Windows.Graphics.SizeInt32((int)(480 * scale), (int)(560 * scale)));
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
            if (rows.Children.Count > 0)
            {
                var divider = Views.Divider();
                divider.SetBinding(UIElement.VisibilityProperty, new Microsoft.UI.Xaml.Data.Binding { Source = control, Path = new PropertyPath("Visibility") });
                rows.Children.Add(divider);
            }
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

    private static ComboBox Choice(string label, (string Label, string Value)[] options, string selected,
        Func<string, string, ComboBoxItem>? itemFactory = null)
    {
        var choice = new ComboBox { Width = 128, HorizontalAlignment = HorizontalAlignment.Right };
        Localized.Name(choice, label);
        foreach (var (text, value) in options)
        {
            var item = itemFactory?.Invoke(text, value) ?? Localized.ComboItem(text, value); choice.Items.Add(item);
            if (value == selected) choice.SelectedItem = item;
        }
        return choice;
    }

    private static ComboBoxItem CommandItem(string label, string value)
    {
        var item = Localized.ComboItem(label, value);
        var dropdownTemplate = item.ContentTemplate;
        var command = label.Split(" (", 2)[0];
        var selectionTemplate = (DataTemplate)Microsoft.UI.Xaml.Markup.XamlReader.Load($"""
            <DataTemplate xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">
                <TextBlock Text="{System.Security.SecurityElement.Escape(command)}" />
            </DataTemplate>
            """);
        item.ContentTemplate = null;
        item.ContentTemplateSelector = new CommandTemplateSelector(dropdownTemplate, selectionTemplate);
        return item;
    }

    private sealed class CommandTemplateSelector(DataTemplate dropdownTemplate, DataTemplate selectionTemplate) : DataTemplateSelector
    {
        protected override DataTemplate SelectTemplateCore(object item, DependencyObject container)
        {
            for (DependencyObject? parent = container; parent is not null; parent = VisualTreeHelper.GetParent(parent))
            {
                if (parent is ComboBoxItem) return dropdownTemplate;
                if (parent is ComboBox) break;
            }
            return selectionTemplate;
        }
    }
}
