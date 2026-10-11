using Microsoft.UI.Input;
using Microsoft.UI.Xaml.Controls;
using Windows.System;
using Windows.UI.Core;

namespace BibCiTeX;

/// <summary>Tracks one native editor's IME lifecycle and keeps its commit Enter out of helper commands.</summary>
internal sealed class HelperInputGuard
{
    private bool commitEnterDown;
    private bool endedInDispatch;
    /// <summary>True while the native text box owns marked composition text.</summary>
    internal bool IsComposing { get; private set; }
    /// <summary>Changes whenever a composition starts, invalidating asynchronous focus or condition work.</summary>
    internal int Epoch { get; private set; }
    /// <summary>Raised after native composition state is updated, before helper-specific handlers run.</summary>
    internal event Action? Started;
    /// <summary>Raised after composition commits; the physical commit Enter remains guarded until release.</summary>
    internal event Action? Ended;
    /// <summary>Observes native events without replacing text, moving selections, or intercepting IME editing keys.</summary>
    internal HelperInputGuard(TextBox input)
    {
        input.TextCompositionStarted += (_, _) => { IsComposing = true; Epoch++; endedInDispatch = false; Started?.Invoke(); };
        input.TextCompositionEnded += (_, _) =>
        {
            IsComposing = false;
            commitEnterDown = (InputKeyboardSource.GetKeyStateForCurrentThread(VirtualKey.Enter) & CoreVirtualKeyStates.Down) != 0;
            endedInDispatch = true;
            int epoch = Epoch;
            input.DispatcherQueue.TryEnqueue(() => { if (epoch == Epoch) endedInDispatch = false; });
            Ended?.Invoke();
        };
        input.PreviewKeyDown += (_, args) => { if (Consumes(args.Key)) args.Handled = true; };
        input.KeyUp += (_, args) => { if (args.Key == VirtualKey.Enter) { commitEnterDown = false; endedInDispatch = false; } };
        input.LostFocus += (_, _) => { if (!IsComposing) { commitEnterDown = false; endedInDispatch = false; } };
    }
    /// <summary>Identifies an already committed IME Enter; callers consume it without confirming or pasting.</summary>
    internal bool Consumes(VirtualKey key) => key == VirtualKey.Enter && !IsComposing && (commitEnterDown || endedInDispatch);
}
