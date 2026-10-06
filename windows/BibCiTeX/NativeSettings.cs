using System.Text.Json;

namespace BibCiTeX;

/// Preferences live outside Velopack's replaceable application directory.
internal static class NativeSettings
{
    internal static Store Values { get; } = new();
    internal sealed class Store
    {
        private readonly string path;
        private readonly Dictionary<string, string> values;
        internal Store(string? settingsPath = null)
        {
            path = settingsPath ?? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "BibCiTeX", "native-settings.json");
            try { values = JsonSerializer.Deserialize<Dictionary<string, string>>(File.ReadAllText(path)) ?? new(); }
            catch (FileNotFoundException) { values = new(); }
            catch (DirectoryNotFoundException) { values = new(); }
            catch (JsonException)
            {
                // Preserve malformed preferences for recovery without blocking app startup.
                File.Move(path, path + ".corrupt-" + Guid.NewGuid().ToString("N"));
                values = new();
            }
        }
        internal object? this[string key]
        {
            get => values.GetValueOrDefault(key);
            set
            {
                var next = new Dictionary<string, string>(values) { [key] = value?.ToString() ?? "" };
                Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                var temporary = path + "." + Guid.NewGuid() + ".tmp";
                try { File.WriteAllText(temporary, JsonSerializer.Serialize(next)); File.Move(temporary, path, true); }
                finally { if (File.Exists(temporary)) File.Delete(temporary); }
                values[key] = next[key];
            }
        }
        internal bool TryGetValue(string key, out object? value)
        {
            var found = values.TryGetValue(key, out var text); value = text; return found;
        }
    }
}
