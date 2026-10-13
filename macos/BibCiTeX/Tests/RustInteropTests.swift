import Foundation
import BibCiTeXCore

/// Exercises actual generated bindings and Rust without creating windows or touching user settings/clipboard.
@main
struct RustInteropTests {
    static func main() throws {
        let attachmentCases = [
            ("/Users/me/paper:2024:final.pdf", "/Users/me/paper:2024:final.pdf"),
            ("file:///Users/me/paper:2024:final.pdf", "/Users/me/paper:2024:final.pdf"),
            ("paper:2024:final.pdf", "/library/paper:2024:final.pdf"),
            (":papers/article.pdf:PDF", "/library/papers/article.pdf"),
            ("Article:/Users/me/paper:2024:final.pdf:PDF", "/Users/me/paper:2024:final.pdf"),
            (":file:///Users/me/paper%20one.pdf:PDF", "/Users/me/paper one.pdf"),
            (#"Article:papers/a\:b\;c.pdf:PDF;:second.pdf:PDF"#, "/library/papers/a:b;c.pdf"),
            (#"papers/a\;b.pdf;second.pdf"#, "/library/papers/a;b.pdf")
        ]
        for (input, expected) in attachmentCases {
            precondition(AttachmentPath.resolve(input, relativeTo: "/library/references.bib")?.path == expected, input)
        }
        precondition(AttachmentPath.resolve("", relativeTo: "/library/references.bib") == nil)

        precondition(BibCiTeXCore.formatCitation(keys: ["A", "B"], format: "raw", mode: "individual") == "A,B")
        precondition(BibCiTeXCore.formatCitation(keys: ["A", "B"], format: "cite", mode: "merged") == #"\cite{A,B}"#)
        precondition(BibCiTeXCore.formatCitation(keys: ["A", "B"], format: "cite", mode: "individual") == #"\cite{A} \cite{B}"#)
        precondition(BibCiTeXCore.formatCitation(keys: ["A", "B"], format: "typst", mode: "individual") == "@A #box[] @B")

        let directory = FileManager.default.temporaryDirectory.appendingPathComponent("bibcitex-interop-" + UUID().uuidString)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        setenv("BIBCITEX_CONFIG_DIRECTORY", directory.path, 1)
        let file = directory.appendingPathComponent("references.bib")
        try """
        @article{unicode2025,
            author = {Zhang, San and Doe, Jane},
            title = {中文研究 $E=mc^2$},
            journal = {Science},
            year = {2025},
            pages = {12--18},
            doi = {10.1000/example}
        }
        @book{emptyfields, title = {Collected Works}, publisher = {Example Press}}
        @phdthesis{thesis2024, author = {Smith, Alice}, title = {Dissertation}, school = {Example University}, year = {2024}}
        """.write(to: file, atomically: true, encoding: .utf8)

        let records = try BibCiTeXCore.search(path: file.path, query: "", field: "all", typeFilter: "all")
        precondition(records.count == 3)
        let article = records.first { $0.citeKey == "unicode2025" }!
        precondition(article.entryType == "Article" && article.year == 2025)
        precondition(article.author.count == 2 && article.pages != nil)
        precondition(article.title.contains { $0.kind == .math && $0.text == "E=mc^2" })
        precondition(article.title.contains { $0.text.contains("中文研究") })
        precondition(article.doi == "10.1000/example" && article.source.contains("unicode2025"))
        precondition(article.abstractText.isEmpty)
        precondition(records.first { $0.citeKey == "emptyfields" }!.year == nil)
        let filtered = try BibCiTeXCore.search(path: file.path, query: "Zhang", field: "author", typeFilter: "Article")
        precondition(filtered.map(\.citeKey) == ["unicode2025"])
        let theses = try BibCiTeXCore.search(path: file.path, query: "", field: "all", typeFilter: "Thesis")
        precondition(theses.map(\.citeKey) == ["thesis2024"])
        let missingAuthors = try BibCiTeXCore.search(path: file.path, query: " ", field: "author", typeFilter: "all")
        precondition(missingAuthors.count == 3)

        let combined = try BibCiTeXCore.search(path: file.path, query: "Zhang 中文研究", field: "all", typeFilter: "Article")
        precondition(combined.map(\.citeKey) == ["unicode2025"])
        let bounded = try BibCiTeXCore.search(path: file.path, query: "Zhang 中文研究", field: "title", typeFilter: "all")
        precondition(bounded.isEmpty)
        let fuzzy = try BibCiTeXCore.search(path: file.path, query: "Dissertatoin", field: "title", typeFilter: "Thesis")
        precondition(fuzzy.map(\.citeKey) == ["thesis2024"])

        let searchFile = directory.appendingPathComponent("ordinary-search.bib")
        try """
        @misc{accent, title={Gödel machine-learning}}
        @misc{godel, title={Unrelated}}
        @misc{plain, title={Godel}}
        """.write(to: searchFile, atomically: true, encoding: .utf8)
        let ranked = try BibCiTeXCore.search(path: searchFile.path, query: "godel", field: "all", typeFilter: "all")
        precondition(ranked.map(\.citeKey) == ["godel", "plain", "accent"])
        let punctuation = try BibCiTeXCore.search(path: searchFile.path, query: "machine learning", field: "title", typeFilter: "all")
        precondition(punctuation.map(\.citeKey) == ["accent"])
        try "@misc{changed,title={New graph methods}}".write(to: searchFile, atomically: true, encoding: .utf8)
        let refreshed = try BibCiTeXCore.search(path: searchFile.path, query: "new graph", field: "title", typeFilter: "all")
        precondition(refreshed.map(\.citeKey) == ["changed"])

        let incomplete = BibCiTeXCore.analyzeQuery(text: "author:")
        precondition(incomplete.status == .incomplete && incomplete.document == nil)
        let row = records[0]
        precondition(UUID(uuidString: row.id) != nil && UUID(uuidString: row.libraryUid) != nil)
        let tagState = try BibCiTeXCore.tagSnapshotUid(libraryUid: row.libraryUid, recordUid: row.id)
        let tagged = try BibCiTeXCore.mutateTag(mutation: TagMutation(libraryUid: row.libraryUid, recordUid: row.id, color: "#3b82f6", action: .create, id: "", name: "Interop", targetId: "", path: "", citeKey: "", expectedRevision: tagState.revision))
        precondition(tagged.tags.first?.color == "#3B82F6" && tagged.assignedIds.count == 1)
        let decorated = try BibCiTeXCore.search(path: file.path, query: "", field: "all", typeFilter: "all")
        precondition(decorated.first { $0.id == row.id }?.tags.first?.id == tagged.assignedIds.first)
        let invalid = BibCiTeXCore.analyzeQuery(text: "year:abc author:")
        precondition(invalid.status == .invalid && !invalid.diagnostics.isEmpty)
        let batchRows = try BibCiTeXCore.search(path: file.path, query: "", field: "all", typeFilter: "all")
        let batchTargets = batchRows.map { TagTarget(libraryUid: $0.libraryUid, recordUid: $0.id) }
        let batchState = try BibCiTeXCore.batchTagSnapshot(targets: batchTargets)
        precondition(batchState.count == UInt64(batchRows.count) && batchState.partialIds.contains(tagged.assignedIds[0]))
        let batchAdded = try BibCiTeXCore.mutateBatchTag(mutation: BatchTagMutation(targets: batchTargets, action: .assign, id: tagged.assignedIds[0], name: "", expectedRevision: batchState.revision))
        precondition(batchAdded.assignedIds.contains(tagged.assignedIds[0]) && batchAdded.partialIds.isEmpty)
        let batchRemoved = try BibCiTeXCore.mutateBatchTag(mutation: BatchTagMutation(targets: batchTargets, action: .unassign, id: tagged.assignedIds[0], name: "", expectedRevision: batchAdded.revision))
        precondition(batchRemoved.assignedIds.isEmpty && batchRemoved.partialIds.isEmpty && batchRemoved.tags.count == batchAdded.tags.count)
        let condition = BibCiTeXCore.buildCondition(condition: QueryCondition(fieldName: "title", textValue: "Collected Works", phrase: false, exclude: false, tagId: nil))
        precondition(condition.status == .valid)
        let rendered = BibCiTeXCore.renderQuery(document: condition.document!)
        precondition(rendered.text == condition.text)
        let advanced = try BibCiTeXCore.advancedSearch(request: AdvancedSearchRequest(
            path: file.path, mode: .advanced, inputKind: .bound, text: condition.text,
            document: condition.document, expectedTagRevision: nil, defaultField: "all", typeFilter: "all", queryVersion: 42))
        precondition(advanced.references.map(\.citeKey) == ["emptyfields"])
        precondition(advanced.context?.queryVersion == 42)
        precondition(advanced.evidence.first?.ranges.isEmpty == false)
        let supplemented = try BibCiTeXCore.advancedSearch(request: AdvancedSearchRequest(
            path: file.path, mode: .advanced, inputKind: .text, text: "Dissertatoin",
            document: nil, expectedTagRevision: nil, defaultField: "title", typeFilter: "Thesis", queryVersion: 43))
        precondition(supplemented.evidence.first?.fuzzyTerms == ["Dissertatoin"])
        let contextValid = try BibCiTeXCore.validateSearchContext(context: advanced.context!)
        precondition(contextValid)
        let catalog = BibCiTeXCore.searchCatalog()
        precondition(catalog.fields.contains("tag") && catalog.types.contains("phdthesis"))
        for _ in 0..<32 {
            let result = try BibCiTeXCore.advancedSearch(request: AdvancedSearchRequest(
                path: file.path, mode: .advanced, inputKind: .text, text: "year:2024 OR type:book",
                document: nil, expectedTagRevision: nil, defaultField: "all", typeFilter: "all", queryVersion: 1))
            precondition(result.analysis.status == .valid && result.references.count == 2)
        }

        for _ in 0..<100 {
            let roundTrip = try BibCiTeXCore.search(path: file.path, query: "", field: "all", typeFilter: "all")
            precondition(roundTrip.count == 3 && roundTrip.first?.id == records.first?.id)
        }
        do {
            _ = try BibCiTeXCore.search(path: directory.appendingPathComponent("missing.bib").path, query: "", field: "all", typeFilter: "all")
            preconditionFailure("Expected a typed Rust error")
        } catch let CoreError.Operation(message) {
            precondition(!message.isEmpty)
        }
        if ProcessInfo.processInfo.environment["BIBCITEX_SEARCH_BENCHMARK"] == "1" {
            for count in [1_000, 10_000, 50_000] {
                let input = (0..<count).map { "@article{item\($0),title={Graph neural network},author={Smith, Jane},year={2026}}" }.joined(separator: "\n")
                let fixture = directory.appendingPathComponent("payload-\(count).bib")
                try input.write(to: fixture, atomically: true, encoding: .utf8)
                _ = try BibCiTeXCore.search(path: fixture.path, query: "", field: "all", typeFilter: "all")
                var samples: [Double] = []
                for _ in 0..<10 {
                    let start = DispatchTime.now().uptimeNanoseconds
                    let payload = try BibCiTeXCore.search(path: fixture.path, query: "", field: "all", typeFilter: "all")
                    precondition(payload.count == count)
                    samples.append(Double(DispatchTime.now().uptimeNanoseconds - start) / 1_000_000)
                }
                samples.sort()
                print("BINDING,swift,\(count),\(samples[5]),\(samples[9])")
            }
        }
        print("PASS UniFFI Swift/Rust Unicode, math chunks, optional metadata, type/field search, stable IDs, repeated ownership and typed errors")
    }
}
