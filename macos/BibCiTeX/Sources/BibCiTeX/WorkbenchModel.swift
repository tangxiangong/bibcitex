import AppKit
import SwiftUI
import BibCiTeXCore

enum CopySurface: Hashable {
    case list, detail
}

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
    private struct Request {
        var library: String?
        var query = ""
        var field = "all"
        var type = "all"
        var advanced = false
        var draft = SearchDraft()
    }

    @Published private(set) var libraries: [Bibliography] = []
    @Published private(set) var selectedLibrary: String?
    @Published private(set) var references: [Reference] = []
    @Published private(set) var selectedReference: String?
    @Published private(set) var query = ""
    @Published private(set) var field = "all"
    @Published private(set) var type = "all"
    @Published private(set) var matchRanges: [String: [SearchMatchRange]] = [:]
    @Published private(set) var matchEvidence: [String: String] = [:]
    @Published private(set) var queryAnalysis: QueryAnalysis?
    private var searchContext: SearchContext?
    var searchDraft: SearchDraft { requested.draft }
    @Published private(set) var advancedSearchEnabled = false
    private var ordinaryDraft = Request()
    private var advancedDraft = Request(advanced: true)
    @Published private(set) var loading = false
    @Published private(set) var errorDetails: LocalizedMessage?
    var error: String? {
        get { errorDetails?.text }
        set { errorDetails = newValue.map { LocalizedMessage(literal: $0) } }
    }
    /// Feedback and expiry are independent for the list and inspector.
    @Published private(set) var copied: [CopySurface: String] = [:]
    @Published var adding = false
    @Published var showSidebar = true {
        didSet { preferences.set(showSidebar, forKey: "mainShowSidebar") }
    }
    @Published var showInspector = false {
        didSet { preferences.set(showInspector, forKey: "mainShowInspector") }
    }

    func reportError(_ error: any Error) { errorDetails = LocalizedMessage(error: error) }

    private let service: any WorkbenchServing
    private let preferences: UserDefaults
    private var requested = Request()
    private var inputCommit: Task<Void, Never>?
    private var searchTask: Task<Void, Never>?
    private var rowCommit: Task<Void, Never>?
    private var copyConfirmation: [CopySurface: Task<Void, Never>] = [:]
    private var generation = 0
    private var registryGeneration = 0
    private var referencesRevision = 0
    private var referencesLibraryPath: String?

    init(service: any WorkbenchServing, preferences: UserDefaults = .standard) {
        self.service = service
        self.preferences = preferences
        advancedSearchEnabled = advancedSearchPreference(preferences, key: "mainAdvancedSearchEnabled")
        requested.advanced = advancedSearchEnabled
        showSidebar = preferences.object(forKey: "mainShowSidebar") as? Bool ?? true
        showInspector = preferences.object(forKey: "mainShowInspector") as? Bool ?? false
    }
    var library: Bibliography? { libraries.first { $0.name == selectedLibrary } }
    var reference: Reference? { references.first { $0.id == selectedReference } }
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
    /// Switches independent in-session drafts while persisting only this surface’s mode.
    func setAdvancedSearch(_ enabled: Bool) {
        guard requested.advanced != enabled else { return }
        if requested.advanced { advancedDraft = requested } else { ordinaryDraft = requested }
        let library = requested.library
        requested = enabled ? advancedDraft : ordinaryDraft
        requested.library = library
        requested.advanced = enabled
        advancedSearchEnabled = enabled
        preferences.set(enabled, forKey: "mainAdvancedSearchEnabled")
        scheduleSearch()
    }
    func setQuery(_ value: String) {
        guard requested.query != value else { return }
        requested.query = value
        requested.draft = SearchDraft()
        scheduleSearch()
    }
    /// Replaces the query with a validated visual edit, retaining stable tag IDs.
    func applyAdvancedQuery(_ text: String, draft: SearchDraft) {
        requested.query = text; requested.draft = draft
        scheduleSearch()
    }
    func refreshSearch() { scheduleSearch() }
    func suspendForComposition(_ composing: Bool) {
        guard composing else { scheduleSearch(); return }
        generation += 1; searchTask?.cancel(); inputCommit?.cancel(); searchContext = nil; referencesLibraryPath = nil
    }
    /// Uses native UTF-8 edit ranges to preserve untouched bound tag nodes.
    func editText(_ text: String, start: UInt32, end: UInt32, replacement: String, full: Bool) {
        guard requested.advanced, let document = requested.draft.document, !full else { setQuery(text); return }
        let result = BibCiTeXCore.editQueryText(edit: QueryTextEdit(document: document, text: requested.query, start: start, end: end, replacement: replacement, fullReplace: false))
        requested.query = text; requested.draft.document = result.document
        queryAnalysis = result; scheduleSearch()
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
              key == nil || references.contains(where: { $0.id == key }) else { return }
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
        let index = references.firstIndex { $0.id == selectedReference }
        let next = index.map { min(max($0 + delta, 0), references.count - 1) }
            ?? (delta < 0 ? references.count - 1 : 0)
        rowCommit?.cancel()
        selectedReference = references[next].id
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
                // A missing saved library stays selected as unavailable; never substitute another.
                requested.library = loaded.first(where: { $0.name == saved })?.name ?? loaded.first(where: \.available)?.name
            }
            scheduleSearch()
        } catch {
            guard version == registryGeneration else { return }
            self.errorDetails = LocalizedMessage(error: error)
        }
    }

    private func scheduleSearch() {
        generation += 1
        referencesRevision += 1
        referencesLibraryPath = nil
        searchContext = nil
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
        queryAnalysis = nil; matchEvidence = [:]
        references = []
        selectedReference = nil
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
        guard let library, library.available else {
            searchTask?.cancel()
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
                let response = try await RustCore.shared.searchExplicit(path: library.path, query: request.query, field: request.field, type: request.type, advanced: request.advanced, draft: request.draft, version: UInt64(version))
                let found = response.references
                guard !Task.isCancelled, generation == version else { return }
                if let text = response.refreshedText, let draft = response.refreshedDraft {
                    requested.query = text; requested.draft = draft; query = text
                }
                matchRanges = response.matchRanges
                matchEvidence = response.evidence
                queryAnalysis = response.analysis
                searchContext = response.context
                referencesLibraryPath = response.analysis.status == .valid ? library.path : nil
                referencesRevision += 1
                references = found
                if !found.contains(where: { $0.id == self.selectedReference }) { selectedReference = found.first?.id }
                if loading { loading = false }
                if error != nil { error = nil }
            } catch {
                guard !Task.isCancelled, generation == version else { return }
                referencesLibraryPath = nil
                referencesRevision += 1
                if loading { loading = false }
                references = []
                selectedReference = nil
                self.errorDetails = LocalizedMessage(error: error)
            }
        }
    }
    func remove(_ library: Bibliography) {
        Task {
            do { try await service.removeLibrary(name: library.name); await reload() }
            catch { self.errorDetails = LocalizedMessage(error: error) }
        }
    }
    func copy(_ text: String, surface: CopySurface = .list) {
        guard !loading, referencesLibraryPath != nil, references.contains(where: { $0.citeKey == text || $0.source == text }) else { return }
        let version = generation
        copyConfirmation[surface]?.cancel()
        copied[surface] = nil
        copyConfirmation[surface] = Task { @MainActor [weak self] in
            guard let self else { return }
            do {
                guard let context = searchContext, try await RustCore.shared.validateContext(context), version == generation else { scheduleSearch(); return }
                try await service.copy(text)
                guard !Task.isCancelled else { return }
                copied[surface] = text
                try await Task.sleep(nanoseconds: 1_400_000_000)
                guard !Task.isCancelled else { return }
                copied[surface] = nil
            } catch {
                guard !Task.isCancelled else { return }
                self.errorDetails = LocalizedMessage(error: error)
            }
        }
    }
    func openURL(_ string: String) {
        guard let context = searchContext, let url = URL(string: string), ["https", "http"].contains(url.scheme?.lowercased() ?? "") else { return }
        let version = generation
        Task { do { guard try await RustCore.shared.validateContext(context), version == generation else { scheduleSearch(); return }; NSWorkspace.shared.open(url) } catch { reportError(error) } }
    }
    func attachmentURL(for reference: Reference, context: ReferenceSelectionContext) -> URL? {
        // An inspector can still deliver an event while a new library is being
        // selected. Only open attachments belonging to the current result context.
        guard context == referenceSelectionContext, requested.library == selectedLibrary,
              let libraryPath = referencesLibraryPath,
              references.contains(where: { $0.citeKey == reference.citeKey && $0.source == reference.source }) else { return nil }
        return AttachmentPath.resolve(reference.file, relativeTo: libraryPath)
    }
    func openFile(_ reference: Reference, context: ReferenceSelectionContext) {
        guard let url = attachmentURL(for: reference, context: context) else { return }
        guard let searchContext else { return }
        let version = generation
        Task {
            do {
                guard try await RustCore.shared.validateContext(searchContext), version == generation else { scheduleSearch(); return }
                if !NSWorkspace.shared.open(url) { errorDetails = LocalizedMessage(key: "无法打开文件：{0}", arguments: [LocalizedMessage(literal: url.path)]) }
            } catch { reportError(error) }
        }
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
