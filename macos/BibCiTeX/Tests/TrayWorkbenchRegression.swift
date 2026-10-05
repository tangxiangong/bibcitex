import Foundation

private actor TrayTestService: WorkbenchServing {
    struct Request: Equatable { let path: String; let query: String; let field: String; let type: String }
    private var pending: [(Request, CheckedContinuation<[Reference], Error>)] = []
    private(set) var copied: [String] = []
    func libraries() -> [Bibliography] {
        [Bibliography(name: "A", path: "/tmp/a.bib", updatedAt: "", descriptionText: nil),
         Bibliography(name: "B", path: "/tmp/b.bib", updatedAt: "", descriptionText: nil)]
    }
    func search(path: String, query: String, field: String, type: String) async throws -> [Reference] {
        try await withCheckedThrowingContinuation { pending.append((Request(path: path, query: query, field: field, type: type), $0)) }
    }
    func waitFor(_ request: Request) async {
        for _ in 0..<200 {
            if pending.contains(where: { $0.0 == request }) { return }
            try? await Task.sleep(nanoseconds: 5_000_000)
        }
        fatalError("Missing request: \(request)")
    }
    func finish(_ request: Request, _ rows: [Reference]) {
        let index = pending.firstIndex { $0.0 == request }!
        pending.remove(at: index).1.resume(returning: rows)
    }
    func copy(_ text: String) { copied.append(text) }
    func removeLibrary(name: String) {}
}

@main private struct TrayWorkbenchRegression {
    @MainActor static func wait(_ condition: () -> Bool) async {
        for _ in 0..<200 {
            if condition() { return }
            try? await Task.sleep(nanoseconds: 5_000_000)
        }
        fatalError("Timed out waiting for tray state")
    }
    static func row(_ key: String) -> Reference {
        Reference(citeKey: key, source: "@article{\(key)}", typeKind: .article, typeUnknown: nil,
            author: [], title: [], journal: "", year: nil, fullJournal: "", volume: nil,
            number: "", pages: nil, note: [], doi: "", mrclass: "", publisher: [],
            series: "", isbn: "", url: "", file: "", abstractChunks: [], edition: nil,
            issue: [], bookPages: "", school: "", address: "", bookTitle: [], editor: [],
            month: "", organization: [], institution: "", eprint: "", archivePrefix: "",
            arxivPrimaryClass: "", howPublished: "")
    }
    @MainActor static func main() async {
        let suite = "BibCiTeX.TrayTests." + UUID().uuidString
        let preferences = UserDefaults(suiteName: suite)!
        defer { preferences.removePersistentDomain(forName: suite) }
        preferences.set("B", forKey: "mainLibrary")
        let service = TrayTestService()
        let model = TrayWorkbenchModel(service: service, preferences: preferences)
        model.move(1)
        precondition(model.selection == nil)
        await model.reload()
        let initial = TrayTestService.Request(path: "/tmp/a.bib", query: "", field: "all", type: "all")
        await service.waitFor(initial)
        await service.finish(initial, [row("first"), row("last")])
        await wait { !model.loading && model.references.count == 2 }
        precondition(model.reference?.citeKey == "first", "Empty search must browse all records")
        model.move(-1); precondition(model.selection == "first")
        model.move(1); model.move(1); precondition(model.selection == "last")
        model.copy("last")
        await wait { model.copied == "last" }
        precondition(model.reference?.citeKey == "last", "Copy must keep the selected record")
        model.query = "old"
        let old = TrayTestService.Request(path: "/tmp/a.bib", query: "old", field: "all", type: "all")
        precondition(model.reference == nil, "Pending searches disable old detail actions immediately")
        await service.waitFor(old)
        model.libraryName = "B"; model.query = "new"; model.field = "title"; model.type = "Article"
        let current = TrayTestService.Request(path: "/tmp/b.bib", query: "new", field: "title", type: "Article")
        await service.waitFor(current)
        await service.finish(current, [row("current")])
        await wait { !model.loading && model.reference?.citeKey == "current" }
        await service.finish(old, [row("stale")])
        try? await Task.sleep(nanoseconds: 20_000_000)
        precondition(model.reference?.citeKey == "current")
        model.query = "closing"
        let closing = TrayTestService.Request(path: "/tmp/b.bib", query: "closing", field: "title", type: "Article")
        await service.waitFor(closing)
        model.suspend()
        await service.finish(closing, [row("late")])
        try? await Task.sleep(nanoseconds: 20_000_000)
        precondition(model.reference == nil && !model.loading)
        precondition(preferences.string(forKey: "mainLibrary") == "B")
        precondition(preferences.string(forKey: "trayLibrary") == "B")
        let fresh = TrayWorkbenchModel(service: service, preferences: preferences)
        precondition(fresh.query.isEmpty && fresh.selection == nil, "Tray instances own independent UI state")
        print("PASS tray empty-query browsing, filters, navigation, copy, stale searches, close lifecycle and isolated state")
    }
}
