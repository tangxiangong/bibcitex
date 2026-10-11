import AppKit
import SwiftUI
import BibCiTeXCore

/// State owned only by the tray workbench. The Rust service is shared; UI state is not.
@MainActor
final class TrayWorkbenchModel: ObservableObject {
    @Published private(set) var libraries: [Bibliography] = []
    @Published var libraryName: String? { didSet { if oldValue != libraryName { search() } } }
    @Published var query = "" { didSet { if oldValue != query { if !restoringDraft { boundDraft = SearchDraft() }; search() } } }
    @Published var field = "all" { didSet { if oldValue != field { search() } } }
    @Published var type = "all" { didSet { if oldValue != type { search() } } }
    @Published private(set) var advancedSearchEnabled = false
    @Published private(set) var matchRanges: [String: [SearchMatchRange]] = [:]
    @Published private(set) var matchEvidence: [String: String] = [:]
    @Published private(set) var queryAnalysis: QueryAnalysis?
    private var searchContext: SearchContext?
    private(set) var boundDraft = SearchDraft()
    private struct Draft { var query = ""; var field = "all"; var type = "all"; var bound = SearchDraft() }
    private var ordinaryDraft = Draft()
    private var advancedDraft = Draft()
    private var restoringDraft = false
    @Published private(set) var references: [Reference] = []
    @Published var detailVisible = false
    @Published var selection: String?
    @Published private(set) var loading = false
    @Published private(set) var copied: [CopySurface: String] = [:]
    @Published private(set) var errorDetails: LocalizedMessage?
    var error: String? {
        get { errorDetails?.text }
        set { errorDetails = newValue.map { LocalizedMessage(literal: $0) } }
    }
    private let service: any WorkbenchServing
    private let preferences: UserDefaults
    private var searchTask: Task<Void, Never>?
    private var copyTask: [CopySurface: Task<Void, Never>] = [:]
    private var searchVersion = 0
    private var loadedVersion = -1
    private var reloadVersion = 0
    private var session = 0

