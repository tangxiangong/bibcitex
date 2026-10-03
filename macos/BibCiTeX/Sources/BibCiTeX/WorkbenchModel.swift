import AppKit
import SwiftUI

@MainActor
final class WorkbenchModel: ObservableObject {
    @Published var libraries: [Bibliography] = []
    @Published var selectedLibrary: String? {
        didSet {
            if oldValue != selectedLibrary { references = []; selectedReference = nil }
            search()
        }
    }
    @Published var references: [Reference] = []
    @Published var selectedReference: String?
    @Published var query = "" { didSet { search() } }
    @Published var field = "all" { didSet { search() } }
    @Published var type = "all" { didSet { search() } }
    @Published var loading = false
    @Published var error: String?
    @Published var adding = false
    @Published var showSidebar = true
    @Published var showInspector = true
    private var task: Task<Void, Never>?
    private var generation = 0

    var library: Bibliography? { libraries.first { $0.name == selectedLibrary } }
    var reference: Reference? { references.first { $0.citeKey == selectedReference } }

    func reload() async {
        do {
            libraries = try await RustCore.shared.libraries()
            if !libraries.contains(where: { $0.name == selectedLibrary }) {
                let saved = UserDefaults.standard.string(forKey: "mainLibrary")
                selectedLibrary = libraries.first(where: { $0.name == saved })?.name ?? libraries.first?.name
            } else { search() }
        } catch { self.error = error.localizedDescription }
    }
    func search() {
        generation += 1
        let version = generation
        task?.cancel()
        guard let library else { references = []; selectedReference = nil; loading = false; return }
        UserDefaults.standard.set(library.name, forKey: "mainLibrary")
        let query = query, field = field, type = type
        loading = true
        task = Task {
            do {
                try await Task.sleep(nanoseconds: 100_000_000)
                let found = try await RustCore.shared.search(path: library.path, query: query, field: field, type: type)
                guard !Task.isCancelled, generation == version else { return }
                references = found
                if !found.contains(where: { $0.citeKey == selectedReference }) { selectedReference = found.first?.citeKey }
                loading = false
                error = nil
            } catch {
                guard !Task.isCancelled, generation == version else { return }
                loading = false
                references = []
                selectedReference = nil
                self.error = error.localizedDescription
            }
        }
    }
    func remove(_ library: Bibliography) {
        Task {
            do { try await RustCore.shared.removeLibrary(name: library.name); await reload() }
            catch { self.error = error.localizedDescription }
        }
    }
    func copy(_ text: String) {
        Task { do { try await RustCore.pasteboard.copy(text) } catch { self.error = error.localizedDescription } }
    }
    func openURL(_ string: String) {
        guard let url = URL(string: string), ["https", "http"].contains(url.scheme?.lowercased() ?? "") else { return }
        NSWorkspace.shared.open(url)
    }
    func openFile(_ reference: Reference) {
        let raw = reference.file
        guard !raw.isEmpty else { return }
        // BibTeX attachments can be plain paths or Zotero's description:path:mimetype form.
        let entry = raw.split(separator: ";", maxSplits: 1).first.map(String.init) ?? raw
        let parts = entry.split(separator: ":", omittingEmptySubsequences: false)
        let path = parts.count >= 3 ? String(parts[1]) : entry
        let url: URL
        if path.hasPrefix("file://"), let file = URL(string: path) { url = file }
        else if path.hasPrefix("/") || path.hasPrefix("~") { url = URL(fileURLWithPath: (path as NSString).expandingTildeInPath) }
        else { url = URL(fileURLWithPath: library?.path ?? reference.source).deletingLastPathComponent().appendingPathComponent(path) }
        if !NSWorkspace.shared.open(url) { error = "无法打开文件：\(url.path)" }
    }
}
