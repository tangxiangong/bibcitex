import AppKit
import Combine
import SwiftUI

protocol HelperServing: Sendable {
    func loadBibliographyStateIfAvailable() async throws -> (bibs: [Bibliography], current: Bibliography?)
    func setCurrentBibliography(name: String, path: String) async throws -> Bibliography
    func searchReferences(query: String) async throws -> [Reference]
    func copyAndPaste(citeKey: String) async throws
}

@MainActor
final class HelperViewModel: ObservableObject {
    var onPasteFailure: (() -> Void)?
    @Published var isPasting = false
    @Published private(set) var isPasteOperationPending = false
    @Published private(set) var bibliographies: [Bibliography] = []
    @Published private(set) var searchResults: [Reference] = []
    @Published private(set) var currentBibliography: Bibliography?
    @Published private(set) var isSelectingBibliography = true
    @Published private(set) var selectedBibliographyIndex: Int?
    @Published private(set) var selectedReferenceIndex: Int?
    @Published private(set) var preferredHeight: CGFloat = 56

    @Published var focusRequest = 0
    @Published private(set) var isSearching = false
    @Published private(set) var isLoading = false
    @Published var query = ""
    @Published var errorMessage: String?
    @Published var failedPasteKey: String?
    @Published var theme: ThemeStyle = .latte

    private let service: any HelperServing
    private var searchTask: Task<Void, Never>?
    private var queryEditTask: Task<Void, Never>?
    private var queryEditVersion = 0
    private var searchVersion = 0
    private var stateVersion = 0
    private(set) var sessionGeneration = 0

    /// Invalidates UI completions; an already-started OS paste cannot be cancelled here.
    func invalidateSession() {
        invalidateQueryEdit()
        sessionGeneration += 1
        isPasting = false
        invalidateSearch()
        stateVersion += 1
        isLoading = false
    }

    @discardableResult
    func beginSession() -> Int {
        invalidateSession()
        return sessionGeneration
    }

    private let rowHeightBib = 64.0
    private let rowHeightSearch = 92.0
    private let headerHeight = 56.0
    private let minHeight = 56.0
    private let maxListHeight = 540.0

    var filteredBibliographies: [Bibliography] {
        guard query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false else {
            return bibliographies
        }
        let normalized = query.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        return bibliographies.filter { bib in
            [bib.name, bib.path, bib.updatedAt, bib.descriptionText ?? ""].contains(where: {
                $0.lowercased().contains(normalized)
            })
        }
    }

    init(service: any HelperServing) {
        self.service = service
    }

    func setTheme(_ themeMode: Int32) {
        updateTheme(ThemeStyle(mode: themeMode))
    }

    func loadState() {
        isLoading = true
        invalidateSearch()
        stateVersion += 1
        let version = stateVersion
        query = ""
        searchResults = []
        selectedReferenceIndex = nil
        errorMessage = nil
        failedPasteKey = nil
        Task {
            do {
                let state = try await service.loadBibliographyStateIfAvailable()
                guard version == stateVersion else { return }
                isLoading = false
                bibliographies = state.bibs.sorted {
                    $0.updatedAt.localizedStandardCompare($1.updatedAt) == .orderedDescending
                }
                if let current = state.current,
                    let index = bibliographies.firstIndex(where: { $0.id == current.id }) {
                    currentBibliography = bibliographies[index]
                    isSelectingBibliography = false
                    selectedBibliographyIndex = index
                } else {
                    currentBibliography = nil
                    isSelectingBibliography = true
                    selectedBibliographyIndex = bibliographies.isEmpty ? nil : 0
                }
                updateQuery(query)
            } catch {
                guard version == stateVersion else { return }
                isLoading = false
                errorMessage = "加载文献库失败：\(error.localizedDescription)"
                isSelectingBibliography = true
                recalcHeight()
            }
        }
    }

    func updateTheme(_ style: ThemeStyle) {
        guard theme != style else { return }
        theme = style
    }

    /// A TextField may call its binding setter while SwiftUI reconciles the view.
    /// Coalesce edits outside that call stack, and discard edits from an old session/library.
    func requestQuery(_ value: String) {
        invalidateQueryEdit()
        guard value != query else { return }
        let edit = queryEditVersion
        let session = sessionGeneration
        let state = stateVersion
        queryEditTask = Task { @MainActor [weak self] in
            guard let self, !Task.isCancelled,
                edit == queryEditVersion, session == sessionGeneration, state == stateVersion else { return }
            queryEditTask = nil
            updateQuery(value)
        }
    }

    private func invalidateQueryEdit() {
        queryEditVersion += 1
        queryEditTask?.cancel()
        queryEditTask = nil
    }

    func updateQuery(_ nextQuery: String) {
        invalidateQueryEdit()
        query = nextQuery
        failedPasteKey = nil
        errorMessage = nil

        invalidateSearch()
        searchResults = []
        selectedReferenceIndex = nil

        if isSelectingBibliography {
            if filteredBibliographies.isEmpty {
                selectedBibliographyIndex = nil
            } else {
                selectedBibliographyIndex = 0
            }
            recalcHeight()
            return
        }

        let trimmed = nextQuery.trimmingCharacters(in: .whitespacesAndNewlines)
        let version = searchVersion
        if !trimmed.isEmpty {
            isSearching = true
            searchTask = Task {
                do {
                    try await Task.sleep(nanoseconds: 90_000_000)
                    try Task.checkCancellation()
                    await performSearch(trimmed, version: version)
                } catch {
                    // A new query or bibliography superseded this search.
                }
            }
        }

        recalcHeight()
    }

