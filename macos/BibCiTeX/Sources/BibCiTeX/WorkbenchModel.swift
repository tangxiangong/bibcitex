import AppKit
import SwiftUI

protocol WorkbenchServing: Sendable {
    func libraries() async throws -> [Bibliography]
    func search(path: String, query: String, field: String, type: String) async throws -> [Reference]
    func removeLibrary(name: String) async throws
    func copy(_ text: String) async throws
}

@MainActor
final class WorkbenchModel: ObservableObject {
    struct ReferenceSelectionContext: Equatable {
        let library: String?
        let revision: Int
    }
    private struct Request: Equatable {
        var library: String?
        var query = ""
        var field = "all"
        var type = "all"
    }

    @Published private(set) var libraries: [Bibliography] = []
    @Published private(set) var selectedLibrary: String?
    @Published private(set) var references: [Reference] = []
    @Published private(set) var selectedReference: String?
    @Published private(set) var query = ""
    @Published private(set) var field = "all"
    @Published private(set) var type = "all"
    @Published private(set) var loading = false
    @Published var error: String?
    /// The text most recently copied, so the control that copied it can confirm
    /// in place. Both the cite key and the BibTeX source copy through `copy`, and
    /// this is what tells them apart.
    @Published private(set) var copied: String?
    @Published var adding = false
    @Published var showSidebar = true
    @Published var showInspector = true

    private let service: any WorkbenchServing
    private let preferences: UserDefaults
    private var requested = Request()
    private var inputCommit: Task<Void, Never>?
    private var searchTask: Task<Void, Never>?
    private var rowCommit: Task<Void, Never>?
    private var copyConfirmation: Task<Void, Never>?
    private var generation = 0
    private var registryGeneration = 0
    private var referencesRevision = 0
    private var referencesLibraryPath: String?

    init(service: any WorkbenchServing, preferences: UserDefaults = .standard) {
        self.service = service
        self.preferences = preferences
    }
    var library: Bibliography? { libraries.first { $0.name == selectedLibrary } }
    var reference: Reference? { references.first { $0.citeKey == selectedReference } }
    var referenceSelectionContext: ReferenceSelectionContext {
        ReferenceSelectionContext(library: selectedLibrary, revision: referencesRevision)
    }

    // SwiftUI may write these bindings while reconciling a native List/TextField.
    // Record intent and invalidate outstanding work immediately, but publish the
    // coherent input/result transition in one cancellable MainActor job.
    func selectLibrary(_ name: String?) {
        guard name == nil || libraries.contains(where: { $0.name == name }) else { return }
        guard requested.library != name else { return }
        requested.library = name
        scheduleSearch()
    }
    func setQuery(_ value: String) {
        guard requested.query != value else { return }
        requested.query = value
        scheduleSearch()
    }
    func setField(_ value: String) {
        guard requested.field != value else { return }
        requested.field = value
        scheduleSearch()
    }
    func setType(_ value: String) {
        guard requested.type != value else { return }
        requested.type = value
        scheduleSearch()
    }
    func selectReference(_ key: String?, context: ReferenceSelectionContext) {
        guard context == referenceSelectionContext, requested.library == context.library,
              key == nil || references.contains(where: { $0.citeKey == key }) else { return }
        rowCommit?.cancel()
        guard key != selectedReference else { return }
        let version = generation
        rowCommit = Task { @MainActor [weak self] in
            guard let self, !Task.isCancelled, generation == version,
                  context == referenceSelectionContext, requested.library == context.library else { return }
            if selectedReference != key { selectedReference = key }
        }
    }

    func moveReference(_ delta: Int) {
        guard !loading, requested.library == selectedLibrary,
              requested.query == query, requested.field == field, requested.type == type,
              !references.isEmpty else { return }
        let index = references.firstIndex { $0.citeKey == selectedReference }
        let next = index.map { min(max($0 + delta, 0), references.count - 1) }
            ?? (delta < 0 ? references.count - 1 : 0)
        rowCommit?.cancel()
        selectedReference = references[next].citeKey
    }

    func reload() async {
        registryGeneration += 1
        let version = registryGeneration
        do {
            let loaded = try await service.libraries()
            guard version == registryGeneration else { return }
            libraries = loaded
            if !loaded.contains(where: { $0.name == requested.library }) {
                let saved = preferences.string(forKey: "mainLibrary")
                requested.library = loaded.first(where: { $0.name == saved })?.name ?? loaded.first?.name
            }
            scheduleSearch()
        } catch {
            guard version == registryGeneration else { return }
            self.error = error.localizedDescription
        }
    }

