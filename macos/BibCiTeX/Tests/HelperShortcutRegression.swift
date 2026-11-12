import Carbon

@main
struct HelperShortcutRegression {
    static func main() {
        let shortcut = HelperShortcut(" shift + command + k ")!
        precondition(shortcut.value == HelperShortcut.defaultValue)
        precondition(shortcut.keyCode == UInt32(kVK_ANSI_K))
        precondition(shortcut.modifiers == UInt32(cmdKey | shiftKey))
        precondition(HelperShortcut("Ctrl+Alt+Space")?.keyCode == UInt32(kVK_Space))
        precondition(HelperShortcut("Ctrl+F12")?.keyCode == UInt32(kVK_F12))
        precondition(HelperShortcut("Option+9")?.value == "Alt+9")
        for invalid in ["", "K", "Shift+K", "Cmd+Cmd+K", "Ctrl+", "Ctrl++K", "Ctrl+F13", "Ctrl+猫", "Ctrl+A+B"] {
            precondition(HelperShortcut(invalid) == nil, "accepted \(invalid)")
        }
        print("PASS helper shortcut parsing, native key codes and invalid combinations")
    }
}
