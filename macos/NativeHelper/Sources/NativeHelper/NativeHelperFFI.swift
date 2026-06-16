import Foundation

struct FfiString {
    let ptr: UnsafeMutablePointer<CChar>?
    let len: Int
}

struct FfiStringArray {
    let hasValue: UInt8
    let ptr: UnsafeMutablePointer<FfiString>?
    let len: Int
}

struct FfiEditor {
    let name: FfiString
    let role: FfiString
}

struct FfiEditorArray {
    let hasValue: UInt8
    let ptr: UnsafeMutablePointer<FfiEditor>?
    let len: Int
}

struct FfiRange {
    let hasValue: UInt8
    let start: UInt32
    let end: UInt32
}

struct FfiChunk {
    let kind: Int32
    let text: FfiString
}

struct FfiChunkArray {
    let hasValue: UInt8
    let ptr: UnsafeMutablePointer<FfiChunk>?
    let len: Int
}

struct FfiEntryType {
    let kind: Int32
    let unknown: FfiString
}

struct FfiReference {
    let citeKey: FfiString
    let source: FfiString
    let entryType: FfiEntryType
    let author: FfiStringArray
    let title: FfiChunkArray
    let journal: FfiString
    let year: Int32
    let hasYear: UInt8
    let fullJournal: FfiString
    let volume: Int64
    let hasVolume: UInt8
    let number: FfiString
    let pages: FfiRange
    let note: FfiChunkArray
    let doi: FfiString
    let mrclass: FfiString
    let publisher: FfiStringArray
    let series: FfiString
    let isbn: FfiString
    let url: FfiString
    let file: FfiString
    let abstract: FfiChunkArray
    let edition: Int64
    let hasEdition: UInt8
    let issue: FfiChunkArray
    let bookPages: FfiString
    let school: FfiString
    let address: FfiString
    let bookTitle: FfiChunkArray
    let editor: FfiEditorArray
    let month: FfiString
    let organization: FfiStringArray
    let institution: FfiString
    let eprint: FfiString
    let archivePrefix: FfiString
    let arxivPrimaryClass: FfiString
    let howPublished: FfiString
}

struct FfiReferenceArray {
    let hasValue: UInt8
    let ptr: UnsafeMutablePointer<FfiReference>?
    let len: Int
}

struct FfiBibliography {
    let name: FfiString
    let path: FfiString
    let updatedAt: FfiString
    let description: FfiString
    let hasDescription: UInt8
}

struct FfiBibliographyArray {
    let hasValue: UInt8
    let ptr: UnsafeMutablePointer<FfiBibliography>?
    let len: Int
}

func boolValue(_ value: UInt8) -> Bool {
    value != 0
}

func stringFromFfi(_ source: FfiString) -> String {
    guard source.len > 0, let ptr = source.ptr else {
        return ""
    }
    return String(decoding: UnsafeBufferPointer(start: ptr, count: source.len), as: UTF8.self)
}

func stringArrayFromFfi(_ source: FfiStringArray) -> [String] {
    if !boolValue(source.hasValue) || source.ptr == nil || source.len == 0 {
        return []
    }

    return Array(UnsafeBufferPointer(start: source.ptr, count: source.len)).map { stringFromFfi($0) }
}

func chunkTextArrayFromFfi(_ source: FfiChunkArray) -> [NativeHelperChunk] {
    if !boolValue(source.hasValue) || source.ptr == nil || source.len == 0 {
        return []
    }

    return Array(UnsafeBufferPointer(start: source.ptr, count: source.len)).compactMap {
        let kind = NativeHelperFFIKind(rawValue: $0.kind)
        guard let resolvedKind = kind else {
            return nil
        }
        return NativeHelperChunk(kind: resolvedKind, text: stringFromFfi($0.text))
    }
}

func editorTupleArrayFromFfi(_ source: FfiEditorArray) -> [(String, String)] {
    if !boolValue(source.hasValue) || source.ptr == nil || source.len == 0 {
        return []
    }

    return Array(UnsafeBufferPointer(start: source.ptr, count: source.len)).map {
        (stringFromFfi($0.name), stringFromFfi($0.role))
    }
}

@_silgen_name("bibcitex_native_helper_init_list")
func bibcitex_native_helper_init_list() -> FfiBibliographyArray

@_silgen_name("bibcitex_native_helper_set_current_bibliography")
func bibcitex_native_helper_set_current_bibliography(_ name: UnsafePointer<CChar>?, _ path: UnsafePointer<CChar>?) -> Int32

@_silgen_name("bibcitex_native_helper_current_bibliography")
func bibcitex_native_helper_current_bibliography() -> FfiBibliography

@_silgen_name("bibcitex_native_helper_search_references")
func bibcitex_native_helper_search_references(_ query: UnsafePointer<CChar>?) -> FfiReferenceArray

@_silgen_name("bibcitex_native_helper_copy_and_paste")
func bibcitex_native_helper_copy_and_paste(_ citeKey: UnsafePointer<CChar>?) -> Int32

@_silgen_name("bibcitex_native_helper_last_error")
func bibcitex_native_helper_last_error() -> FfiString

@_silgen_name("bibcitex_free_bibliography_array")
func bibcitex_free_bibliography_array(_ value: FfiBibliographyArray)

@_silgen_name("bibcitex_free_bibliography")
func bibcitex_free_bibliography(_ value: FfiBibliography)

@_silgen_name("bibcitex_free_reference_array")
func bibcitex_free_reference_array(_ value: FfiReferenceArray)

@_silgen_name("bibcitex_free_reference")
func bibcitex_free_reference(_ value: FfiReference)

@_silgen_name("bibcitex_free_string")
func bibcitex_free_string(_ value: FfiString)

@_silgen_name("bibcitex_free_string_array")
func bibcitex_free_string_array(_ value: FfiStringArray)

@_silgen_name("bibcitex_free_editor_array")
func bibcitex_free_editor_array(_ value: FfiEditorArray)

@_silgen_name("bibcitex_free_chunk_array")
func bibcitex_free_chunk_array(_ value: FfiChunkArray)
