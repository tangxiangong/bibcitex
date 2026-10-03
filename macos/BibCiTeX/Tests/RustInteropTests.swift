import Foundation
import BibCiTeXCore

/// Exercises actual generated bindings and Rust without creating windows or touching settings/clipboard.
@main
struct RustInteropTests {
    static func main() throws {
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent("bibcitex-interop-" + UUID().uuidString)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
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
        print("PASS UniFFI Swift/Rust Unicode, math chunks, optional metadata, type/field search, stable IDs, repeated ownership and typed errors")
    }
}