    init(service: any WorkbenchServing, preferences: UserDefaults = .standard) {
        self.service = service
        self.preferences = preferences
        advancedSearchEnabled = advancedSearchPreference(preferences, key: "trayAdvancedSearchEnabled")
    }
    var reference: Reference? {
        guard loadedVersion == searchVersion else { return nil }
        return references.first { $0.citeKey == selection }
    }
    func reload() async {
        reloadVersion += 1
        let version = reloadVersion
        searchVersion += 1
        searchTask?.cancel()
        loading = true
        error = nil
        do {
            let rows = try await service.libraries()
            guard version == reloadVersion else { return }
            libraries = rows
            let preferred = libraryName ?? preferences.string(forKey: "trayLibrary")
            let selected = rows.first { $0.name == preferred }?.name ?? rows.first(where: \.available)?.name
            if libraryName != selected { libraryName = selected } else { search() }
        } catch {
            guard version == reloadVersion else { return }
            references = []
            selection = nil
            loading = false
            self.errorDetails = LocalizedMessage(error: error)
        }
    }
    /// Switches independent in-session drafts while persisting only this surface’s mode.
    func setAdvancedSearch(_ enabled: Bool) {
        guard enabled != advancedSearchEnabled else { return }
        let draft = Draft(query: query, field: field, type: type, bound: boundDraft)
        if advancedSearchEnabled { advancedDraft = draft } else { ordinaryDraft = draft }
        advancedSearchEnabled = enabled
        preferences.set(enabled, forKey: "trayAdvancedSearchEnabled")
        let next = enabled ? advancedDraft : ordinaryDraft
        restoringDraft = true
        query = next.query; field = next.field; type = next.type; boundDraft = next.bound
        restoringDraft = false
        search()
    }
    /// Replaces the query with a validated visual edit, retaining stable tag IDs.
    func applyAdvancedQuery(_ text: String, draft: SearchDraft) {
        restoringDraft = true; query = text; boundDraft = draft; restoringDraft = false; search()
    }
    func suspendForComposition(_ composing: Bool) {
        guard composing else { search(); return }
        searchVersion += 1; searchTask?.cancel(); searchContext = nil; loadedVersion = -1
    }
    /// Uses native UTF-8 edit ranges to preserve untouched bound tag nodes.
    func editText(_ text: String, start: UInt32, end: UInt32, replacement: String, full: Bool) {
        guard advancedSearchEnabled, let document = boundDraft.document, !full else { query = text; return }
        let result = BibCiTeXCore.editQueryText(edit: QueryTextEdit(document: document, text: query, start: start, end: end, replacement: replacement, fullReplace: false))
        restoringDraft = true; query = text; restoringDraft = false
        boundDraft.document = result.document; queryAnalysis = result; search()
    }
    func refreshSearch() { search() }
    private func search() {
        guard !restoringDraft else { return }
        queryAnalysis = nil; matchEvidence = [:]
        references = []; selection = nil; loadedVersion = -1; searchContext = nil
        loading = true
        searchVersion += 1
        let version = searchVersion
        let library = libraries.first { $0.name == libraryName }
        let query = query, field = field, type = type
        let advanced = advancedSearchEnabled, draft = boundDraft
        searchTask?.cancel()
        searchTask = Task { @MainActor [weak self] in
            guard let self, !Task.isCancelled else { return }
            loading = true
            error = nil
            guard let library, library.available else {
                references = []; selection = nil; loading = false
                return
            }
            preferences.set(library.name, forKey: "trayLibrary")
            do {
                try await Task.sleep(nanoseconds: 100_000_000)
                let response = try await RustCore.shared.searchExplicit(path: library.path, query: query, field: field, type: type, advanced: advanced, draft: draft, version: UInt64(version))
                let rows = response.references
                guard !Task.isCancelled, version == searchVersion else { return }
                if let text = response.refreshedText, let draft = response.refreshedDraft {
                    restoringDraft = true; self.query = text; boundDraft = draft; restoringDraft = false
                }
                matchRanges = response.matchRanges
                matchEvidence = response.evidence
                queryAnalysis = response.analysis; searchContext = response.context
                references = rows
                loadedVersion = version
                if !rows.contains(where: { $0.citeKey == self.selection }) { selection = rows.first?.citeKey }
                loading = false
            } catch {
                guard !Task.isCancelled, version == searchVersion else { return }
                references = []; selection = nil; loading = false
                self.errorDetails = LocalizedMessage(error: error)
            }
        }
    }
    func move(_ delta: Int) {
        guard !loading, loadedVersion == searchVersion, !references.isEmpty else { return }
        let index = references.firstIndex { $0.citeKey == selection }
        let next = index.map { min(max($0 + delta, 0), references.count - 1) }
            ?? (delta > 0 ? 0 : references.count - 1)
        selection = references[next].citeKey
    }
    func openURL(_ string: String) {
        guard let url = URL(string: string), ["https", "http"].contains(url.scheme?.lowercased() ?? "") else { return }
        guard let context = searchContext else { return }
        let version = searchVersion
        Task { do { guard try await RustCore.shared.validateContext(context), version == searchVersion else { search(); return }; NSWorkspace.shared.open(url) } catch { self.errorDetails = LocalizedMessage(error: error) } }
    }
    func copy(_ value: String, surface: CopySurface = .list) {
        guard !loading, loadedVersion == searchVersion, references.contains(where: { $0.citeKey == value || $0.source == value }) else { return }
        let currentSession = session
        let version = searchVersion
        copyTask[surface]?.cancel()
        copied[surface] = nil
        copyTask[surface] = Task { @MainActor [weak self] in
            guard let self else { return }
            do {
                guard let context = searchContext, try await RustCore.shared.validateContext(context), version == searchVersion else { search(); return }
                try await service.copy(value)
                guard !Task.isCancelled, currentSession == session else { return }
                copied[surface] = value
                try await Task.sleep(nanoseconds: 1_500_000_000)
                if !Task.isCancelled { copied[surface] = nil }
            } catch {
                guard !Task.isCancelled, currentSession == session else { return }
                self.errorDetails = LocalizedMessage(error: error)
            }
        }
    }
    func suspend() {
        session += 1; reloadVersion += 1; searchVersion += 1
        searchTask?.cancel(); copyTask.values.forEach { $0.cancel() }; copyTask.removeAll()
        loading = false; copied.removeAll()
    }
}
