import AppKit
import SwiftUI

/// State owned only by the tray workbench. The Rust service is shared; UI state is not.
@MainActor
final class TrayWorkbenchModel: ObservableObject {
    @Published private(set) var libraries: [Bibliography] = []
    @Published var libraryName: String? { didSet { if oldValue != libraryName { search() } } }
    @Published var query = "" { didSet { if oldValue != query { search() } } }
    @Published var field = "all" { didSet { if oldValue != field { search() } } }
    @Published var type = "all" { didSet { if oldValue != type { search() } } }
    @Published private(set) var references: [Reference] = []
    @Published var selection: String?
    @Published private(set) var loading = false
    @Published private(set) var copied: String?
    @Published var error: String?
    private let service: any WorkbenchServing
    private let preferences: UserDefaults
    private var searchTask: Task<Void, Never>?
    private var copyTask: Task<Void, Never>?
    private var searchVersion = 0
    private var loadedVersion = -1
    private var reloadVersion = 0
    private var session = 0

    init(service: any WorkbenchServing, preferences: UserDefaults = .standard) {
        self.service = service
        self.preferences = preferences
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
            let selected = rows.first { $0.name == preferred }?.name ?? rows.first?.name
            if libraryName != selected { libraryName = selected } else { search() }
        } catch {
            guard version == reloadVersion else { return }
            references = []
            selection = nil
            loading = false
            self.error = error.localizedDescription
        }
    }
    private func search() {
        searchVersion += 1
        let version = searchVersion
        let library = libraries.first { $0.name == libraryName }
        let query = query, field = field, type = type
        searchTask?.cancel()
        searchTask = Task { @MainActor [weak self] in
            guard let self, !Task.isCancelled else { return }
            loading = true
            error = nil
            guard let library else {
                references = []; selection = nil; loading = false
                return
            }
            preferences.set(library.name, forKey: "trayLibrary")
            do {
                try await Task.sleep(nanoseconds: 100_000_000)
                let rows = try await service.search(path: library.path, query: query, field: field, type: type)
                guard !Task.isCancelled, version == searchVersion else { return }
                references = rows
                loadedVersion = version
                if !rows.contains(where: { $0.citeKey == self.selection }) { selection = rows.first?.citeKey }
                loading = false
            } catch {
                guard !Task.isCancelled, version == searchVersion else { return }
                references = []; selection = nil; loading = false
                self.error = error.localizedDescription
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
    func copy(_ value: String) {
        let currentSession = session
        copyTask?.cancel()
        copyTask = Task { @MainActor [weak self] in
            guard let self else { return }
            do {
                try await service.copy(value)
                guard !Task.isCancelled, currentSession == session else { return }
                copied = value
                try await Task.sleep(nanoseconds: 1_500_000_000)
                if !Task.isCancelled { copied = nil }
            } catch {
                guard !Task.isCancelled, currentSession == session else { return }
                self.error = error.localizedDescription
            }
        }
    }
    func suspend() {
        session += 1; reloadVersion += 1; searchVersion += 1
        searchTask?.cancel(); copyTask?.cancel()
        loading = false; copied = nil
    }
}
