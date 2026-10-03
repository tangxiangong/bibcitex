import Foundation

private actor TestService: HelperServing {
    private var current: Bibliography?
    private var pending: CheckedContinuation<[Reference], Error>?
    private(set) var searchedOnMain = false
    private(set) var didStartSearch = false
    private var pasteFails = true
    private var delayPaste = false
    private var pendingPastes: [CheckedContinuation<Void, Error>] = []
    private(set) var pastedKeys: [String] = []
    var bibliography: Bibliography {
        Bibliography(name: "Research", path: "/tmp/research.bib", updatedAt: "2026-10-03", descriptionText: nil)
    }
    func loadBibliographyStateIfAvailable() -> (bibs: [Bibliography], current: Bibliography?) {
        ([bibliography], current == nil ? nil : bibliography)
    }
    func setCurrentBibliography(name: String, path: String) -> Bibliography {
        current = bibliography
        return bibliography
    }
    func searchReferences(query: String) async throws -> [Reference] {
        recordSearchThread()
        didStartSearch = true
        return try await withCheckedThrowingContinuation { pending = $0 }
    }
    private func recordSearchThread() {
        searchedOnMain = Thread.isMainThread
    }
    func finishSearchWithError() {
        pending?.resume(throwing: HelperError.ffiError("stale search"))
        pending = nil
    }
    func setPasteFailure(_ fails: Bool) { pasteFails = fails }
    func setDelayedPaste() { delayPaste = true }
    func waitForPendingPastes(_ count: Int) async {
        for _ in 0..<200 {
            if pendingPastes.count == count { return }
            try? await Task.sleep(nanoseconds: 5_000_000)
        }
        fatalError("Timed out waiting for paste calls")
    }
    func completeOldestPaste(fails: Bool) {
        let pending = pendingPastes.removeFirst()
        if fails { pending.resume(throwing: HelperError.ffiError("delayed paste failure")) }
        else { pending.resume() }
    }
    func copyAndPaste(citeKey: String) async throws {
        pastedKeys.append(citeKey)
        if delayPaste {
            try await withCheckedThrowingContinuation { pendingPastes.append($0) }
        } else if pasteFails { throw HelperError.ffiError("test paste failure") }
    }
}

