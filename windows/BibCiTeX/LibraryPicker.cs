using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace BibCiTeX;

// Compact native menu picker used inside the tray search field.
internal sealed class LibraryPicker : UserControl
{
    private readonly MiddleEllipsisText label = new(12);
    private readonly MenuFlyout menu = new();
    private Library? selected;
    private List<Library> items = [];
    internal event Action? SelectionChanged;
    internal static readonly DependencyProperty PlaceholderTextProperty = DependencyProperty.Register(nameof(PlaceholderText), typeof(string), typeof(LibraryPicker), new PropertyMetadata("", (owner, _) => ((LibraryPicker)owner).Refresh()));
    internal string PlaceholderText { get => (string)GetValue(PlaceholderTextProperty); set => SetValue(PlaceholderTextProperty, value); }
    internal List<Library> ItemsSource
    {
        set
        {
            items = value; menu.Items.Clear();
            foreach (var library in items)
            {
                var option = new ToggleMenuFlyoutItem { Text = library.Name, Tag = library };
                option.Click += (_, _) => { SelectedItem = library; Refresh(); }; menu.Items.Add(option);
            }
            IsEnabled = items.Count > 0; Refresh();
        }
    }
    internal Library? SelectedItem
    {
        get => selected;
        set { if (selected == value) return; selected = value; Refresh(); SelectionChanged?.Invoke(); }
    }
    internal LibraryPicker()
    {
        var row = new Grid { ColumnSpacing = 6 };
        row.ColumnDefinitions.Add(new()); row.ColumnDefinitions.Add(new() { Width = GridLength.Auto });
        label.VerticalAlignment = VerticalAlignment.Center; row.Children.Add(label);
        var arrow = new SvgIcon("chevronDown", 10) { VerticalAlignment = VerticalAlignment.Center }; Grid.SetColumn(arrow, 1); row.Children.Add(arrow);
        var button = new Button { Content = row, Flyout = menu, HorizontalAlignment = HorizontalAlignment.Stretch, HorizontalContentAlignment = HorizontalAlignment.Stretch, Padding = new Thickness(6, 0, 6, 0), MinHeight = 28, BorderThickness = new Thickness(0), Background = new Microsoft.UI.Xaml.Media.SolidColorBrush(Microsoft.UI.Colors.Transparent) };
        Localized.Name(button, "文献库"); Content = button;
        Localized.BindValue(label, MiddleEllipsisText.ValueProperty, () => selected?.Name ?? PlaceholderText);
        menu.Opening += (_, _) => Refresh();
    }
    private void Refresh()
    {
        Localized.BindValue(label, MiddleEllipsisText.ValueProperty, () => selected?.Name ?? PlaceholderText);
        Localized.BindValue(this, ToolTipService.ToolTipProperty, () => selected?.Name ?? L10n.Text("文献库"));
        foreach (var item in menu.Items.OfType<ToggleMenuFlyoutItem>()) item.IsChecked = item.Tag is Library library && library.Name == selected?.Name;
    }
}
