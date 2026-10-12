import AppKit
import Combine
import SwiftUI
import BibCiTeXCore

protocol HelperServing: Sendable {
    func loadBibliographyStateIfAvailable() async throws -> (bibs: [Bibliography], current: Bibliography?)
    func setCurrentBibliography(name: String, path: String) async throws -> Bibliography
    func searchReferences(query: String) async throws -> [Reference]
    func copyAndPaste(citeKey: String) async throws
    func copy(citeKey: String) async throws
}

@MainActor
final class HelperViewModel: ObservableObject {
    var onPasteStart: (() -> Void)?
    var onPasteFailure: (() -> Void)?
    @Published private(set) var matchRanges: [String: [SearchMatchRange]] = [:]
    @Published private(set) var matchEvidence: [String: String] = [:]
    @Published private(set) var queryAnalysis: QueryAnalysis?
    @Published var candidateKind: String? { didSet { candidateOperation += 1 } }
    @Published var candidateFilter = "" { didSet { candidateOperation += 1 } }
    @Published var candidateSelection = 0 { didSet { candidateOperation += 1 } }
    @Published var candidateField: String? { didSet { candidateOperation += 1 } }
    @Published var candidateValue = "" { didSet { candidateOperation += 1 } }
    @Published var candidatePhrase = false { didSet { candidateOperation += 1 } }
    @Published var candidateExclude = false { didSet { candidateOperation += 1 } }
    @Published var candidateFocusRequest = 0
    @Published private(set) var candidateFilterFocus = false
    @Published var showTagManager = false
    @Published private(set) var candidateCatalog: SearchCatalog?
    @Published private(set) var candidateTags: TagSnapshot?
    private var candidateOperation = 0
    private var candidateParentKind: String?
    private var candidateOriginal = ""
    private var candidateTrigger = ""
    /// Confirmed semantic operands are displayed as capsules; only the tail is editable text.
    struct Capsule: Identifiable {
        let id: UUID
        var text: String
        var document: QueryDocumentRecord
        var condition: QueryCondition?
        var scopeField: String? = nil
    }
    @Published private(set) var capsules: [Capsule] = []
    @Published private(set) var tailText = ""
    private var editingCapsuleID: UUID?
    private var capsuleRevision: UInt64?
    private var activeKeywordField: String?
    private var optionalScopeBefore: (String?, [(Int, Capsule)])?
    var editingOptionalFieldValue: Bool { optionalScopeBefore != nil && candidateField != nil }
    private var savedKeywordField: String?
    private var candidateTail = ""
    private var librarySwitchCapsules: ([Capsule], String, UInt64?)?
    var inputText: String {
        if candidateKind != nil { return candidateTail }
        return isSelectingBibliography || capsules.isEmpty ? query : tailText
    }