@main private struct Regression {
    @MainActor static func wait(_ condition: () -> Bool) async {
        for _ in 0..<200 {
            if condition() { return }
            try? await Task.sleep(nanoseconds: 5_000_000)
        }
        fatalError("Timed out waiting for model state")
    }
    @MainActor static func main() async {
        let service = TestService()
        let model = HelperViewModel(service: service)
        let first = await service.bibliography
        let second = await service.bibliography
        precondition(first.id == second.id, "Library identity must survive reload")
        model.loadState()
        await wait { !model.isLoading && !model.bibliographies.isEmpty }
        model.chooseCurrentBibliography()
        await wait { !model.isSelectingBibliography && !model.isLoading }
        model.updateQuery("old query")
        model.loadState()
        await wait { !model.isLoading }
        precondition(!model.isSelectingBibliography, "Reopening must restore the selected library")
        precondition(model.query.isEmpty, "Reopening must clear the query")
        precondition(model.currentBibliography?.id == first.id)
        print("PASS stable identity, library restoration, query reset")

        model.updateQuery("slow search")
        for _ in 0..<200 {
            if await service.didStartSearch { break }
            try? await Task.sleep(nanoseconds: 5_000_000)
        }
        let started = await service.didStartSearch
        let onMain = await service.searchedOnMain
        precondition(started && !onMain, "Search must leave the UI executor")
        model.startSelectMode()
        await service.finishSearchWithError()
        try? await Task.sleep(nanoseconds: 30_000_000)
        precondition(model.errorMessage == nil, "Obsolete searches must not replace the current UI")
        precondition(!model.isSearching && model.isSelectingBibliography)
        print("PASS background search, cancellation, stale error suppression")

        model.updateQuery("no matching library")
        precondition(model.filteredBibliographies.isEmpty)
        model.loadState()
        await wait { !model.isLoading }
        precondition(model.filteredBibliographies.count == 1 && model.query.isEmpty)
        precondition(model.preferredHeight == 56, "Empty search must use the compact panel")
        print("PASS filtered-library reopen and compact panel height")

        let reference = Reference(
            citeKey: "key_raw", source: "", typeKind: .article, typeUnknown: nil,
            author: [], title: [], journal: "", year: nil, fullJournal: "", volume: nil,
            number: "", pages: nil, note: [], doi: "", mrclass: "", publisher: [],
            series: "", isbn: "", url: "", file: "", abstractChunks: [], edition: nil,
            issue: [], bookPages: "", school: "", address: "", bookTitle: [], editor: [],
            month: "", organization: [], institution: "", eprint: "", archivePrefix: "",
            arxivPrimaryClass: "", howPublished: ""
        )
        var didHide = false
        var restoredFailures = 0
        model.onPasteFailure = { restoredFailures += 1 }
        model.copyAndPaste(reference, onSuccess: { didHide = true })
        precondition(model.isPasting)
        model.copyAndPaste(reference, onSuccess: { didHide = true })
        await wait { !model.isPasting }
        precondition(!didHide && restoredFailures == 1)
        precondition(model.failedPasteKey == "key_raw")
        precondition(model.errorMessage?.contains("test paste failure") == true)
        let failedKeys = await service.pastedKeys
        precondition(failedKeys == ["key_raw"], "Concurrent activation must not paste twice")
        await service.setPasteFailure(false)
        model.copyAndPaste(reference, onSuccess: { didHide = true })
        await wait { !model.isPasting }
        precondition(didHide && model.failedPasteKey == nil && model.errorMessage == nil)
        let keys = await service.pastedKeys
        precondition(keys == ["key_raw", "key_raw"], "Paste must keep the plain cite key")
        print("PASS paste failure restore, retained key, duplicate suppression and successful retry")

        await service.setDelayedPaste()
        for oldFails in [false, true] {
            model.beginSession()
            var oldDidHide = false
            var newDidHide = false
            let failuresBefore = restoredFailures
            model.copyAndPaste(reference, onSuccess: { oldDidHide = true })
            await service.waitForPendingPastes(1)
            model.invalidateSession() // Escape closes the old helper session.
            model.beginSession() // Reopening creates a distinct visible session.
            model.loadState()
            await wait { !model.isLoading }
            precondition(model.isPasteOperationPending, "Hiding must not pretend the OS operation was cancelled")
            model.copyAndPaste(reference, onSuccess: { newDidHide = true })
            await service.waitForPendingPastes(1)
            await service.completeOldestPaste(fails: oldFails)
            await wait { !model.isPasteOperationPending }
            precondition(!oldDidHide && !newDidHide, "Old completion must not hide the reopened panel")
            precondition(restoredFailures == failuresBefore, "Old failure must not restore/focus a new session")
            precondition(!model.isPasting, "New paste must wait for the old OS operation to finish")
            precondition(model.failedPasteKey == nil && model.errorMessage == nil)
            model.copyAndPaste(reference, onSuccess: { newDidHide = true })
            await service.waitForPendingPastes(1)
            await service.completeOldestPaste(fails: false)
            await wait { !model.isPasting }
            precondition(newDidHide && !oldDidHide)
        }
        let failuresBeforeClose = restoredFailures
        model.beginSession()
        model.copyAndPaste(reference, onSuccess: { preconditionFailure("Closed session must stay hidden") })
        await service.waitForPendingPastes(1)
        model.invalidateSession()
        await service.completeOldestPaste(fails: true)
        try? await Task.sleep(nanoseconds: 30_000_000)
        precondition(restoredFailures == failuresBeforeClose && !model.isPasting)
        precondition(model.failedPasteKey == nil && model.errorMessage == nil)
        print("PASS dismissed/reopened sessions ignore delayed paste success/failure without cancelling the OS operation")
    }
}