    func startSelectMode() {
        isLoading = false
        invalidateSearch()
        stateVersion += 1
        isSelectingBibliography = true
        query = ""
        searchResults = []
        selectedReferenceIndex = nil
        selectedBibliographyIndex = bibliographies.isEmpty ? nil : 0
        errorMessage = nil
        failedPasteKey = nil
        recalcHeight()
    }

    func chooseCurrentBibliography() {
        guard !isLoading, isSelectingBibliography, let index = selectedBibliographyIndex else {
            return
        }
        let list = filteredBibliographies
        guard list.indices.contains(index) else {
            return
        }
        selectBibliography(list[index])
    }

    func selectBibliography(_ bibliography: Bibliography, at index: Int? = nil) {
        invalidateSearch()
        stateVersion += 1
        let version = stateVersion
        isLoading = true
        if let index {
            selectedBibliographyIndex = index
        }
        query = ""
        selectedReferenceIndex = nil
        failedPasteKey = nil
        errorMessage = nil

        Task { @MainActor in
            do {
                let current = try await service.setCurrentBibliography(name: bibliography.name, path: bibliography.path)
                guard version == stateVersion else { return }
                isLoading = false
                currentBibliography = current
                isSelectingBibliography = false
                selectedReferenceIndex = nil
                searchResults = []
                updateQuery(query)
            } catch {
                guard version == stateVersion else { return }
                isLoading = false
                errorMessage = "加载失败：\(error.localizedDescription)"
                recalcHeight()
            }
        }
    }

    func moveSelection(delta: Int) {
        guard delta != 0 else { return }

        if isSelectingBibliography {
            let items = filteredBibliographies
            guard !items.isEmpty else {
                selectedBibliographyIndex = nil
                return
            }

            let start = selectedBibliographyIndex ?? (delta > 0 ? -1 : 0)
            let count = items.count
            let next = (start + delta + count) % count
            selectedBibliographyIndex = next
            return
        }

        guard !searchResults.isEmpty else {
            selectedReferenceIndex = nil
            return
        }
        let start = selectedReferenceIndex ?? (delta > 0 ? -1 : 0)
        let count = searchResults.count
        let next = (start + delta + count) % count
        selectedReferenceIndex = next
    }

    func activateSelection(onSuccess: @escaping () -> Void) {
        if isSelectingBibliography {
            chooseCurrentBibliography()
            return
        }

        guard let index = selectedReferenceIndex, searchResults.indices.contains(index) else {
            return
        }
        let reference = searchResults[index]
        copyAndPaste(reference, onSuccess: onSuccess)
    }

    func copyAndPaste(_ reference: Reference, at index: Int? = nil, onSuccess: (() -> Void)? = nil) {
        if let index {
            selectedReferenceIndex = index
        }
        guard !isPasteOperationPending else { return }
        let key = reference.citeKey
        let session = sessionGeneration
        isPasting = true
        isPasteOperationPending = true
        failedPasteKey = nil
        errorMessage = nil

        Task { @MainActor in
            defer { isPasteOperationPending = false }
            guard session == sessionGeneration else { return }
            do {
                try await service.copyAndPaste(citeKey: key)
                guard session == sessionGeneration else { return }
                isPasting = false
                onSuccess?()
            } catch {
                guard session == sessionGeneration else { return }
                isPasting = false
                failedPasteKey = key
                errorMessage = "粘贴失败，可复制重试: \(error.localizedDescription)"
                recalcHeight()
                onPasteFailure?()
            }
        }
    }

    func copyFailedKeyAgain() {
        guard let key = failedPasteKey else {
            return
        }
        copyAndPasteKeyToClipboard(key)
    }

    private func copyAndPasteKeyToClipboard(_ key: String) {
        guard !key.isEmpty else { return }
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(key, forType: .string)
    }

    private func performSearch(_ queryText: String, version: Int) async {
        do {
            let found = try await service.searchReferences(query: queryText)
            guard version == searchVersion, !Task.isCancelled, !isSelectingBibliography else { return }
            isSearching = false
            searchResults = found
            selectedReferenceIndex = found.isEmpty ? nil : 0
            errorMessage = nil
            failedPasteKey = nil
            recalcHeight()
        } catch {
            guard version == searchVersion, !Task.isCancelled, !isSelectingBibliography else { return }
            searchResults = []
            selectedReferenceIndex = nil
            isSearching = false
            errorMessage = "搜索失败：\(error.localizedDescription)"
            recalcHeight()
        }
    }

    private func invalidateSearch() {
        isSearching = false
        searchTask?.cancel()
        searchTask = nil
        searchVersion += 1
    }

    func recalcHeight() {
        let listHeight: CGFloat

        if isLoading {
            listHeight = 112
        } else if isSelectingBibliography && filteredBibliographies.isEmpty {
            listHeight = 112
        } else if isSelectingBibliography {
            let count = filteredBibliographies.count
            listHeight = min(CGFloat(count) * (rowHeightBib + 2) + 16, maxListHeight)
        } else if query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            listHeight = 0
        } else {
            if searchResults.isEmpty {
                listHeight = 128
            } else {
                listHeight = min(CGFloat(searchResults.count) * (rowHeightSearch + 2) + 16, maxListHeight)
            }
        }

        let errorHeight = (errorMessage == nil ? 0 : 40)
        let fallbackHeight = (failedPasteKey == nil ? 0 : 34)
        let height = max(minHeight, headerHeight + listHeight + Double(errorHeight) + Double(fallbackHeight))
        if preferredHeight != height { preferredHeight = height }
    }
}
