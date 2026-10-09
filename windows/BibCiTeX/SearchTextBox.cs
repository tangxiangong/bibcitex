using Microsoft.UI.Input;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Markup;

namespace BibCiTeX;

internal sealed class SearchTextBox : TextBox
{
    internal SearchTextBox()
    {
        ProtectedCursor = InputSystemCursor.Create(InputSystemCursorShape.IBeam);
        Views.PlainTextBox(this);

        // WinUI inverts the background to draw the native caret. Transparent
        // Mica/Acrylic pixels cannot provide that contrast (WinUI issue #8207).
        // Keep the borderless style, but give focused input an opaque surface.
        Resources.Remove("TextControlBackgroundFocused");
        Resources.MergedDictionaries.Add((ResourceDictionary)XamlReader.Load("""
            <ResourceDictionary xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                                xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
                <ResourceDictionary.ThemeDictionaries>
                    <ResourceDictionary x:Key="Default">
                        <SolidColorBrush x:Key="TextControlBackgroundFocused"
                                         Color="{ThemeResource SolidBackgroundFillColorBase}" />
                    </ResourceDictionary>
                    <ResourceDictionary x:Key="HighContrast">
                        <SolidColorBrush x:Key="TextControlBackgroundFocused"
                                         Color="{ThemeResource SystemColorWindowColor}" />
                    </ResourceDictionary>
                </ResourceDictionary.ThemeDictionaries>
            </ResourceDictionary>
            """));
    }
}
