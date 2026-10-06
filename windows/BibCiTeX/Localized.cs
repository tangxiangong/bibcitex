using System.Runtime.CompilerServices;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;

namespace BibCiTeX;

/// Bind only UI-owned text. Bibliography names, paths, keys and contents are never translated.
internal static class Localized
{
    private static readonly ConditionalWeakTable<DependencyObject, Dictionary<DependencyProperty, Func<string>>> Values = new();
    private static readonly List<WeakReference<DependencyObject>> Targets = [];
    static Localized() => L10n.Changed += Refresh;

    internal static void Bind(DependencyObject target, DependencyProperty property, string key, params object[] arguments)
        => BindValue(target, property, () => L10n.Text(key, arguments));

    internal static void BindValue(DependencyObject target, DependencyProperty property, Func<string> value)
    {
        if (!Values.TryGetValue(target, out var properties))
        {
            properties = new(); Values.Add(target, properties);
            Targets.Add(new(target));
            if (Targets.Count % 256 == 0) Targets.RemoveAll(reference => !reference.TryGetTarget(out _));
        }
        properties[property] = value;
        target.SetValue(property, value());
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
    private static void Refresh()
    {
        Targets.RemoveAll(reference => !reference.TryGetTarget(out _));
        foreach (var reference in Targets.ToArray())
            if (reference.TryGetTarget(out var target) && Values.TryGetValue(target, out var properties))
                foreach (var (property, value) in properties.ToArray()) target.SetValue(property, value());
    }
}
