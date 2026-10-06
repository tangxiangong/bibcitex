namespace BibCiTeX;

internal static class DialogQueueTests
{
    internal static async Task Run()
    {
        var owner = new object();
        var firstRelease = new TaskCompletionSource<int>(TaskCreationOptions.RunContinuationsAsynchronously);
        var first = DialogQueue.Run(owner, () => firstRelease.Task);
        var secondStarted = new TaskCompletionSource<bool>(TaskCreationOptions.RunContinuationsAsynchronously);
        var second = DialogQueue.Run(owner, () => { secondStarted.SetResult(true); return Task.FromResult(2); });
        if (secondStarted.Task.IsCompleted) throw new InvalidOperationException("Nested dialogs must queue until the active dialog closes.");
        // A separate window must stay usable while the first window has a dialog.
        var other = DialogQueue.Run(new object(), () => Task.FromResult(3));
        if (await other.WaitAsync(TimeSpan.FromSeconds(2)) != 3) throw new InvalidOperationException("Independent dialog roots were blocked.");
        firstRelease.SetException(new InvalidOperationException("Failed dialog"));
        try { await first; throw new InvalidOperationException("Expected failed dialog"); }
        catch (InvalidOperationException error) when (error.Message == "Failed dialog") { }
        if (await second.WaitAsync(TimeSpan.FromSeconds(2)) != 2 || !secondStarted.Task.IsCompleted)
            throw new InvalidOperationException("A failed dialog must release the queue for the next request.");
        Console.WriteLine("PASS dialog queue: nested requests wait, independent windows remain usable, failure releases pending dialogs");
    }
}
