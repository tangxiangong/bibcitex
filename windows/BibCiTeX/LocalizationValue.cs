using System.ComponentModel;

namespace BibCiTeX;

// A binding keeps its source alive. The registry must not keep discarded UI alive.
#if WINDOWS
[Microsoft.UI.Xaml.Data.Bindable]
#endif
public sealed class LocalizationValue : INotifyPropertyChanged
{
    private static readonly List<WeakReference<LocalizationValue>> Sources = [];
    private readonly Func<string> read;
    static LocalizationValue() => L10n.Changed += Refresh;
    internal LocalizationValue(Func<string> read)
    {
        this.read = read;
        Sources.Add(new(this));
        if (Sources.Count % 256 == 0) Sources.RemoveAll(source => !source.TryGetTarget(out _));
    }
    public string Value => read();
    public override string ToString() => Value;
    public event PropertyChangedEventHandler? PropertyChanged;
    private static void Refresh()
    {
        Sources.RemoveAll(source => !source.TryGetTarget(out _));
        foreach (var source in Sources.ToArray())
            if (source.TryGetTarget(out var value))
                value.PropertyChanged?.Invoke(value, new PropertyChangedEventArgs(nameof(Value)));
    }
}
