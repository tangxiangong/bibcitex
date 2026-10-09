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
    private func translated<T>(_ operation: () throws -> T) throws -> T {
        do { return try operation() }
        catch let error as CoreError {
            switch error { case .Operation(let message): throw HelperError.ffiError(message) }
        }
    }
    private func library(_ record: LibraryRecord) -> Bibliography {
        Bibliography(name: record.name, path: record.path, updatedAt: record.updatedAt, descriptionText: record.description, pinned: record.pinned, available: record.available)
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
            citeKey: value.citeKey, source: value.source, typeKind: type, typeUnknown: value.entryType,
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
