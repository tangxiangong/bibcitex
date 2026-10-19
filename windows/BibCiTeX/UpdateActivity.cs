namespace BibCiTeX;

/// Tracks queued and running service operations across the workbench and helper.
internal static class UpdateActivity
{
    private static int active;
    internal static bool IsBusy => Volatile.Read(ref active) != 0;
    internal static event Action? Changed;

    internal static Task Run(Action operation) => Run(() => { operation(); return true; });
    internal static async Task<T> Run<T>(Func<T> operation)
    {
        Interlocked.Increment(ref active);
        Changed?.Invoke();
        try { return await Task.Run(operation).ConfigureAwait(false); }
        finally { Interlocked.Decrement(ref active); Changed?.Invoke(); }
    }
}
