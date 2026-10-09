import ApplicationServices
import Foundation

/// One source of truth for the helper surface metrics: the view draws these
/// heights and `HelperViewModel` sizes the panel from them.
enum HelperMetrics {
    /// A 44pt search row centred in 10pt of slack above and below.
    static let header: CGFloat = 64
    static let panelRadius: CGFloat = 26
    static let rowRadius: CGFloat = 10
    /// Title (two lines), authors and the metadata line, plus the row padding.
    static let referenceRow: CGFloat = 80
    static let libraryRow: CGFloat = 56
    static let listPadding: CGFloat = 16
    static let emptyRow: CGFloat = 112
    static let errorBar: CGFloat = 40
    static let bottomBar: CGFloat = 52
    static let barButton: CGFloat = 28
    static let keyCap: CGFloat = 18
}

enum TextChunkKind: Int32, Sendable {
    case normal = 0
    case verbatim = 1
    case math = 2
}

enum ThemeMode: Int32, Sendable {
    case light = 0
    case dark = 1

    var isDark: Bool {
        self == .dark
    }
}

/// The accessibility permission that cross-app paste depends on. `xpaste` still
/// enforces it at the point of injection; this only decides when the system is
/// allowed to show its permission prompt. The paste model independently handles
/// denied access with automatic copying and a paste-failure alert.
@MainActor
enum AccessibilityPermission {
    private static var prompted = false

    static var isTrusted: Bool { AXIsProcessTrusted() }

    /// Returns whether a paste may proceed. On the first attempt without the
    /// permission it opens the system prompt instead, and reports failure so the
    /// caller skips key injection. Later attempts do not repeat the OS prompt;
    /// the caller still handles denial and reports the paste failure.
    static func ensureTrusted() -> Bool {
        if isTrusted { return true }
        guard !prompted else { return false }
        prompted = true
        let promptKey = kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String
        _ = AXIsProcessTrustedWithOptions([promptKey: true] as CFDictionary)
        return false
    }
}

enum EntryType: Int32, Sendable {
    case article = 0
    case book = 1
    case booklet = 2
    case inBook = 3
    case inCollection = 4
    case inProceedings = 5
    case manual = 6
    case mastersThesis = 7
    case phdThesis = 8
    case misc = 9
    case proceedings = 10
    case techReport = 11
    case thesis = 27
    case unpublished = 12
    case mvBook = 13
    case bookInBook = 14
    case suppBook = 15
    case periodical = 16
    case suppPeriodical = 17
    case collection = 18
    case mvCollection = 19
    case suppCollection = 20
    case reference = 21
    case mvReference = 22
    case inReference = 23
    case mvProceedings = 24
    case report = 25
    case patent = 26
    case software = 29
    case dataset = 30
    case set = 31
    case xData = 32
    case unknown = 255

    var displayText: String {
        switch self {
        case .article:
            "Article"
        case .book:
            "Book"
        case .booklet:
            "Booklet"
        case .inBook:
            "InBook"
        case .inCollection:
            "InCollection"
        case .inProceedings:
            "InProceedings"
        case .manual:
            "Manual"
        case .mastersThesis:
            "MastersThesis"
        case .phdThesis:
            "PhdThesis"
        case .misc:
            "Misc"
        case .proceedings:
            "Proceedings"
        case .techReport:
            "TechReport"
        case .thesis:
            "Thesis"
        case .unpublished:
            "Unpublished"
        case .mvBook:
            "MvBook"
        case .bookInBook:
            "BookInBook"
        case .suppBook:
            "SuppBook"
        case .periodical:
            "Periodical"
        case .suppPeriodical:
            "SuppPeriodical"
        case .collection:
            "Collection"
        case .mvCollection:
            "MvCollection"
        case .suppCollection:
            "SuppCollection"
        case .reference:
            "Reference"
        case .mvReference:
            "MvReference"
        case .inReference:
            "InReference"
        case .mvProceedings:
            "MvProceedings"
        case .report:
            "Report"
        case .patent:
            "Patent"
        case .software:
            "Software"
        case .dataset:
            "Dataset"
        case .set:
            "Set"
        case .xData:
            "XData"
        case .unknown:
            "Unknown"
        }
    }
}

struct TextChunk: Identifiable, Sendable {
    let id = UUID()
    let kind: TextChunkKind
    let text: String
}

struct ReferencePages: Sendable {
    let start: UInt32
    let end: UInt32
}

struct Reference: Identifiable, Sendable {
    var id: String { citeKey }
    let citeKey: String
    let source: String
    let typeKind: EntryType
    let typeUnknown: String?
    let author: [String]
    let title: [TextChunk]
    let journal: String
    let year: Int?
    let fullJournal: String
    let volume: Int64?
    let number: String
    let pages: ReferencePages?
    let note: [TextChunk]
    let doi: String
    let mrclass: String
    let publisher: [String]
    let series: String
    let isbn: String
    let url: String
    let file: String
    let abstractChunks: [TextChunk]
    let edition: Int64?
    let issue: [TextChunk]
    let bookPages: String
    let school: String
    let address: String
    let bookTitle: [TextChunk]
    let editor: [(String, String)]
    let month: String
    let organization: [String]
    let institution: String
    let eprint: String
    let archivePrefix: String
    let arxivPrimaryClass: String
    let howPublished: String

    var displayType: String {
        if typeKind == .unknown, let unknownType = typeUnknown, !unknownType.isEmpty {
            unknownType
        } else {
            typeKind.displayText
        }
    }

    var titleText: String {
        title.map(\.text).joined()
    }

    var venueText: String {
        if !fullJournal.isEmpty {
            return fullJournal
        }
        if !journal.isEmpty {
            return journal
        }
        if !bookTitle.isEmpty {
            return bookTitle.map(\.text).joined()
        }
        if !publisher.isEmpty {
            return publisher.joined(separator: ", ")
        }
        if !school.isEmpty {
            return school
        }
        if !institution.isEmpty {
            return institution
        }
        if !organization.isEmpty {
            return organization.joined(separator: ", ")
        }
        return howPublished
    }

    var pagesText: String {
        guard let pages else { return "" }
        return "\(pages.start)-\(pages.end)"
    }
}

struct Bibliography: Identifiable, Equatable, Sendable {
    var id: [String] { [name, path] }
    let name: String
    let path: String
    let updatedAt: String
    let descriptionText: String?
    var pinned: Bool = false
    var available: Bool = true
}
