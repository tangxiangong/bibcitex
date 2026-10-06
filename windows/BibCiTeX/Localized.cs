using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Data;

namespace BibCiTeX;

/// Bind only UI-owned text. Bibliography names, paths, keys and contents are never translated.
internal static class Localized
{
    internal static void Bind(DependencyObject target, DependencyProperty property, string key, params object[] arguments)
        => BindValue(target, property, () => L10n.Text(key, arguments));

    internal static void BindValue(DependencyObject target, DependencyProperty property, Func<string> value)
    {
        // The native Binding owns the managed source even if the target's CLR wrapper
        // is collected. Weak references to DependencyObject wrappers lose live menus/rows.
        BindingOperations.SetBinding(target, property, new Binding
        {
            Source = new LocalizationValue(value),
            Path = new PropertyPath(nameof(LocalizationValue.Value)),
            Mode = BindingMode.OneWay
        });
    }
    internal static ComboBoxItem ComboItem(string key, string value)
    {
        // ComboBox caches the selected Content separately. Keep that content stable
        // and bind inside its template so both presenters observe language changes.
        var item = new ComboBoxItem
        {
            Tag = value,
            Content = new LocalizationValue(() => L10n.Text(key)),
            ContentTemplate = (DataTemplate)Microsoft.UI.Xaml.Markup.XamlReader.Load("""
                <DataTemplate xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">
                    <TextBlock Text="{Binding Value, Mode=OneWay}" />
                </DataTemplate>
                """)
        };
        Name(item, key);
        return item;
    }
    internal static void Error(DependencyObject target, DependencyProperty property, Exception error)
        => BindValue(target, property, () => L10n.ErrorMessage(error));
    internal static ContentDialog Dialog(ContentDialog dialog, string title, string close = "确定", string? primary = null)
    {
        Bind(dialog, ContentDialog.TitleProperty, title);
        Bind(dialog, ContentDialog.CloseButtonTextProperty, close);
        if (primary is not null) Bind(dialog, ContentDialog.PrimaryButtonTextProperty, primary);
        BindValue(dialog, FrameworkElement.LanguageProperty, () => L10n.Language);
        return dialog;
    }
    internal static void Name(DependencyObject target, string key) => Bind(target, AutomationProperties.NameProperty, key);
    internal static void Tooltip(DependencyObject target, string key) => Bind(target, ToolTipService.ToolTipProperty, key);
    internal static void Count(TextBlock target, int count) => Bind(target, TextBlock.TextProperty, count == 1 ? "1 条文献" : "{0} 条文献", count);
}
