import AppKit
import Combine
import SwiftUI

@MainActor
final class HelperViewModel: ObservableObject {
    @Published private(set) var bibliographies: [NativeHelperBibliography] = []
    @Published private(set) var searchResults: [NativeHelperReference] = []
    @Published private(set) var currentBibliography: NativeHelperBibliography?
    @Published private(set) var isSelectingBibliography = true
    @Published private(set) var selectedBibliographyIndex: Int?
    @Published private(set) var selectedReferenceIndex: Int?
    @Published private(set) var preferredHeight: CGFloat = 180

    @Published var query = ""
    @Published var errorMessage: String?
    @Published var failedPasteKey: String?
    @Published var theme: NativeHelperThemeStyle = .latte

    private let service: NativeHelperService
    private var searchTask: Task<Void, Never>?
    private var searchVersion = 0

    private let rowHeightBib = 64.0
    private let rowHeightSearch = 92.0
    private let headerHeight = 64.0
    private let minHeight = 180.0
    private let maxListHeight = 540.0

    var filteredBibliographies: [NativeHelperBibliography] {
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

    init(service: NativeHelperService) {
        self.service = service
    }

    func setTheme(_ themeMode: Int32) {
        theme = NativeHelperThemeStyle(mode: themeMode)
        recalcHeight()
    }

    func loadState() {
        Task { @MainActor in
            do {
                let state = try service.loadBibliographyStateIfAvailable()
                bibliographies = state.bibs.sorted {
                    $0.updatedAt.localizedStandardCompare($1.updatedAt) == .orderedDescending
                }
                currentBibliography = state.current

                if let current = state.current,
                    let index = bibliographies.firstIndex(of: current) {
                    isSelectingBibliography = false
                    query = ""
                    selectedReferenceIndex = 0
                    selectedBibliographyIndex = index
                } else {
                    isSelectingBibliography = true
                    selectedBibliographyIndex = bibliographies.isEmpty ? nil : 0
                    selectedReferenceIndex = nil
                }
                searchResults = []
                errorMessage = nil
                failedPasteKey = nil
                recalcHeight()
            } catch {
                errorMessage = "加载文献库失败：\(error.localizedDescription)"
                isSelectingBibliography = true
                recalcHeight()
            }
        }
    }

    func updateTheme(_ style: NativeHelperThemeStyle) {
        theme = style
    }

    func updateQuery(_ nextQuery: String) {
        query = nextQuery
        failedPasteKey = nil
        errorMessage = nil

        searchTask?.cancel()

        if isSelectingBibliography {
            if filteredBibliographies.isEmpty {
                selectedBibliographyIndex = nil
            } else {
                selectedBibliographyIndex = 0
            }
            recalcHeight()
            return
        }

        searchTask = Task { [query = nextQuery] in
            let trimmed = query.trimmingCharacters(in: .whitespacesAndNewlines)
            let currentVersion = nextSearchVersion()

            if trimmed.isEmpty {
                await MainActor.run {
                    searchResults = []
                    selectedReferenceIndex = nil
                    recalcHeight()
                }
                return
            }

            do {
                try await Task.sleep(for: .milliseconds(90))
            } catch {
                return
            }
            await performSearch(trimmed, version: currentVersion)
        }

        recalcHeight()
    }

    func startSelectMode() {
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
        guard isSelectingBibliography, let index = selectedBibliographyIndex else {
            return
        }
        let list = filteredBibliographies
        guard list.indices.contains(index) else {
            return
        }
        selectBibliography(list[index])
    }

    func selectBibliography(_ bibliography: NativeHelperBibliography) {
        query = ""
        selectedReferenceIndex = nil
        failedPasteKey = nil
        errorMessage = nil

        Task { @MainActor in
            do {
                let current = try service.setCurrentBibliography(name: bibliography.name, path: bibliography.path)
                currentBibliography = current
                isSelectingBibliography = false
                selectedReferenceIndex = nil
                searchResults = []
                recalcHeight()
                doSearchNow()
            } catch {
                errorMessage = "加载失败：\(error.localizedDescription)"
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

            let start = selectedBibliographyIndex ?? 0
            let count = items.count
            let next = (start + delta + count) % count
            selectedBibliographyIndex = next
            return
        }

        guard !searchResults.isEmpty else {
            selectedReferenceIndex = nil
            return
        }
        let start = selectedReferenceIndex ?? 0
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

    func copyAndPaste(_ reference: NativeHelperReference, onSuccess: (() -> Void)? = nil) {
        let key = reference.citeKey
        failedPasteKey = nil
        errorMessage = nil

        Task { @MainActor in
            do {
                try service.copyAndPaste(citeKey: key)
                await MainActor.run {
                    onSuccess?()
                }
            } catch {
                failedPasteKey = key
                errorMessage = "粘贴失败，可复制重试: \(error.localizedDescription)"
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

    private func doSearchNow() {
        let normalized = query.trimmingCharacters(in: .whitespacesAndNewlines)
        if normalized.isEmpty {
            searchResults = []
            selectedReferenceIndex = nil
            recalcHeight()
            return
        }
        Task {
            await performSearch(normalized, version: nextSearchVersion())
        }
    }

    private func performSearch(_ queryText: String, version: Int) async {
        do {
            let found = try service.searchReferences(query: queryText)
            await MainActor.run {
                guard version == searchVersion else { return }
                searchResults = found
                selectedReferenceIndex = found.isEmpty ? nil : 0
                errorMessage = nil
                failedPasteKey = nil
                recalcHeight()
            }
        } catch {
            await MainActor.run {
                searchResults = []
                selectedReferenceIndex = nil
                errorMessage = "搜索失败：\(error.localizedDescription)"
                recalcHeight()
            }
        }
    }

    private func nextSearchVersion() -> Int {
        searchVersion += 1
        return searchVersion
    }

    func recalcHeight() {
        let listHeight: CGFloat

        if isSelectingBibliography {
            let count = max(1, filteredBibliographies.count)
            listHeight = min(CGFloat(count) * rowHeightBib, maxListHeight)
        } else if query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            listHeight = rowHeightSearch
        } else {
            if searchResults.isEmpty {
                listHeight = 128
            } else {
                listHeight = min(CGFloat(searchResults.count) * rowHeightSearch, maxListHeight)
            }
        }

        let errorHeight = (errorMessage == nil ? 0 : 40)
        let fallbackHeight = (failedPasteKey == nil ? 0 : 34)
        preferredHeight = max(minHeight, headerHeight + listHeight + Double(errorHeight) + Double(fallbackHeight))
    }
}
