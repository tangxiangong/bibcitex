import Foundation

/// Shared source-language keys, with platform-local language preferences.
enum L10n {
    static let languages = ["system", "zh-Hans", "en"]
    private static let catalogs: [String: [String: String]] = Dictionary(uniqueKeysWithValues:
        ["zh-Hans", "en"].map { language in
            let url = Bundle.main.url(forResource: language, withExtension: "json")
            let data = url.flatMap { try? Data(contentsOf: $0) }
            let strings = data.flatMap { try? JSONDecoder().decode([String: String].self, from: $0) } ?? [:]
            return (language, strings)
        })

    static func resolvedLanguage(_ selection: String, preferred: [String] = Locale.preferredLanguages) -> String {
        if selection == "zh-Hans" || selection == "en" { return selection }
        // Traditional Chinese and unsupported languages use the English fallback.
        let first = preferred.first?.lowercased().replacingOccurrences(of: "_", with: "-") ?? "en"
        return first == "zh" || first.hasPrefix("zh-hans") || first == "zh-cn" || first == "zh-sg" ? "zh-Hans" : "en"
    }

    static var language: String {
        resolvedLanguage(UserDefaults.standard.string(forKey: "language") ?? "system")
    }
    static var locale: Locale { Locale(identifier: language) }

    static func text(_ key: String, _ arguments: Any...) -> String {
        translate(key, language: language, arguments: arguments.map { String(describing: $0) })
    }

    private static let placeholderPattern = try! NSRegularExpression(pattern: #"\{([0-9]+)\}"#)

    static func translate(_ key: String, language: String, arguments: [String] = []) -> String {
        var result = catalogs[language]?[key] ?? catalogs["en"]?[key] ?? key
        guard !arguments.isEmpty else { return result }
        let matches = placeholderPattern.matches(in: result, range: NSRange(result.startIndex..., in: result))
        for match in matches.reversed() {
            guard let indexRange = Range(match.range(at: 1), in: result),
                  let index = Int(result[indexRange]), index < arguments.count,
                  let range = Range(match.range, in: result) else { continue }
            result.replaceSubrange(range, with: arguments[index])
        }
        return result
    }

    // Only entries reserved for native menus participate in title matching.
    // Bibliography values and window titles must never use this lookup.
    private static let menuAliases: [String: String] = {
        var aliases: [String: String] = [:]
        for language in ["en", "zh-Hans"] {
            for (key, title) in (catalogs[language] ?? [:]).sorted(by: { $0.key < $1.key })
                where (key.hasPrefix("menu.") || key.hasPrefix("undoAction.")) && !title.contains("{0}") {
                if aliases[title] == nil { aliases[title] = key }
            }
        }
        return aliases
    }()

    private static let menuTemplates: [(key: String, prefix: String, suffix: String)] = {
        var templates: [(key: String, prefix: String, suffix: String)] = []
        for language in ["en", "zh-Hans"] {
            for (key, title) in (catalogs[language] ?? [:]).sorted(by: { $0.key < $1.key }) where key.hasPrefix("menu.") {
                let parts = title.components(separatedBy: "{0}")
                if parts.count == 2 && (!parts[0].isEmpty || !parts[1].isEmpty) { templates.append((key, parts[0], parts[1])) }
            }
        }
        // Prefer the more specific template when prefixes overlap (Move / Move All).
        return templates.sorted { $0.prefix.count + $0.suffix.count > $1.prefix.count + $1.suffix.count }
    }()

    static func nativeMenuTitle(_ title: String, language: String) -> String? {
        if let key = menuAliases[title] ?? menuAliases[title.replacingOccurrences(of: "...", with: "…")] {
            return translate(key, language: language)
        }
        for template in menuTemplates where title.hasPrefix(template.prefix) && title.hasSuffix(template.suffix)
            && title.count > template.prefix.count + template.suffix.count {
            var argument = String(title.dropFirst(template.prefix.count).dropLast(template.suffix.count))
            if template.key == "menu.undoAction" || template.key == "menu.redoAction" {
                let actionName = argument.trimmingCharacters(in: .whitespaces)
                if let key = menuAliases[actionName] { argument = translate(key, language: language) }
            }
            return translate(template.key, language: language, arguments: [argument])
        }
        return nil
    }

    static func references(_ count: Int) -> String {
        count == 1 ? text("1 条文献") : text("{0} 条文献", count)
    }
}

protocol LocalizedMessageProviding {
    var localizedMessage: LocalizedMessage { get }
}

/// Keep keys and literal diagnostics separate until the view renders them.
struct LocalizedMessage: Sendable {
    private indirect enum Content: Sendable {
        case literal(String)
        case translated(String, [LocalizedMessage])
    }
    private let content: Content

    init(literal: String) { content = .literal(literal) }
    init(key: String, arguments: [LocalizedMessage] = []) { content = .translated(key, arguments) }
    init(error: any Error) {
        self = (error as? any LocalizedMessageProviding)?.localizedMessage
            ?? LocalizedMessage(literal: error.localizedDescription)
    }
    var text: String { resolve(language: L10n.language) }
    func resolve(language: String) -> String {
        switch content {
        case .literal(let text): return text
        case .translated(let key, let arguments):
            return L10n.translate(key, language: language, arguments: arguments.map { $0.resolve(language: language) })
        }
    }
}
