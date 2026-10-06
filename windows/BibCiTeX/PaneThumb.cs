using Microsoft.UI.Input;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;

namespace BibCiTeX;

// The Thumb owns native dragging; its host supplies the resize cursor.
internal sealed class PaneThumb : UserControl
{
    internal event Action<double>? Dragged;
    internal PaneThumb()
    {
        HorizontalContentAlignment = Microsoft.UI.Xaml.HorizontalAlignment.Stretch;
        VerticalContentAlignment = Microsoft.UI.Xaml.VerticalAlignment.Stretch;
        ProtectedCursor = InputSystemCursor.Create(InputSystemCursorShape.SizeWestEast);
        var thumb = new Thumb
        {
            Template = (ControlTemplate)Microsoft.UI.Xaml.Markup.XamlReader.Load("<ControlTemplate xmlns=\"http://schemas.microsoft.com/winfx/2006/xaml/presentation\" TargetType=\"Thumb\"><Grid Background=\"Transparent\"><Border Width=\"1\" Background=\"{ThemeResource ControlStrokeColorDefaultBrush}\" /></Grid></ControlTemplate>"),
            IsTabStop = false
        };
        thumb.DragDelta += (_, args) => Dragged?.Invoke(args.HorizontalChange);
        Content = thumb;
    }
}
