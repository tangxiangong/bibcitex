namespace BibCiTeX;

// Compile-only harness. The Windows-only XAML compiler owns InitializeComponent.
// Never run this assembly: it validates C# against real WinUI reference assemblies,
// but does not replace Windows XAML compilation or prove an application build.
public partial class App
{
    private void InitializeComponent() => throw new PlatformNotSupportedException("This assembly is a C# semantic check, not the Windows application.");
}