    /// Edits the single tail operand; confirmed capsules never become duplicate filters.
    func editInput(_ text: String, start: UInt32, end: UInt32, replacement: String, full: Bool) {
        if capsules.isEmpty { editText(text, start: start, end: end, replacement: replacement, full: full); return }
        if candidateKind != nil { candidateKind = nil; candidateField = nil; editingCapsuleID = nil }
        tailText = text
        rebuildCapsuleQuery()
    }
    /// Parses the unscoped keyword operand; the UI field is applied once at execution.
    private func keywordOperand(_ text: String) -> QueryAnalysis { BibCiTeXCore.analyzeQuery(text: text) }
    private func scopedProjection() -> QueryAnalysis {
        let base = capsuleProjection()
        guard base.status == .valid, let field = activeKeywordField, var document = base.document,
              let root = document.root else { return base }
        document.nodes.append(QueryNodeRecord(kind: .scope, textValue: field, tagId: nil, children: [root], start: 0, end: 0))
        document.root = UInt32(document.nodes.count - 1)
        return BibCiTeXCore.renderQuery(document: document)
    }
    private func preserveKeywordsBeforeSelection(_ text: String) -> Bool {
        guard !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return true }
        let words = keywordOperand(text)
        guard words.status == .valid, let document = words.document else { queryAnalysis = words; return false }
        capsules.append(Capsule(id: UUID(), text: words.text, document: document, condition: nil))
        return true
    }
    private func capsuleProjection() -> QueryAnalysis {
        var document = QueryDocumentRecord(nodes: [], root: nil)
        for capsule in capsules where capsule.document.root != nil {
            let joined = BibCiTeXCore.editQuery(edit: QueryEdit(document: document, action: .appendAnd, target: nil, operand: capsule.document))
            guard joined.status == .valid, let next = joined.document else { return joined }
            document = next
        }
        let tail = keywordOperand(tailText)
        if tail.status == .valid, let operand = tail.document {
            return operand.root == nil ? BibCiTeXCore.renderQuery(document: document) : BibCiTeXCore.editQuery(edit: QueryEdit(document: document, action: .appendAnd, target: nil, operand: operand))
        }
        // Ask the shared serializer to group the existing expression, then replace
        // its final placeholder using an exact edit range. Invalid tail input is
        // retained for editing together with untouched ById anchors, never executed.
        let marker = "helperTail" + UUID().uuidString.replacingOccurrences(of: "-", with: "")
        let placeholder = BibCiTeXCore.buildCondition(condition: QueryCondition(fieldName: "", textValue: marker, phrase: false, exclude: false, tagId: nil))
        let skeleton = BibCiTeXCore.editQuery(edit: QueryEdit(document: document, action: .appendAnd, target: nil, operand: placeholder.document))
        guard skeleton.status == .valid, let bound = skeleton.document,
              let leaf = bound.nodes.last(where: { $0.kind == .word && $0.textValue == marker }) else { return skeleton }
        return BibCiTeXCore.editQueryText(edit: QueryTextEdit(document: bound, text: skeleton.text, start: leaf.start, end: leaf.end, replacement: tailText, fullReplace: false))
    }
    private func rebuildCapsuleQuery() {
        let result = scopedProjection()
        guard result.status == .valid else {
            invalidateSearch(); searchContext = nil; searchResults = []; selectedReferenceIndex = nil
            query = result.text; boundDraft = SearchDraft(document: result.document, tagRevision: capsuleRevision)
            queryAnalysis = result; recalcHeight(); return
        }
        applyAdvancedQuery(result.text, draft: SearchDraft(document: result.document, tagRevision: capsuleRevision))
    }
    /// Removes exactly one confirmed semantic operand, then recomposes the query in source order.
    func removeCapsule(_ id: UUID, refocusTail: Bool = true) {
        if candidateKind != nil { cancelCandidates() }
        if capsules.first(where: { $0.id == id })?.scopeField != nil { activeKeywordField = nil }
        capsules.removeAll { $0.id == id }
        if capsules.isEmpty { boundDraft = SearchDraft(); updateQuery(tailText) }
        else { rebuildCapsuleQuery() }
        if refocusTail { focusRequest += 1 }
    }
    /// Native Backspace at tail offset zero removes the adjacent operand and keeps
    /// the caret at zero so repeated deletes walk backward through the capsules.
    func deletePreviousCapsule() -> Bool {
        guard !isSelectingBibliography, candidateKind == nil,
              let previous = capsules.last else { return false }
        removeCapsule(previous.id, refocusTail: false)
        return true
    }
    /// Reopens a simple condition inline; grouped or handwritten operands remain text-editable.
    func editCapsule(_ id: UUID) {
        guard let capsule = capsules.first(where: { $0.id == id }) else { return }
        if capsule.scopeField != nil { beginCandidates("$"); editingCapsuleID = id; return }
        guard let condition = capsule.condition else { expandCapsulesToText(); return }
        beginCandidates(condition.fieldName == "tag" ? "#" : condition.fieldName == "type" ? "@" : "$")
        editingCapsuleID = id
        candidatePhrase = condition.phrase; candidateExclude = condition.exclude
        if condition.fieldName == "tag" || condition.fieldName == "type" {
            candidateFilter = condition.textValue
        } else { cancelCandidates(); expandCapsulesToText(); return }
        requestCandidateFocus()
    }
    /// Expands the exact shared projection for arbitrary grouped-expression editing.
    func expandCapsulesToText() {
        if candidateKind != nil { cancelCandidates() }
        let projection = scopedProjection()
        guard projection.status == .valid || !projection.text.isEmpty || projection.document != nil else {
            queryAnalysis = projection; recalcHeight(); return
        }
        capsules = []; tailText = ""; editingCapsuleID = nil; activeKeywordField = nil
        boundDraft = SearchDraft(document: projection.document, tagRevision: capsuleRevision)
        if projection.status == .valid { updateQuery(projection.text) }
        else { invalidateSearch(); searchContext = nil; searchResults = []; selectedReferenceIndex = nil; query = projection.text; queryAnalysis = projection; recalcHeight() }
        focusRequest += 1
    }
    private var librarySwitchDraft: (String, SearchDraft)?
    private(set) var boundDraft = SearchDraft()
    private var searchContext: SearchContext?
    var editingCondition: Bool { candidateKind != nil }
    struct Candidate: Identifiable { let id: String; let name: String; var searchTerms: [String] = [] }
    /// Chinese UI also accepts the English display label; English never indexes hidden Chinese keys.
    private func catalogCandidate(name: String, labelKey: String) -> Candidate {
        let display = L10n.text(labelKey)
        var terms = [name, display]
        if L10n.language == "zh-Hans" { terms.append(L10n.translate(labelKey, language: "en")) }
        return Candidate(id: name, name: display, searchTerms: terms)
    }
    var candidates: [Candidate] {
        let values: [Candidate]
        switch candidateKind {
        case "@": values = (candidateCatalog?.typeDescriptors ?? []).map { catalogCandidate(name: $0.name, labelKey: $0.labelKey) }
        case "$": values = (candidateCatalog?.fieldDescriptors ?? []).map { catalogCandidate(name: $0.name, labelKey: $0.labelKey) }
        case "#": values = (candidateTags?.tags ?? []).map { Candidate(id: $0.id, name: $0.name) }
        default: values = []
        }
        return values.filter { item in
            candidateFilter.isEmpty || (item.searchTerms.isEmpty ? [item.name] : item.searchTerms).contains { $0.localizedCaseInsensitiveContains(candidateFilter) }
        }
    }
    /// Candidate geometry follows its labels and caps the scroll viewport at six rows.
    var candidatePopupSize: CGSize {
        let width = candidates.map { ($0.name as NSString).size(withAttributes: [.font: NSFont.systemFont(ofSize: 13)]).width + 28 }.max() ?? 140
        return CGSize(width: min(220, max(140, width)), height: CGFloat(max(1, min(6, candidates.count))) * 28 + 12)
    }
    /// Matching capabilities and labels come from the shared field catalog.
    var candidateFieldDescriptor: SearchFieldDescriptor? { candidateCatalog?.fieldDescriptors.first { $0.name == candidateField } }
    var candidateSupportsPhrase: Bool { candidateFieldDescriptor?.supportsPhrase == true }
    var candidateFieldLabel: String { candidateFieldDescriptor.map { L10n.text($0.labelKey) } ?? candidateField ?? "" }
    /// Opens a physical-key candidate session without executing its provisional trigger.
    func beginCandidates(_ symbol: String, nativeBefore: String? = nil, nativeAfter: String? = nil) {
        guard !isSelectingBibliography else { return }
        if editingOptionalFieldValue { skipOptionalFieldValue() }
        candidateParentKind = nil; candidateFilterFocus = false
        candidateOriginal = capsules.isEmpty ? (nativeBefore ?? query) : query
        candidateTrigger = symbol; candidateTail = nativeBefore ?? inputText
        invalidateQueryEdit()
        editingCapsuleID = nil
        invalidateSearch(); searchContext = nil; searchResults = []; selectedReferenceIndex = nil
        candidateKind = symbol; candidateField = nil; candidateValue = ""; candidateFilter = ""; candidateSelection = 0
        candidatePhrase = false; candidateExclude = false
        // Native insertion already placed the caret correctly. The trigger itself
        // is replaced by the dedicated inline filter, not appended to query syntax.
        if capsules.isEmpty { query = candidateTail } else { tailText = candidateTail }
        requestCandidateFocus()
        recalcHeight()
        let version = searchVersion, session = sessionGeneration
        Task {
            let catalog = await RustCore.shared.searchCatalog()
            guard version == searchVersion, session == sessionGeneration, candidateKind != nil else { return }
            candidateCatalog = catalog
            do {
                let tags = try await RustCore.shared.tagSnapshot(path: "", citeKey: "")
                guard version == searchVersion, session == sessionGeneration, candidateKind != nil else { return }
                candidateTags = tags
            } catch {
                guard version == searchVersion, session == sessionGeneration, candidateKind != nil else { return }
                errorDetails = LocalizedMessage(error: error)
            }
            recalcHeight()
        }
    }
    /// Esc unwinds a field editor or nested catalog before closing the candidate session.
    func cancelCandidateLevel() {
        if editingOptionalFieldValue { skipOptionalFieldValue(); return }
        if candidateField != nil {
            candidateField = nil; candidateValue = ""; queryAnalysis = nil; requestCandidateFocus()
        } else if let parent = candidateParentKind {
            candidateParentKind = nil; candidateKind = parent; candidateFilter = ""; candidateSelection = 0
            requestCandidateFocus()
        } else { cancelCandidates() }
    }
    /// Cancels the candidate session and restores its exact pre-trigger draft.
    func cancelCandidates() {
        if editingOptionalFieldValue { skipOptionalFieldValue(); return }
        guard candidateKind != nil else { return }
        candidateKind = nil; candidateField = nil; editingCapsuleID = nil
        tailText = candidateTail
        updateQuery(candidateOriginal); focusRequest += 1
    }
    /// Transfers focus only after an explicit Tab or a field/catalog selection.
    func requestCandidateFocus() {
        candidateFilterFocus = candidateField == nil
        candidateFocusRequest += 1
    }
    func moveCandidate(_ delta: Int) {
        candidateSelection = max(0, min(candidateSelection + delta, candidates.count - 1))
    }
    func confirmCandidate() {
        if editingOptionalFieldValue { commitOptionalFieldValue(); return }
        guard candidates.indices.contains(candidateSelection) else { return }
        let candidate = candidates[candidateSelection]
        if candidateKind == "$" {
            var keywords = candidateTail
            // A structured text draft may already carry stable tag identities.
            // Preserve that unscoped document instead of reparsing its display names.
            if capsules.isEmpty, let document = boundDraft.document,
               document.nodes.contains(where: { $0.tagId != nil }) {
                let syntax = BibCiTeXCore.analyzeQuery(text: candidateOriginal)
                guard syntax.status == .valid else { queryAnalysis = syntax; recalcHeight(); return }
                let preserved = BibCiTeXCore.renderQuery(document: document)
                guard preserved.status == .valid, let base = preserved.document else { queryAnalysis = preserved; recalcHeight(); return }
                capsules.append(Capsule(id: UUID(), text: preserved.text, document: base, condition: nil))
                capsuleRevision = boundDraft.tagRevision
                keywords = ""
            }
            optionalScopeBefore = (activeKeywordField, capsules.enumerated().compactMap { $0.element.scopeField == nil ? nil : ($0.offset, $0.element) })
            if let editingCapsuleID { capsules.removeAll { $0.id == editingCapsuleID } }
            capsules.removeAll { $0.scopeField != nil }
            activeKeywordField = candidate.id
            capsules.append(Capsule(id: UUID(), text: candidate.name, document: QueryDocumentRecord(nodes: [], root: nil), condition: nil, scopeField: candidate.id))
            candidateField = candidate.id; candidateValue = ""; editingCapsuleID = nil
            candidateTail = keywords; tailText = keywords
            queryAnalysis = nil; requestCandidateFocus(); recalcHeight()
            return
        }
        commitCandidate(candidate)
    }
    /// Empty Enter, Escape, or Tab accepts only the selected range and resumes keywords.
    func skipOptionalFieldValue() {
        guard editingOptionalFieldValue else { return }
        optionalScopeBefore = nil; candidateKind = nil; candidateField = nil; candidateValue = ""
        tailText = candidateTail; rebuildCapsuleQuery(); focusRequest += 1
    }
    /// A supplied value is an explicit typed restriction; the previous UI range is restored.
    private func commitOptionalFieldValue() {
        guard let field = candidateField, let previous = optionalScopeBefore else { return }
        if candidateValue.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty { skipOptionalFieldValue(); return }
        let condition = QueryCondition(fieldName: field, textValue: candidateValue, phrase: false, exclude: false, tagId: nil)
        let built = BibCiTeXCore.buildCondition(condition: condition)
        guard built.status == .valid, let operand = built.document else {
            queryAnalysis = built; searchContext = nil; searchResults = []; selectedReferenceIndex = nil; recalcHeight(); return
        }
        // Validate the original keyword operand before changing any confirmed capsules.
        let words = keywordOperand(candidateTail)
        guard words.status == .valid else { queryAnalysis = words; recalcHeight(); return }
        capsules.removeAll { $0.scopeField != nil }
        for (index, capsule) in previous.1 { capsules.insert(capsule, at: min(index, capsules.count)) }
        activeKeywordField = previous.0
        if let document = words.document, document.root != nil {
            capsules.append(Capsule(id: UUID(), text: words.text, document: document, condition: nil))
        }
        capsules.append(Capsule(id: UUID(), text: built.text, document: operand, condition: condition))
        optionalScopeBefore = nil; candidateKind = nil; candidateField = nil; candidateValue = ""
        tailText = ""; rebuildCapsuleQuery(); focusRequest += 1
    }
    private func commitCandidate(_ candidate: Candidate? = nil) {
        candidateOperation += 1
        let operation = candidateOperation, version = searchVersion, session = sessionGeneration
        let kind = candidateKind
        let field = candidateField ?? (kind == "@" ? "type" : "tag")
        let value = candidate.map { kind == "@" ? $0.id : $0.name } ?? candidateValue
        let condition = QueryCondition(fieldName: field, textValue: value,
            phrase: candidateSupportsPhrase && candidatePhrase, exclude: candidateExclude,
            tagId: kind == "#" ? candidate?.id : nil)
        let originalText = candidateOriginal, document = boundDraft.document, revision = candidateTags?.revision
        let replacing = editingCapsuleID, priorTail = candidateTail
        func current() -> Bool { operation == candidateOperation && version == searchVersion && session == sessionGeneration }
        Task {
            let built = await RustCore.shared.buildCondition(condition)
            guard current() else { return }
            guard built.status == .valid, let operand = built.document else { queryAnalysis = built; recalcHeight(); return }
            for index in capsules.indices where capsules[index].scopeField == nil {
                let refreshed: QueryAnalysis
                do { refreshed = try await RustCore.shared.refreshQuery(capsules[index].document) }
                catch { guard current() else { return }; errorDetails = LocalizedMessage(error: error); return }
                guard current() else { return }
                guard refreshed.status == .valid, let refreshedDocument = refreshed.document else { continue }
                capsules[index].document = refreshedDocument; capsules[index].text = refreshed.text
                if var condition = capsules[index].condition, condition.tagId != nil,
                   let tag = refreshedDocument.nodes.first(where: { $0.tagId != nil }) {
                    condition.tagId = tag.tagId; condition.textValue = tag.textValue; capsules[index].condition = condition
                }
            }
            if let replacing, let index = capsules.firstIndex(where: { $0.id == replacing }) {
                capsules[index] = Capsule(id: replacing, text: built.text, document: operand, condition: condition)
                candidateKind = nil; candidateField = nil; candidateParentKind = nil; editingCapsuleID = nil
                tailText = priorTail; capsuleRevision = revision; rebuildCapsuleQuery(); focusRequest += 1
                return
            }
            let syntax = await RustCore.shared.analyzeQuery(originalText)
            guard current() else { return }
            guard syntax.status == .valid else { queryAnalysis = syntax; recalcHeight(); return }
            let original: QueryAnalysis
            if let document {
                do { original = try await RustCore.shared.refreshQuery(document) }
                catch { guard current() else { return }; errorDetails = LocalizedMessage(error: error); return }
            } else { original = syntax }
            guard current() else { return }
            guard original.status == .valid, let document = original.document else { queryAnalysis = original; return }
            let result = await RustCore.shared.editQuery(QueryEdit(document: document, action: .appendAnd, target: nil, operand: operand))
            guard current() else { return }
            guard result.status == .valid else { queryAnalysis = result; return }
            candidateKind = nil; candidateField = nil; candidateParentKind = nil
            if let replacing, let index = capsules.firstIndex(where: { $0.id == replacing }) {
                capsules[index] = Capsule(id: replacing, text: built.text, document: operand, condition: condition)
                tailText = priorTail
            } else {
                if capsules.isEmpty, let originalDocument = original.document, originalDocument.root != nil {
                    capsules.append(Capsule(id: UUID(), text: original.text, document: originalDocument, condition: nil))
                } else if !capsules.isEmpty, !priorTail.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                    let prior = keywordOperand(priorTail)
                    if prior.status == .valid, let priorDoc = prior.document { capsules.append(Capsule(id: UUID(), text: prior.text, document: priorDoc, condition: nil)) }
                }
                capsules.append(Capsule(id: UUID(), text: built.text, document: operand, condition: condition))
                tailText = ""
            }
            editingCapsuleID = nil; capsuleRevision = revision
            rebuildCapsuleQuery()
            focusRequest += 1
        }
    }
    /// Commits a service-built tree and its tag snapshot as the helper’s one active query.
    func applyAdvancedQuery(_ text: String, draft: SearchDraft) {
        boundDraft = draft; updateQuery(text)
    }
    func suspendForComposition(_ composing: Bool) {
        guard composing else { if capsules.isEmpty { updateQuery(query) } else { rebuildCapsuleQuery() }; return }
        invalidateQueryEdit(); invalidateSearch(); searchContext = nil
    }
    /// Applies an actual native edit range; whole replacements deliberately resolve names anew.
    func editText(_ text: String, start: UInt32, end: UInt32, replacement: String, full: Bool) {
        if candidateKind != nil {
            if let document = boundDraft.document {
                let offset = UInt32(candidateOriginal.utf8.count)
                let materialized = BibCiTeXCore.editQueryText(edit: QueryTextEdit(document: document, text: candidateOriginal,
                    start: offset, end: offset, replacement: candidateTrigger, fullReplace: false))
                boundDraft.document = materialized.document
            }
            candidateKind = nil; candidateField = nil
        }
        guard let document = boundDraft.document, !full else { requestQuery(text); return }
        let result = BibCiTeXCore.editQueryText(edit: QueryTextEdit(document: document, text: query, start: start, end: end, replacement: replacement, fullReplace: false))
        boundDraft.document = result.document; queryAnalysis = result; updateQuery(text)
    }
    func tagsChanged() {
        invalidateSearch()
        searchContext = nil
        let state = stateVersion, session = sessionGeneration
        Task {
            do {
                let updated = try await service.loadBibliographyStateIfAvailable()
                guard state == stateVersion, session == sessionGeneration else { return }
                bibliographies = updated.bibs
                if let uid = currentBibliography?.uid {
                    currentBibliography = updated.bibs.first { $0.uid == uid && $0.name == currentBibliography?.name }
                        ?? updated.bibs.first { $0.uid == uid }
                    if currentBibliography == nil { isSelectingBibliography = true }
                }
                refreshSearch()
            } catch {
                guard state == stateVersion, session == sessionGeneration else { return }
                errorDetails = LocalizedMessage(error: error)
            }
        }
    }
    func refreshSearch() {
        updateQuery(query)
        if candidateKind != nil {
            let version = searchVersion, session = sessionGeneration
            Task {
                do {
                    let tags = try await RustCore.shared.tagSnapshot(path: "", citeKey: "")
                    guard version == searchVersion, session == sessionGeneration, candidateKind != nil else { return }
                    candidateTags = tags
                } catch {
                    guard version == searchVersion, session == sessionGeneration, candidateKind != nil else { return }
                    errorDetails = LocalizedMessage(error: error)
                }
            }
        }
    }
    @Published var pasteTargetApplicationName: String?
    @Published var isPasting = false
    @Published private(set) var isPasteOperationPending = false
    @Published private(set) var bibliographies: [Bibliography] = []
    @Published private(set) var searchResults: [Reference] = [] { didSet { pasteSelection = nil; pasteAnchor = nil } }
    @Published private(set) var currentBibliography: Bibliography?
    @Published private(set) var isSelectingBibliography = true
    @Published private(set) var selectedBibliographyIndex: Int?
    @Published private(set) var selectedReferenceIndex: Int?
    @Published private(set) var preferredHeight: CGFloat = HelperMetrics.header

    @Published var focusRequest = 0
    @Published private(set) var isSearching = false
    @Published private(set) var isLoading = false
    @Published var query = ""
    @Published private(set) var errorDetails: LocalizedMessage?
    var errorMessage: String? {
        get { errorDetails?.text }
        set { errorDetails = newValue.map { LocalizedMessage(literal: $0) } }
    }
    @Published var theme: ThemeStyle = .latte

    /// The cite key most recently copied, so that row can confirm the copy in
    /// place. Cleared whenever the rows change, since the confirmation belongs to
    /// a row that will no longer be there.
    @Published private(set) var copiedKey: String?
    /// The key whose paste failed. Only the paste surface sets it, and the error
    /// bar offers it for copying, since that surface's rows do not.
    @Published private(set) var failedPasteKey: String?

    private let service: any HelperServing
    private let requestPastePermission: @MainActor () -> Bool
    private var searchTask: Task<Void, Never>?
    private var queryEditTask: Task<Void, Never>?
    private var copyConfirmation: Task<Void, Never>?
    private var queryEditVersion = 0
    private var searchVersion = 0
    private var stateVersion = 0
    private(set) var sessionGeneration = 0

    /// Invalidates UI completions; an already-started OS paste cannot be cancelled here.
    func invalidateSession() {
        invalidateQueryEdit()
        sessionGeneration += 1
        isPasting = false
        invalidateSearch()
        stateVersion += 1
        isLoading = false
    }

    @discardableResult
    func beginSession() -> Int {
        invalidateSession()
        boundDraft = SearchDraft(); candidateKind = nil; candidateField = nil
        queryAnalysis = nil; searchContext = nil; matchEvidence = [:]; librarySwitchDraft = nil
        capsules = []; tailText = ""; capsuleRevision = nil; editingCapsuleID = nil; librarySwitchCapsules = nil; activeKeywordField = nil; savedKeywordField = nil; optionalScopeBefore = nil
        return sessionGeneration
    }

    // Row heights come from `HelperMetrics`, which the view also draws from, so
    // the panel never clips a row or leaves dead space under the last one.
    private let rowHeightBib = HelperMetrics.libraryRow
    private let rowHeightSearch = HelperMetrics.referenceRow
    private let headerHeight = HelperMetrics.header
    private let minHeight = HelperMetrics.header
    private let maxListHeight = 460.0

    var filteredBibliographies: [Bibliography] {
        guard query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false else {
            return bibliographies
        }
        let normalized = query.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        return bibliographies.filter { bib in
            [bib.name, bib.path, bib.updatedAt, bib.descriptionText ?? ""].contains(where: {
                $0.lowercased().contains(normalized)
            })
        }
    }

    init(service: any HelperServing, requestPastePermission: @escaping @MainActor () -> Bool = { AccessibilityPermission.ensureTrusted() }) {
        self.service = service
        self.requestPastePermission = requestPastePermission
    }

    func setTheme(_ themeMode: Int32) {
        updateTheme(ThemeStyle(mode: themeMode))
    }

    func loadState() {
        isLoading = true
        invalidateSearch()
        stateVersion += 1
        let version = stateVersion
        query = ""
        searchResults = []
        selectedReferenceIndex = nil
        errorMessage = nil
        failedPasteKey = nil
        Task {
            do {
                let state = try await service.loadBibliographyStateIfAvailable()
                guard version == stateVersion else { return }
                isLoading = false
                bibliographies = state.bibs.sorted {
                    $0.updatedAt.localizedStandardCompare($1.updatedAt) == .orderedDescending
                }
                if let current = state.current, current.available,
                    let index = bibliographies.firstIndex(where: { $0.uid == current.uid && $0.name == current.name }) {
                    currentBibliography = bibliographies[index]
                    isSelectingBibliography = false
                    selectedBibliographyIndex = index
                } else {
                    currentBibliography = nil
                    isSelectingBibliography = true
                    selectedBibliographyIndex = bibliographies.isEmpty ? nil : 0
                }
                updateQuery(query)
            } catch {
                guard version == stateVersion else { return }
                isLoading = false
                errorDetails = LocalizedMessage(key: "加载文献库失败：{0}", arguments: [LocalizedMessage(error: error)])
                isSelectingBibliography = true
                recalcHeight()
            }
        }
    }

    func updateTheme(_ style: ThemeStyle) {
        guard theme != style else { return }
        theme = style
    }

    /// A TextField may call its binding setter while SwiftUI reconciles the view.
    /// Coalesce edits outside that call stack, and discard edits from an old session/library.
    func requestQuery(_ value: String) {
        invalidateQueryEdit()
        guard value != query else { return }
        boundDraft = SearchDraft()
        if candidateKind != nil { candidateKind = nil; candidateField = nil }
        updateQuery(value)
    }

    private func invalidateQueryEdit() {
        queryEditVersion += 1
        queryEditTask?.cancel()
        queryEditTask = nil
    }

    func updateQuery(_ nextQuery: String) {
        invalidateQueryEdit()
        query = nextQuery
        queryAnalysis = nil; matchEvidence = [:]; searchContext = nil
        errorMessage = nil
        failedPasteKey = nil

        invalidateSearch()
        searchResults = []
        selectedReferenceIndex = nil

        if isSelectingBibliography {
            if filteredBibliographies.isEmpty {
                selectedBibliographyIndex = nil
            } else {
                selectedBibliographyIndex = 0
            }
            recalcHeight()
            return
        }

        let trimmed = nextQuery.trimmingCharacters(in: .whitespacesAndNewlines)
        let version = searchVersion
        if !trimmed.isEmpty && !editingCondition {
            isSearching = true
            searchTask = Task {
                do {
                    try await Task.sleep(nanoseconds: 90_000_000)
                    try Task.checkCancellation()
                    await performSearch(nextQuery, version: version)
                } catch {
                    // A new query or bibliography superseded this search.
                }
            }
        }

        recalcHeight()
    }

    func startSelectMode() {
        if candidateKind != nil { cancelCandidates() }
        if !isSelectingBibliography {
            librarySwitchDraft = (query, boundDraft); librarySwitchCapsules = (capsules, tailText, capsuleRevision); savedKeywordField = activeKeywordField
        }
        capsules = []; tailText = ""
        boundDraft = SearchDraft()
        isLoading = false
        invalidateSearch()
        stateVersion += 1
        isSelectingBibliography = true
        query = ""
        searchResults = []
        selectedReferenceIndex = nil
        selectedBibliographyIndex = bibliographies.isEmpty ? nil : 0
        errorMessage = nil
        failedPasteKey = nil
        recalcHeight()
    }

    func chooseCurrentBibliography() {
        guard !isLoading, isSelectingBibliography, let index = selectedBibliographyIndex else {
            return
        }
        let list = filteredBibliographies
        guard list.indices.contains(index) else {
            return
        }
        selectBibliography(list[index])
    }

    func selectBibliography(_ bibliography: Bibliography, at index: Int? = nil) {
        guard bibliography.available else {
            if let index { selectedBibliographyIndex = index }
            return
        }
        invalidateSearch()
        stateVersion += 1
        let version = stateVersion
        isLoading = true
        if let index {
            selectedBibliographyIndex = index
        }
        query = ""
        selectedReferenceIndex = nil
        errorMessage = nil
        failedPasteKey = nil

        Task { @MainActor in
            do {
                let current = try await service.setCurrentBibliography(name: bibliography.name, path: bibliography.path)
                guard version == stateVersion else { return }
                isLoading = false
                currentBibliography = current
                isSelectingBibliography = false
                selectedReferenceIndex = nil
                searchResults = []
                if let draft = librarySwitchDraft {
                    boundDraft = draft.1; librarySwitchDraft = nil
                    if let saved = librarySwitchCapsules { capsules = saved.0; tailText = saved.1; capsuleRevision = saved.2; librarySwitchCapsules = nil; activeKeywordField = savedKeywordField }
                    updateQuery(draft.0)
                } else { updateQuery(query) }
            } catch {
                guard version == stateVersion else { return }
                isLoading = false
                errorDetails = LocalizedMessage(key: "加载失败：{0}", arguments: [LocalizedMessage(error: error)])
                recalcHeight()
            }
        }
    }

    func moveSelection(delta: Int, extending: Bool = false) {
        guard delta != 0 else { return }

        if isSelectingBibliography {
            let items = filteredBibliographies
            guard !items.isEmpty else {
                selectedBibliographyIndex = nil
                return
            }

            let start = selectedBibliographyIndex ?? (delta > 0 ? -1 : 0)
            let count = items.count
            let next = (start + delta + count) % count
            selectedBibliographyIndex = next
            return
        }

        guard !searchResults.isEmpty else {
            selectedReferenceIndex = nil
            return
        }
        let start = selectedReferenceIndex ?? (delta > 0 ? -1 : 0)
        let count = searchResults.count
        let next = min(max(start + delta, 0), count - 1)
        selectForPaste(at: next, extending: extending, toggling: false)
    }

    @Published private(set) var pasteSelection: Set<String>?
    private var pasteAnchor: Int?

    func isSelectedForPaste(_ reference: Reference, at index: Int) -> Bool {
        pasteSelection?.contains(reference.id) ?? (selectedReferenceIndex == index)
    }

    func selectAllForPaste() {
        guard !isSelectingBibliography, !isSearching else { return }
        pasteSelection = Set(searchResults.map(\.id))
        pasteAnchor = selectedReferenceIndex
    }

    func selectForPaste(at index: Int, extending: Bool, toggling: Bool) {
        guard searchResults.indices.contains(index) else { return }
        let previous = selectedReferenceIndex ?? index
        var ids = pasteSelection ?? Set([searchResults[previous].id])
        if extending {
            let anchor = pasteAnchor ?? previous
            pasteAnchor = anchor
            if !toggling { ids.removeAll() }
            ids.formUnion(searchResults[min(anchor, index)...max(anchor, index)].map(\.id))
        } else if toggling {
            let id = searchResults[index].id
            if !ids.insert(id).inserted { ids.remove(id) }
            pasteAnchor = index
        } else {
            ids = [searchResults[index].id]
            pasteAnchor = index
        }
        pasteSelection = ids
        selectedReferenceIndex = index
    }

    /// Return and row activation paste into the previously focused application.
    func activate(_ reference: Reference, at index: Int? = nil, onSuccess: @escaping () -> Void = {}) {
        pasteSelection = [reference.id]
        if let index {
            selectedReferenceIndex = index
            pasteAnchor = index
        }
        paste([reference], onSuccess: onSuccess)
    }

    func activateSelection(onSuccess: @escaping () -> Void) {
        if isSelectingBibliography {
            chooseCurrentBibliography()
            return
        }
        if let ids = pasteSelection {
            paste(searchResults.filter { ids.contains($0.id) }, onSuccess: onSuccess)
            return
        }
        guard let index = selectedReferenceIndex, searchResults.indices.contains(index) else {
            return
        }
        activate(searchResults[index], onSuccess: onSuccess)
    }

    private func paste(_ references: [Reference], onSuccess: @escaping () -> Void) {
        guard !isPasteOperationPending, !editingCondition, !isSearching, searchContext != nil, !references.isEmpty, references.allSatisfy({ reference in searchResults.contains(where: { $0.id == reference.id }) }) else { return }
        let key = references.map(\.citeKey).joined(separator: ",")
        let version = searchVersion
        let session = sessionGeneration
        isPasting = true
        isPasteOperationPending = true
        errorMessage = nil
        let hasPermission = requestPastePermission()

        Task { @MainActor in
            defer { isPasteOperationPending = false; if session == sessionGeneration { isPasting = false } }
            guard session == sessionGeneration else { return }
            do {
                guard hasPermission else {
                    throw HelperError.ffiError("缺少辅助功能权限，无法控制其他应用")
                }
                guard let context = searchContext, try await RustCore.helper.validateContext(context), version == searchVersion, session == sessionGeneration else { refreshSearch(); return }
                onPasteStart?()
                try await service.copyAndPaste(citeKey: key)
                guard session == sessionGeneration else { return }
                isPasting = false
                onSuccess()
            } catch {
                guard session == sessionGeneration else { return }
                let pasteError = LocalizedMessage(error: error)
                do {
                    guard let context = searchContext, try await RustCore.helper.validateContext(context), version == searchVersion else { refreshSearch(); return }
                    try await service.copy(citeKey: key)
                    guard session == sessionGeneration else { return }
                    failedPasteKey = nil
                } catch {
                    guard session == sessionGeneration else { return }
                    failedPasteKey = key
                }
                isPasting = false
                errorDetails = LocalizedMessage(key: "粘贴失败：{0}", arguments: [pasteError])
                recalcHeight()
                onPasteFailure?()
            }
        }
    }

    /// Copies an explicit key, for the paste surface's failure bar, where there is
    /// no row control to press.
    func copyKey(_ key: String) {
        guard !key.isEmpty, !editingCondition, let context = searchContext else { return }
        let session = sessionGeneration
        let version = searchVersion
        Task { @MainActor in
            do {
                guard try await RustCore.helper.validateContext(context), session == sessionGeneration, version == searchVersion else { refreshSearch(); return }
                try await service.copy(citeKey: key)
                guard session == sessionGeneration else { return }
                confirmCopy(of: key)
                if failedPasteKey == key { failedPasteKey = nil; recalcHeight() }
            } catch {
                guard session == sessionGeneration else { return }
                errorDetails = LocalizedMessage(key: "复制失败：{0}", arguments: [LocalizedMessage(error: error)])
                recalcHeight()
            }
        }
    }

    /// A brief in-place confirmation: a click on a cite key is answered on the row
    /// itself, without an alert or a message the user has to dismiss.
    private func confirmCopy(of key: String) {
        copiedKey = key
        copyConfirmation?.cancel()
        copyConfirmation = Task { @MainActor [weak self] in
            try? await Task.sleep(nanoseconds: 1_400_000_000)
            guard let self, !Task.isCancelled else { return }
            if self.copiedKey == key { self.copiedKey = nil }
        }
    }

    private func performSearch(_ queryText: String, version: Int) async {
        do {
            guard let library = currentBibliography else { return }
            let response = try await RustCore.helper.searchExplicit(path: library.path, query: queryText, advanced: true, draft: boundDraft, version: UInt64(version))
            let found = response.references
            guard version == searchVersion, !Task.isCancelled, !isSelectingBibliography else { return }
            isSearching = false
            if let text = response.refreshedText, let draft = response.refreshedDraft {
                query = text; boundDraft = draft
                if !capsules.isEmpty {
                    for index in capsules.indices where capsules[index].scopeField == nil {
                        let refreshed = try await RustCore.helper.refreshQuery(capsules[index].document)
                        guard version == searchVersion, !Task.isCancelled else { return }
                        guard refreshed.status == .valid, let document = refreshed.document else {
                            queryAnalysis = refreshed; searchContext = nil; searchResults = []; selectedReferenceIndex = nil; recalcHeight(); return
                        }
                        capsules[index].document = document; capsules[index].text = refreshed.text
                        if var condition = capsules[index].condition, let id = condition.tagId,
                           let tag = document.nodes.first(where: { $0.tagId == id || $0.tagId != nil }) {
                            condition.tagId = tag.tagId; condition.textValue = tag.textValue; capsules[index].condition = condition
                        }
                    }
                    capsuleRevision = draft.tagRevision
                }
            }
            matchRanges = response.matchRanges
            matchEvidence = response.evidence
            queryAnalysis = response.analysis; searchContext = response.context
            searchResults = found
            selectedReferenceIndex = found.isEmpty ? nil : 0
            errorMessage = nil
            failedPasteKey = nil
            recalcHeight()
        } catch {
            guard version == searchVersion, !Task.isCancelled, !isSelectingBibliography else { return }
            searchResults = []
            selectedReferenceIndex = nil
            isSearching = false
            errorDetails = LocalizedMessage(key: "搜索失败：{0}", arguments: [LocalizedMessage(error: error)])
            recalcHeight()
        }
    }

    private func invalidateSearch() {
        isSearching = false
        searchTask?.cancel()
        searchTask = nil
        searchVersion += 1
    }

    /// The floating key hints belong to a result list, where Return and Tab both apply.
    var showsBottomBar: Bool {
        !isLoading && !isSelectingBibliography && !searchResults.isEmpty
            && !query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    func recalcHeight() {
        let listHeight: CGFloat

        if candidateKind != nil {
            listHeight = 0
        } else if isLoading {
            listHeight = HelperMetrics.emptyRow
        } else if isSelectingBibliography && filteredBibliographies.isEmpty {
            listHeight = HelperMetrics.emptyRow
        } else if isSelectingBibliography {
            let count = filteredBibliographies.count
            listHeight = min(CGFloat(count) * (rowHeightBib + 2) + HelperMetrics.listPadding, maxListHeight)
        } else if query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            listHeight = 0
        } else {
            if searchResults.isEmpty {
                listHeight = HelperMetrics.emptyRow
            } else {
                // The bottom safe-area inset consumes space in addition to the rows.
                listHeight = min(CGFloat(searchResults.count) * (rowHeightSearch + 2) + HelperMetrics.listPadding, maxListHeight)
            }
        }

        // The error bar is self-contained: any copy affordance lives on the row,
        // not in a second bar under the message.
        let errorHeight = (errorMessage == nil ? 0 : HelperMetrics.errorBar)
        let diagnosticHeight = CGFloat(min(queryAnalysis?.diagnostics.count ?? 0, 3)) * 18
        let height = max(minHeight, headerHeight + listHeight + (showsBottomBar ? HelperMetrics.bottomBar : 0) + errorHeight + diagnosticHeight)
        if preferredHeight != height { preferredHeight = height }
    }
}
