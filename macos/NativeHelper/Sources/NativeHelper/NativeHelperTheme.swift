import Foundation

enum NativeHelperThemeStyle {
    case latte
    case mocha

    init(mode: Int32) {
        self = mode == NativeHelperThemeMode.dark.rawValue ? .mocha : .latte
    }

    var isDark: Bool {
        self == .mocha
    }

    var panelMaterial: String {
        isDark ? "ultraThickMaterial" : "thickMaterial"
    }
}

enum NativeHelperError: LocalizedError {
    case ffiError(String)
    case emptyCiteKey

    var errorDescription: String? {
        switch self {
        case .ffiError(let message):
            return message
        case .emptyCiteKey:
            return "引用键为空"
        }
    }
}

final class NativeHelperService {
    func loadBibliographies() throws -> [NativeHelperBibliography] {
        var array = FfiBibliographyArray.empty
        guard bibcitex_native_helper_init_list(&array) != 0 else {
            throw NativeHelperError.ffiError("加载文献库失败")
        }
        defer {
            withUnsafePointer(to: array) {
                bibcitex_free_bibliography_array($0)
            }
        }

        guard boolValue(array.hasValue) == true, array.len > 0, let ptr = array.ptr else {
            return []
        }

        let source = UnsafeBufferPointer(start: ptr, count: array.len)
        return source.map {
            let name = stringFromFfi($0.name)
            let path = stringFromFfi($0.path)
            let updatedAt = stringFromFfi($0.updatedAt)
            let description = boolValue($0.hasDescription) ? stringFromFfi($0.description) : nil
            return NativeHelperBibliography(
                name: name,
                path: path,
                updatedAt: updatedAt,
                descriptionText: description,
            )
        }
    }

    func currentBibliography() throws -> NativeHelperBibliography? {
        var bibliography = FfiBibliography.empty
        guard bibcitex_native_helper_current_bibliography(&bibliography) != 0 else {
            throw NativeHelperError.ffiError(fetchLastError() ?? "读取当前文献库失败")
        }

        let hasName = bibliography.name.len > 0
        let hasPath = bibliography.path.len > 0
        if !hasName || !hasPath {
            withUnsafePointer(to: bibliography) {
                bibcitex_free_bibliography($0)
            }
            return nil
        }

        defer {
            withUnsafePointer(to: bibliography) {
                bibcitex_free_bibliography($0)
            }
        }

        let description = boolValue(bibliography.hasDescription)
            ? stringFromFfi(bibliography.description)
            : nil

        return NativeHelperBibliography(
            name: stringFromFfi(bibliography.name),
            path: stringFromFfi(bibliography.path),
            updatedAt: stringFromFfi(bibliography.updatedAt),
            descriptionText: description,
        )
    }

    func setCurrentBibliography(name: String, path: String) throws -> NativeHelperBibliography {
        let changed = name.withCString { namePointer in
            path.withCString { pathPointer in
                bibcitex_native_helper_set_current_bibliography(namePointer, pathPointer)
            }
        }

        if changed == 0 {
            throw NativeHelperError.ffiError(fetchLastError() ?? "设置文献库失败")
        }

        guard let bibliography = try currentBibliography() else {
            throw NativeHelperError.ffiError("设置文献库后返回空")
        }
        return bibliography
    }

    func searchReferences(query: String) throws -> [NativeHelperReference] {
        var references = FfiReferenceArray.empty
        let ok = query.withCString { queryPointer in
            bibcitex_native_helper_search_references(queryPointer, &references)
        }
        guard ok != 0 else {
            throw NativeHelperError.ffiError(fetchLastError() ?? "搜索失败")
        }

        defer {
            withUnsafePointer(to: references) {
                bibcitex_free_reference_array($0)
            }
        }

        guard boolValue(references.hasValue) == true, references.len > 0, let ptr = references.ptr else {
            return []
        }

        return Array(UnsafeBufferPointer(start: ptr, count: references.len)).map { value in
            buildReference(from: value)
        }
    }

    func copyAndPaste(citeKey: String) throws {
        let result = citeKey.withCString { citeKeyPointer in
            bibcitex_native_helper_copy_and_paste(citeKeyPointer)
        }

        if result == 0 {
            throw NativeHelperError.ffiError(fetchLastError() ?? "粘贴失败")
        }
    }

    private func buildReference(from value: FfiReference) -> NativeHelperReference {
        let entryKind = NativeEntryTypeKind(rawValue: value.entryType.kind) ?? .unknown

        let typeUnknown = value.entryType.unknown.len > 0 ? stringFromFfi(value.entryType.unknown) : nil
        let hasYear = boolValue(value.hasYear)
        let year = hasYear ? Int(value.year) : nil
        let hasVolume = boolValue(value.hasVolume)
        let hasEdition = boolValue(value.hasEdition)
        let pages = boolValue(value.pages.hasValue) == true
            ? NativeHelperRange(start: value.pages.start, end: value.pages.end)
            : nil

        return NativeHelperReference(
            citeKey: stringFromFfi(value.citeKey),
            source: stringFromFfi(value.source),
            typeKind: entryKind,
            typeUnknown: typeUnknown,
            author: stringArrayFromFfi(value.author),
            title: chunkTextArrayFromFfi(value.title),
            journal: stringFromFfi(value.journal),
            year: year,
            fullJournal: stringFromFfi(value.fullJournal),
            volume: hasVolume ? value.volume : nil,
            number: stringFromFfi(value.number),
            pages: pages,
            note: chunkTextArrayFromFfi(value.note),
            doi: stringFromFfi(value.doi),
            mrclass: stringFromFfi(value.mrclass),
            publisher: stringArrayFromFfi(value.publisher),
            series: stringFromFfi(value.series),
            isbn: stringFromFfi(value.isbn),
            url: stringFromFfi(value.url),
            file: stringFromFfi(value.file),
            abstractChunks: chunkTextArrayFromFfi(value.abstract),
            edition: hasEdition ? value.edition : nil,
            issue: chunkTextArrayFromFfi(value.issue),
            bookPages: stringFromFfi(value.bookPages),
            school: stringFromFfi(value.school),
            address: stringFromFfi(value.address),
            bookTitle: chunkTextArrayFromFfi(value.bookTitle),
            editor: editorTupleArrayFromFfi(value.editor),
            month: stringFromFfi(value.month),
            organization: stringArrayFromFfi(value.organization),
            institution: stringFromFfi(value.institution),
            eprint: stringFromFfi(value.eprint),
            archivePrefix: stringFromFfi(value.archivePrefix),
            arxivPrimaryClass: stringFromFfi(value.arxivPrimaryClass),
            howPublished: stringFromFfi(value.howPublished),
        )
    }

    private func fetchLastError() -> String? {
        var error = FfiString.empty
        guard bibcitex_native_helper_last_error(&error) != 0 else {
            return nil
        }
        defer {
            withUnsafePointer(to: error) {
                bibcitex_free_string($0)
            }
        }
        let message = stringFromFfi(error)
        return message.isEmpty ? nil : message
    }
}

extension NativeHelperService {
    func loadBibliographyStateIfAvailable() throws -> (bibs: [NativeHelperBibliography], current: NativeHelperBibliography?) {
        let bibs = try loadBibliographies()
        let current = try? currentBibliography()
        return (bibs, current)
    }
}
