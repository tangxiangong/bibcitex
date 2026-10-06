using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Controls;
using Windows.Foundation;

namespace BibCiTeX;

// Retain both the directory and filename using native text measurement and rendering.
internal sealed class MiddleEllipsisText : UserControl
{
    private readonly TextBlock label = new() { TextWrapping = TextWrapping.Wrap };
    private readonly TextBlock measure = new() { TextWrapping = TextWrapping.Wrap };
    internal static readonly DependencyProperty ValueProperty = DependencyProperty.Register(nameof(Value), typeof(string), typeof(MiddleEllipsisText), new PropertyMetadata("", (owner, _) => ((MiddleEllipsisText)owner).Render()));
    internal string Value { get => (string)GetValue(ValueProperty); set => SetValue(ValueProperty, value); }
    internal MiddleEllipsisText(double size = 11, int lines = 1)
    {
        FontSize = label.FontSize = measure.FontSize = size;
        label.MaxLines = lines; label.TextTrimming = TextTrimming.CharacterEllipsis;
        Content = label; SizeChanged += (_, _) => Render(); Loaded += (_, _) => Render();
    }
    private void Render()
    {
        var full = Value;
        AutomationProperties.SetName(this, full); ToolTipService.SetToolTip(this, full);
        if (ActualWidth <= 0) { label.Text = full; return; }
        measure.Text = "Ag"; measure.Measure(new Size(ActualWidth, double.PositiveInfinity));
        var height = measure.DesiredSize.Height * label.MaxLines + 1;
        label.Text = TextElision.Middle(full, candidate =>
        {
            measure.Text = candidate; measure.Measure(new Size(ActualWidth, double.PositiveInfinity));
            return measure.DesiredSize.Height <= height && measure.DesiredSize.Width <= ActualWidth + .5;
        });
    }
}
