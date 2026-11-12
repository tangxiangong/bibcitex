using BibCiTeX;

var shortcut = HelperShortcut.Parse(" shift + win + k ");
if (shortcut != new HelperShortcut(12, 0x4B, HelperShortcut.DefaultValue))
    throw new Exception("Default shortcut did not normalize to the Windows registration values.");
if (HelperShortcut.Parse("Ctrl+Alt+Space") != new HelperShortcut(3, 0x20, "Ctrl+Alt+Space"))
    throw new Exception("Space shortcut did not preserve modifiers.");
if (HelperShortcut.Parse("Ctrl+F12")?.Key != 0x7B || HelperShortcut.Parse("Super+9")?.Value != "Win+9")
    throw new Exception("Function keys, digits or modifier aliases did not normalize.");
foreach (var invalid in new[] { "", "K", "Shift+K", "Ctrl+Ctrl+K", "Ctrl+", "Ctrl++K", "Ctrl+F13", "Ctrl+猫", "Ctrl+A+B" })
    if (HelperShortcut.Parse(invalid) is not null) throw new Exception("Accepted invalid shortcut: " + invalid);
Console.WriteLine("PASS helper shortcut parsing, Windows key codes and invalid combinations");
