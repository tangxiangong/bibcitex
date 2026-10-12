import Foundation

/// Resolves the first attachment without treating colons inside a path or URL
/// as field separators. Structured entries use description:path:type.
enum AttachmentPath {
    static func resolve(_ value: String, relativeTo bibliography: String) -> URL? {
        let end = separators(in: value, matching: ";").first ?? value.endIndex
        let entry = String(value[..<end]).trimmingCharacters(in: .whitespacesAndNewlines)
        guard !entry.isEmpty else { return nil }
        var path = unescape(entry)
        if !path.hasPrefix("/") && !path.hasPrefix("~") && !path.hasPrefix("file://") {
            let colons = separators(in: entry, matching: ":")
            if let first = colons.first, let last = colons.last, first != last {
                let type = String(entry[entry.index(after: last)...])
                // A type suffix distinguishes structured entries from relative
                // filenames such as paper:2024:final.pdf.
                if type.isEmpty || type.range(of: #"^[A-Za-z][A-Za-z0-9+-]*(/[A-Za-z0-9.+-]+)?$"#, options: .regularExpression) != nil {
                    path = unescape(String(entry[entry.index(after: first)..<last]))
                }
            }
        }
        guard !path.isEmpty else { return nil }
        if path.hasPrefix("file://") { return URL(string: path) }
        if path.hasPrefix("/") || path.hasPrefix("~") {
            return URL(fileURLWithPath: (path as NSString).expandingTildeInPath)
        }
        return URL(fileURLWithPath: bibliography).deletingLastPathComponent().appendingPathComponent(path)
    }

    private static func separators(in text: String, matching separator: Character) -> [String.Index] {
        var escaped = false
        return text.indices.filter { index in
            let character = text[index]
            if escaped { escaped = false; return false }
            if character == "\\" { escaped = true; return false }
            return character == separator
        }
    }

    private static func unescape(_ text: String) -> String {
        var result = "", index = text.startIndex
        while index < text.endIndex {
            let character = text[index]
            let next = text.index(after: index)
            if character == "\\", next < text.endIndex, [":", ";", "\\"].contains(text[next]) {
                result.append(text[next]); index = text.index(after: next)
            } else {
                result.append(character); index = next
            }
        }
        return result
    }
}
