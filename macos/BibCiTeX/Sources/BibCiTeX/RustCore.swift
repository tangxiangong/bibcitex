import Foundation
import BibCiTeXCore

/// Calls generated UniFFI bindings on background actor executors.
/// Separate search and paste actors keep large bibliography parsing off the helper's activation path.
actor RustCore {
    static let shared = RustCore()
    static let helper = RustCore()
    static let pasteboard = RustCore()

    func libraries() throws -> [Bibliography] { try translated { try BibCiTeXCore.libraries() }.map(library) }
    func currentLibrary() throws -> Bibliography? { try translated { try BibCiTeXCore.helperCurrent() }.map(library) }
    func selectLibrary(name: String, path: String) throws -> Bibliography {
        library(try translated { try BibCiTeXCore.helperSelect(name: name, path: path) })
    }
    func addLibrary(name: String, path: String, description: String) throws {
        _ = try translated { try BibCiTeXCore.addLibrary(name: name, path: path, description: description.isEmpty ? nil : description) }
    }
    func updateLibrary(name: String, newName: String, path: String?, description: String?) throws {
        _ = try translated { try BibCiTeXCore.updateLibrary(name: name, newName: newName, path: path, description: description) }
    }
    func setLibraryPinned(name: String, pinned: Bool) throws {
        try translated { try BibCiTeXCore.setLibraryPinned(name: name, pinned: pinned) }
    }
    func removeLibrary(name: String) throws { try translated { try BibCiTeXCore.removeLibrary(name: name) } }
    func capturePasteTarget() throws { try translated { try BibCiTeXCore.capturePasteTarget() } }
    func copy(_ text: String) throws { try translated { try BibCiTeXCore.copy(text: text) } }
    func paste(_ text: String) throws { try translated { try BibCiTeXCore.paste(text: text) } }
    func search(path: String, query: String, field: String = "all", type: String = "all") throws -> [Reference] {
        try translated { try BibCiTeXCore.search(path: path, query: query, field: field, typeFilter: type) }.map(reference)
    }
    /// Applies a native UTF-8 edit range while retaining untouched tag IDs.
    /// Edits the shared ordered tree, preserving group semantics and tag identity.
    func editQueryText(_ edit: QueryTextEdit) -> QueryAnalysis { BibCiTeXCore.editQueryText(edit: edit) }
    /// Returns the shared field/type vocabulary; clients never duplicate parser rules.
    func searchCatalog() -> SearchCatalog { BibCiTeXCore.searchCatalog() }
    /// Analyzes text without executing a bibliography search.
    func analyzeQuery(_ text: String) -> QueryAnalysis { BibCiTeXCore.analyzeQuery(text: text) }
    /// Serializes a validated typed tree without changing tag bindings.
    func renderQuery(_ document: QueryDocumentRecord) -> QueryAnalysis { BibCiTeXCore.renderQuery(document: document) }
    /// Constructs a validated typed condition, including optional stable tag binding.
    func buildCondition(_ condition: QueryCondition) -> QueryAnalysis { BibCiTeXCore.buildCondition(condition: condition) }
    func editQuery(_ edit: QueryEdit) -> QueryAnalysis { BibCiTeXCore.editQuery(edit: edit) }
    /// Refreshes bound tags by ID; deleted bindings remain errors.
    func refreshQuery(_ document: QueryDocumentRecord) throws -> QueryAnalysis { try translated { try BibCiTeXCore.refreshQuery(document: document) } }
    /// Reads one revision of the tag directory and optional reference assignments.
    func tagSnapshot(path: String, citeKey: String) throws -> TagSnapshot { try translated { try BibCiTeXCore.tagSnapshot(path: path, citeKey: citeKey) } }
    func tagSnapshot(libraryUID: String, recordUID: String) throws -> TagSnapshot {
        try translated { try BibCiTeXCore.tagSnapshotUid(libraryUid: libraryUID, recordUid: recordUID) }
    }
    /// Mutates tags with optimistic revision checking; never edits BibTeX.
    func batchTagSnapshot(_ targets: [TagTarget]) throws -> BatchTagSnapshot { try translated { try BibCiTeXCore.batchTagSnapshot(targets: targets) } }
    func mutateBatchTag(_ mutation: BatchTagMutation) throws -> BatchTagSnapshot { try translated { try BibCiTeXCore.mutateBatchTag(mutation: mutation) } }
    func mutateTag(_ mutation: TagMutation) throws -> TagSnapshot { try translated { try BibCiTeXCore.mutateTag(mutation: mutation) } }
    /// Checks file and tag revisions immediately before acting on a result.
    func validateContext(_ context: SearchContext) throws -> Bool { try translated { try BibCiTeXCore.validateSearchContext(context: context) } }
    /// Executes an explicit mode and refreshes stale bound IDs before one retry.
    func searchExplicit(path: String, query: String, field: String = "all", type: String = "all", advanced: Bool, draft: SearchDraft = SearchDraft(), version: UInt64) throws -> NativeSearchResult {
        var request = AdvancedSearchRequest(path: path, mode: advanced ? .advanced : .ordinary,
            inputKind: advanced && draft.document != nil ? .bound : .text, text: query, document: advanced ? draft.document : nil,
            expectedTagRevision: advanced ? draft.tagRevision : nil, defaultField: field, typeFilter: type, queryVersion: version)
        var refreshedText: String?
        var refreshedDraft: SearchDraft?
        var result = try translated { try BibCiTeXCore.advancedSearch(request: request) }
        if result.analysis.status == .staleQuery, let document = request.document {
            let refreshed = try refreshQuery(document)
            if refreshed.status == .valid {
                request.document = refreshed.document; request.text = refreshed.text; request.expectedTagRevision = refreshed.tagRevision
                result = try translated { try BibCiTeXCore.advancedSearch(request: request) }
                if result.analysis.status == .valid {
                    refreshedText = refreshed.text
                    refreshedDraft = SearchDraft(document: refreshed.document, tagRevision: refreshed.tagRevision)
                }
            } else {
                result.analysis = refreshed; result.references = []; result.context = nil
            }
        }
        let keys = Dictionary(uniqueKeysWithValues: result.references.map { ($0.id, $0.citeKey) })
        let evidence = Dictionary(result.evidence.compactMap { item -> (String, String)? in
            guard let key = keys[item.referenceId] else { return nil }
            let spelling = item.fuzzyTerms.isEmpty ? "" : L10n.text("拼写补充") + ": " + item.fuzzyTerms.joined(separator: ", ")
            return (key, [item.fields.joined(separator: ", "), spelling].filter { !$0.isEmpty }.joined(separator: " · "))
        }, uniquingKeysWith: { first, _ in first })
        let ranges = Dictionary(result.evidence.compactMap { item -> (String, [SearchMatchRange])? in
            guard let key = keys[item.referenceId] else { return nil }; return (key, item.ranges)
        }, uniquingKeysWith: { $0 + $1 })
        let valid = result.analysis.status == .valid
        return NativeSearchResult(references: valid ? result.references.map(reference) : [], analysis: result.analysis,
            context: valid ? result.context : nil, evidence: evidence, matchRanges: ranges, refreshedText: refreshedText, refreshedDraft: refreshedDraft)
    }
    private func translated<T>(_ operation: () throws -> T) throws -> T {
        do { return try operation() }
        catch let error as CoreError {
            switch error { case .Operation(let message): throw HelperError.ffiError(message) }
        }
    }
    private func library(_ record: LibraryRecord) -> Bibliography {
        Bibliography(uid: record.uid, name: record.name, path: record.path, updatedAt: record.updatedAt, descriptionText: record.description, pinned: record.pinned, available: record.available)
    }
    private func chunks(_ records: [ChunkRecord]) -> [TextChunk] {
        records.map {
            let kind: TextChunkKind
            switch $0.kind { case .normal: kind = .normal; case .verbatim: kind = .verbatim; case .math: kind = .math }
            return TextChunk(kind: kind, text: $0.text)
        }
    }
    private func reference(_ value: ReferenceRecord) -> Reference {
        let type = (0...32).compactMap(EntryType.init(rawValue:)).first { $0.displayText.lowercased() == value.entryType.lowercased() } ?? .unknown
        return Reference(
            id: value.id, libraryUID: value.libraryUid, tags: value.tags, citeKey: value.citeKey, source: value.source, typeKind: type, typeUnknown: value.entryType,
            author: value.author, title: chunks(value.title), journal: value.journal ?? "", year: value.year.map(Int.init),
            fullJournal: value.fullJournal ?? "", volume: value.volume, number: value.number ?? "",
            pages: value.pages.map { ReferencePages(start: $0.start, end: $0.end) },
            note: chunks(value.note), doi: value.doi ?? "", mrclass: value.mrclass ?? "", publisher: value.publisher,
            series: value.series ?? "", isbn: value.isbn ?? "", url: value.url ?? "", file: value.file ?? "",
            abstractChunks: chunks(value.abstractText), edition: value.edition, issue: chunks(value.issue), bookPages: value.bookPages ?? "",
            school: value.school ?? "", address: value.address ?? "", bookTitle: chunks(value.bookTitle), editor: value.editor.map { ($0.name, $0.role) },
            month: value.month ?? "", organization: value.organization, institution: value.institution ?? "", eprint: value.eprint ?? "",
            archivePrefix: value.archivePrefix ?? "", arxivPrimaryClass: value.arxivPrimaryClass ?? "", howPublished: value.howPublished ?? ""
        )
    }
}
