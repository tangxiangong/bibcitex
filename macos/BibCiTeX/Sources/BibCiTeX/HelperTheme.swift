import Foundation

enum ThemeStyle: Equatable {
    case latte
    case mocha

    init(mode: Int32) {
        self = mode == ThemeMode.dark.rawValue ? .mocha : .latte
    }

    var isDark: Bool {
        self == .mocha
    }

    var panelMaterial: String {
        isDark ? "ultraThickMaterial" : "thickMaterial"
    }
}

enum HelperError: LocalizedError {
    case ffiError(String)
    case emptyCiteKey

    var errorDescription: String? {
        switch self {
        case .ffiError(let message):
            return message
        case .emptyCiteKey:
            return "引用键为空"
        }
    }
}

actor HelperService: HelperServing {
    func loadBibliographyStateIfAvailable() async throws -> (bibs: [Bibliography], current: Bibliography?) {
        let bibs = try await RustCore.helper.libraries()
        let current = try await RustCore.helper.currentLibrary()
        return (bibs, current)
    }
    func setCurrentBibliography(name: String, path: String) async throws -> Bibliography {
        try await RustCore.helper.selectLibrary(name: name, path: path)
    }
    func searchReferences(query: String) async throws -> [Reference] {
        guard let library = try await RustCore.helper.currentLibrary() else { return [] }
        return try await RustCore.helper.search(path: library.path, query: query)
    }
    func copyAndPaste(citeKey: String) async throws {
        guard !citeKey.isEmpty else { throw HelperError.emptyCiteKey }
        try await RustCore.pasteboard.paste(citeKey)
    }
}
