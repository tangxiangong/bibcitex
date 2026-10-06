import Combine
import Foundation

private actor WorkbenchTestService: WorkbenchServing {
    struct Search: Equatable, Sendable {
        let path: String
        let query: String
        let field: String
        let type: String
    }
    private(set) var searches: [Search] = []
    private var pending: [(Search, CheckedContinuation<[Reference], Error>)] = []
    func libraries() -> [Bibliography] {
        [Bibliography(name: "A", path: "/tmp/a/references.bib", updatedAt: "", descriptionText: nil),
         Bibliography(name: "B", path: "/tmp/b/references.bib", updatedAt: "", descriptionText: nil)]
    }
    func search(path: String, query: String, field: String, type: String) async throws -> [Reference] {
        let request = Search(path: path, query: query, field: field, type: type)
        searches.append(request)
        return try await withCheckedThrowingContinuation { pending.append((request, $0)) }
    }
    func waitForSearch(path: String, query: String) async {
        for _ in 0..<200 {
            if pending.contains(where: { $0.0.path == path && $0.0.query == query }) { return }
            try? await Task.sleep(nanoseconds: 5_000_000)
        }
        fatalError("Timed out waiting for search")
    }
    func complete(path: String, query: String, references: [Reference]) {
        let index = pending.firstIndex { $0.0.path == path && $0.0.query == query }!
        pending.remove(at: index).1.resume(returning: references)
    }
    func removeLibrary(name: String) {}
    func copy(_ text: String) {}
}

@main private struct WorkbenchRegression {
    @MainActor static func wait(_ condition: () -> Bool) async {
        for _ in 0..<200 {
            if condition() { return }
            try? await Task.sleep(nanoseconds: 5_000_000)
        }
        fatalError("Timed out waiting for model state")
    }
    static func reference(_ key: String) -> Reference {
        Reference(citeKey: key, source: "@article{\(key)}", typeKind: .article, typeUnknown: nil,
            author: [], title: [], journal: "", year: nil, fullJournal: "", volume: nil,
            number: "", pages: nil, note: [], doi: "", mrclass: "", publisher: [],
            series: "", isbn: "", url: "", file: "paper.pdf", abstractChunks: [], edition: nil,
            issue: [], bookPages: "", school: "", address: "", bookTitle: [], editor: [],
            month: "", organization: [], institution: "", eprint: "", archivePrefix: "",
            arxivPrimaryClass: "", howPublished: "")
    }
    @MainActor static func main() async {
        let suite = "BibCiTeX.WorkbenchTests." + UUID().uuidString
        let preferences = UserDefaults(suiteName: suite)!
        defer { preferences.removePersistentDomain(forName: suite) }
        let service = WorkbenchTestService()
        let model = WorkbenchModel(service: service, preferences: preferences)
        precondition(model.showSidebar && !model.showInspector, "Fresh workbench opens only the left pane")
        model.showSidebar = false
        model.showInspector = true
        let restored = WorkbenchModel(service: service, preferences: preferences)
        precondition(!restored.showSidebar && restored.showInspector, "Both pane choices survive model recreation")
        restored.showSidebar = true
        restored.showInspector = false
        let restoredAgain = WorkbenchModel(service: service, preferences: preferences)
        precondition(restoredAgain.showSidebar && !restoredAgain.showInspector, "Saved false is distinct from an absent preference")
        let a = "/tmp/a/references.bib", b = "/tmp/b/references.bib"
        model.moveReference(1)
        model.moveReference(-1)
        precondition(model.selectedReference == nil, "Empty lists ignore navigation")
        await model.reload()
        await service.waitForSearch(path: a, query: "")
        await service.complete(path: a, query: "", references: [reference("shared"), reference("a-second")])
        await wait { !model.loading && model.references.count == 2 }
        model.moveReference(-1)
        precondition(model.selectedReference == "shared")
        model.moveReference(1)
        precondition(model.selectedReference == "a-second")
        model.moveReference(1)
        precondition(model.selectedReference == "a-second", "Navigation clamps at the last row")
        model.moveReference(-1)
        precondition(model.selectedReference == "shared")
        let aContext = model.referenceSelectionContext
        precondition(model.attachmentURL(for: reference("shared"), context: aContext)?.path == "/tmp/a/paper.pdf")

        var publications = 0
        let observation = model.objectWillChange.sink { publications += 1 }
        let baseline = publications
        model.selectReference("a-second", context: aContext)
        model.setQuery("slow")
        precondition(publications == baseline, "Binding setters must not synchronously publish")
        model.moveReference(1)
        precondition(model.query.isEmpty && model.selectedReference == "shared", "Pending search blocks stale navigation")
        await service.waitForSearch(path: a, query: "slow")

        let beforeSwitch = publications
        model.selectLibrary("B")
        model.setQuery("discarded")
        model.setQuery("latest")
        model.setField("title")
        model.setType("Article")
        precondition(publications == beforeSwitch, "Derived state must not publish on the binding's update stack")
        precondition(model.selectedLibrary == "A" && model.query == "slow")
        precondition(model.attachmentURL(for: reference("shared"), context: aContext) == nil,
                     "Pending library switches must immediately disable stale attachment context")
        await service.waitForSearch(path: b, query: "latest")
        let bSearches = await service.searches.filter { $0.path == b }
        precondition(bSearches.count == 1 && bSearches[0].field == "title" && bSearches[0].type == "Article")
        await service.complete(path: b, query: "latest", references: [reference("b-first"), reference("shared")])
        await wait { !model.loading && model.selectedReference == "b-first" }
        await service.complete(path: a, query: "slow", references: [reference("stale")])
        try? await Task.sleep(nanoseconds: 30_000_000)
        precondition(model.selectedLibrary == "B" && model.references.map(\.citeKey) == ["b-first", "shared"])
        print("PASS deferred binding publication, coalesced inputs and stale-search suppression")

        let beforeOldRow = publications
        model.selectReference("shared", context: aContext)
        precondition(publications == beforeOldRow)
        await Task.yield()
        precondition(model.selectedReference == "b-first", "Old rows cannot select the same cite key in a new library")
        precondition(model.attachmentURL(for: reference("shared"), context: aContext) == nil)
        let bContext = model.referenceSelectionContext
        precondition(model.attachmentURL(for: reference("shared"), context: bContext)?.path == "/tmp/b/paper.pdf")
        model.selectReference("shared", context: bContext)
        precondition(model.selectedReference == "b-first")
        await wait { model.selectedReference == "shared" }
        let beforeNoOp = publications
        model.selectReference("shared", context: bContext)
        model.selectLibrary("B")
        model.setQuery("latest")
        model.setField("title")
        model.setType("Article")
        await Task.yield()
        precondition(publications == beforeNoOp, "Equal binding values must not restart work or publish")
        model.selectReference(nil, context: bContext)
        await wait { model.selectedReference == nil }
        model.moveReference(-1)
        precondition(model.selectedReference == "shared", "Up without selection starts at the last row")
        model.selectReference(nil, context: bContext)
        await wait { model.selectedReference == nil }
        model.moveReference(1)
        precondition(model.selectedReference == "b-first", "Down without selection starts at the first row")
        print("PASS keyboard navigation, boundaries, empty lists and pending-search protection")
        withExtendedLifetime(observation) {}
        print("PASS library-scoped row selection, attachment context and equal-value suppression")
    }
}
