using System.Runtime.CompilerServices;

namespace BibCiTeX;

/// WinUI permits one ContentDialog per XamlRoot, including dialogs requested from flyouts.
internal static class DialogQueue
{
    private static readonly ConditionalWeakTable<object, SemaphoreSlim> Gates = new();
    internal static async Task<T> Run<T>(object owner, Func<Task<T>> show)
    {
        var gate = Gates.GetValue(owner, _ => new SemaphoreSlim(1, 1));
        await gate.WaitAsync();
        try { return await show(); }
        finally { gate.Release(); }
    }
}
