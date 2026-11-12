import Carbon
import AppKit
import SwiftUI

struct HelperShortcut {
    static let defaultValue = "Cmd+Shift+K"
    let value: String
    let keyCode: UInt32
    let modifiers: UInt32
    let keyboardShortcut: KeyboardShortcut

    var displayValue: String {
        let parts = value.components(separatedBy: "+")
        let modifiers = [("Ctrl", "⌃"), ("Alt", "⌥"), ("Shift", "⇧"), ("Cmd", "⌘")]
            .filter { parts.contains($0.0) }.map(\.1).joined()
        return modifiers + (parts.last == "Space" ? "␣" : parts.last ?? "")
    }

    private static let keyNames: [UInt32: String] = {
        let keys = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".map(String.init) + ["Space"] + (1...12).map { "F\($0)" }
        return Dictionary(uniqueKeysWithValues: keys.compactMap { key in
            HelperShortcut("Cmd+" + key).map { ($0.keyCode, key) }
        })
    }()

    init?(keyCode: UInt16, modifierFlags: NSEvent.ModifierFlags) {
        guard let key = Self.keyNames[UInt32(keyCode)] else { return nil }
        let names = [(NSEvent.ModifierFlags.command, "Cmd"), (.control, "Ctrl"), (.option, "Alt"), (.shift, "Shift")]
            .filter { modifierFlags.contains($0.0) }.map(\.1)
        self.init((names + [key]).joined(separator: "+"))
    }

    init?(_ text: String) {
        var parts = text.uppercased().split(separator: "+", omittingEmptySubsequences: false)
            .map { $0.trimmingCharacters(in: .whitespaces) }
        guard let key = parts.popLast(), !parts.isEmpty else { return nil }
        var carbon: UInt32 = 0
        var flags: SwiftUI.EventModifiers = []
        for part in parts {
            let bit: UInt32
            let flag: SwiftUI.EventModifiers
            switch part {
            case "CMD", "COMMAND": bit = UInt32(cmdKey); flag = .command
            case "CTRL", "CONTROL": bit = UInt32(controlKey); flag = .control
            case "ALT", "OPTION", "OPT": bit = UInt32(optionKey); flag = .option
            case "SHIFT": bit = UInt32(shiftKey); flag = .shift
            default: return nil
            }
            guard carbon & bit == 0 else { return nil }
            carbon |= bit; flags.insert(flag)
        }
        guard carbon & UInt32(cmdKey | controlKey | optionKey) != 0 else { return nil }
        let codes: [String: UInt32] = [
            "A": 0, "S": 1, "D": 2, "F": 3, "H": 4, "G": 5, "Z": 6, "X": 7,
            "C": 8, "V": 9, "B": 11, "Q": 12, "W": 13, "E": 14, "R": 15,
            "Y": 16, "T": 17, "1": 18, "2": 19, "3": 20, "4": 21, "6": 22,
            "5": 23, "9": 25, "7": 26, "8": 28, "0": 29, "O": 31, "U": 32,
            "I": 34, "P": 35, "L": 37, "J": 38, "K": 40, "N": 45, "M": 46,
            "SPACE": 49, "F1": 122, "F2": 120, "F3": 99, "F4": 118, "F5": 96,
            "F6": 97, "F7": 98, "F8": 100, "F9": 101, "F10": 109, "F11": 103, "F12": 111
        ]
        guard let code = codes[key] else { return nil }
        let character: Character
        if key == "SPACE" { character = " " }
        else if key.hasPrefix("F"), let number = Int(key.dropFirst()), (1...12).contains(number),
                let scalar = UnicodeScalar(0xF703 + number) { character = Character(scalar) }
        else { character = Character(key.lowercased()) }
        let names = [(UInt32(cmdKey), "Cmd"), (UInt32(controlKey), "Ctrl"),
                     (UInt32(optionKey), "Alt"), (UInt32(shiftKey), "Shift")]
            .filter { carbon & $0.0 != 0 }.map(\.1)
        value = (names + [key == "SPACE" ? "Space" : key]).joined(separator: "+")
        keyCode = code
        modifiers = carbon
        keyboardShortcut = KeyboardShortcut(KeyEquivalent(character), modifiers: flags)
    }
}