    private func scheduleSearch() {
        generation += 1
        let version = generation
        let request = requested
        searchTask?.cancel()
        rowCommit?.cancel()
        inputCommit?.cancel()
        inputCommit = Task { @MainActor [weak self] in
            guard let self, !Task.isCancelled, generation == version else { return }
            commit(request, version: version)
        }
    }
    private func commit(_ request: Request, version: Int) {
        if selectedLibrary != request.library {
            // Clear the old result context together with its library selection.
            referencesRevision += 1
            referencesLibraryPath = nil
            references = []
            selectedReference = nil
            selectedLibrary = request.library
        }
        if query != request.query { query = request.query }
        if field != request.field { field = request.field }
        if type != request.type { type = request.type }
        guard let library else {
            if !references.isEmpty { references = []; referencesRevision += 1 }
            referencesLibraryPath = nil
            if selectedReference != nil { selectedReference = nil }
            if loading { loading = false }
            return
        }
        preferences.set(library.name, forKey: "mainLibrary")
        if !loading { loading = true }
        searchTask = Task { @MainActor [weak self] in
            guard let self else { return }
            do {
                try await Task.sleep(nanoseconds: 100_000_000)
                let found = try await service.search(path: library.path, query: request.query, field: request.field, type: request.type)
                guard !Task.isCancelled, generation == version else { return }
                referencesLibraryPath = library.path
                referencesRevision += 1
                references = found
                if !found.contains(where: { $0.citeKey == self.selectedReference }) { selectedReference = found.first?.citeKey }
                if loading { loading = false }
                if error != nil { error = nil }
            } catch {
                guard !Task.isCancelled, generation == version else { return }
                referencesLibraryPath = nil
                referencesRevision += 1
                if loading { loading = false }
                references = []
                selectedReference = nil
                self.error = error.localizedDescription
            }
        }
    }
    func remove(_ library: Bibliography) {
        Task {
            do { try await service.removeLibrary(name: library.name); await reload() }
            catch { self.error = error.localizedDescription }
        }
    }
    func copy(_ text: String) {
        Task {
            do {
                try await service.copy(text)
                confirmCopy(of: text)
            } catch { self.error = error.localizedDescription }
        }
    }
    /// The copied text itself identifies which button was pressed, so the inspector
    /// can answer on that button instead of raising an alert for a copy that needs
    /// no acknowledgement beyond "it worked".
    private func confirmCopy(of text: String) {
        copied = text
        copyConfirmation?.cancel()
        copyConfirmation = Task { @MainActor [weak self] in
            try? await Task.sleep(nanoseconds: 1_400_000_000)
            guard let self, !Task.isCancelled else { return }
            if self.copied == text { self.copied = nil }
        }
    }
    func openURL(_ string: String) {
        guard let url = URL(string: string), ["https", "http"].contains(url.scheme?.lowercased() ?? "") else { return }
        NSWorkspace.shared.open(url)
    }
    func attachmentURL(for reference: Reference, context: ReferenceSelectionContext) -> URL? {
        // An inspector can still deliver an event while a new library is being
        // selected. Only open attachments belonging to the current result context.
        guard context == referenceSelectionContext, requested.library == selectedLibrary,
              let libraryPath = referencesLibraryPath,
              references.contains(where: { $0.citeKey == reference.citeKey && $0.source == reference.source }) else { return nil }
        let raw = reference.file
        guard !raw.isEmpty else { return nil }
        let entry = raw.split(separator: ";", maxSplits: 1).first.map(String.init) ?? raw
        let parts = entry.split(separator: ":", omittingEmptySubsequences: false)
        let path = parts.count >= 3 ? String(parts[1]) : entry
        let url: URL
        if path.hasPrefix("file://"), let file = URL(string: path) { url = file }
        else if path.hasPrefix("/") || path.hasPrefix("~") { url = URL(fileURLWithPath: (path as NSString).expandingTildeInPath) }
        else { url = URL(fileURLWithPath: libraryPath).deletingLastPathComponent().appendingPathComponent(path) }
        return url
    }
    func openFile(_ reference: Reference, context: ReferenceSelectionContext) {
        guard let url = attachmentURL(for: reference, context: context) else { return }
        if !NSWorkspace.shared.open(url) { error = "无法打开文件：\(url.path)" }
    }
}

actor RustWorkbenchService: WorkbenchServing {
    func libraries() async throws -> [Bibliography] { try await RustCore.shared.libraries() }
    func search(path: String, query: String, field: String, type: String) async throws -> [Reference] {
        try await RustCore.shared.search(path: path, query: query, field: field, type: type)
    }
    func removeLibrary(name: String) async throws { try await RustCore.shared.removeLibrary(name: name) }
    func copy(_ text: String) async throws { try await RustCore.pasteboard.copy(text) }
}

extension WorkbenchModel {
    convenience init() { self.init(service: RustWorkbenchService()) }
}
