import Foundation

enum NativeHelperFFIKind: Int32 {
    case normal = 0
    case verbatim = 1
    case math = 2
}

enum NativeHelperThemeMode: Int32 {
    case light = 0
    case dark = 1

    var isDark: Bool {
        self == .dark
    }
}

enum NativeEntryTypeKind: Int32 {
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

struct NativeHelperChunk: Identifiable {
    let id = UUID()
    let kind: NativeHelperFFIKind
    let text: String
}

struct NativeHelperRange {
    let start: UInt32
    let end: UInt32
}

struct NativeHelperReference: Identifiable {
    let id = UUID()
    let citeKey: String
    let source: String
    let typeKind: NativeEntryTypeKind
    let typeUnknown: String?
    let author: [String]
    let title: [NativeHelperChunk]
    let journal: String
    let year: Int?
    let fullJournal: String
    let volume: Int64?
    let number: String
    let pages: NativeHelperRange?
    let note: [NativeHelperChunk]
    let doi: String
    let mrclass: String
    let publisher: [String]
    let series: String
    let isbn: String
    let url: String
    let file: String
    let abstractChunks: [NativeHelperChunk]
    let edition: Int64?
    let issue: [NativeHelperChunk]
    let bookPages: String
    let school: String
    let address: String
    let bookTitle: [NativeHelperChunk]
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

struct NativeHelperBibliography: Identifiable, Equatable {
    let id = UUID()
    let name: String
    let path: String
    let updatedAt: String
    let descriptionText: String?
}
