import AppKit
import CoreFoundation
import SwiftUI
import BibCiTeXCore

/// The non-persistent bound portion of one window mode’s query draft.
struct SearchDraft: Sendable {
    var document: QueryDocumentRecord?
    var tagRevision: UInt64?
}

/// Search rows and their grammar diagnostics share one actionable snapshot context.
struct NativeSearchResult: Sendable {
    let references: [Reference]
    let analysis: QueryAnalysis
    let context: SearchContext?
    let evidence: [String: String]
    let matchRanges: [String: [SearchMatchRange]]
    let refreshedText: String?
    let refreshedDraft: SearchDraft?
}

extension Notification.Name {
    static let searchTagsChanged = Notification.Name("BibCiTeX.searchTagsChanged")
}

/// Editing remains transactional: the sheet owns its draft until Apply succeeds.
struct AdvancedSearchEditor: View {
    let text: String
    let draft: SearchDraft
    let apply: (String, SearchDraft) -> Void
    @Environment(\.dismiss) private var dismiss
    @State private var analysis: QueryAnalysis?
    @State private var catalog: SearchCatalog?
    @State private var tags: TagSnapshot?
    @State private var selected: UInt32?
    @State private var field = ""
    @State private var value = ""
    @State private var yearFrom = ""
    @State private var yearTo = ""
    @State private var phrase = false
    @State private var exclude = false
    @State private var tagID = ""
    @State private var combineOr = false
    @State private var matchStyle = "all"
    @State private var error: String?
    @State private var showTags = false
    @State private var busy = false
    @State private var editGeneration = 0

    private var fieldDescriptor: SearchFieldDescriptor? { catalog?.fieldDescriptors.first { $0.name == field } }
    private var supportsPhrase: Bool { field.isEmpty || fieldDescriptor?.supportsPhrase == true }
    private var matchMode: String { field.isEmpty ? "text" : (fieldDescriptor?.matchMode ?? "") }

    var body: some View {
        ScrollView {
        VStack(alignment: .leading, spacing: 12) {
            Text(L10n.text("搜索条件")).font(.headline)
            if let document = analysis?.document {
                ScrollView {
                    VStack(alignment: .leading, spacing: 5) {
                        ForEach(visibleNodes(document), id: \.index) { row in
                            Button { selected = row.index } label: {
                                HStack {
                                    Text(String(repeating: "  ", count: min(row.depth, 12)) + label(document.nodes[Int(row.index)]))
                                    Spacer()
                                }.padding(6).background(selected == row.index ? Color.accentColor.opacity(0.15) : Color.clear)
                            }.buttonStyle(.plain).accessibilityAddTraits(selected == row.index ? .isSelected : [])
                        }
                    }
                }.frame(maxHeight: 220)
                HStack {
                    Button(L10n.text("整组取反")) { edit(.negate) }.disabled(selected == nil)
                    Button(L10n.text("移除")) { edit(.remove) }.disabled(selected == nil)
                    Button(L10n.text("全部条件")) { selected = nil }
                }
            }
            Divider()
            Picker(L10n.text("字段"), selection: $field) {
                Text(L10n.text("全部字段")).tag("")
                ForEach(catalog?.fieldDescriptors ?? [], id: \.name) { Text(L10n.text($0.labelKey)).tag($0.name) }
            }.onChange(of: field) { _ in phrase = false; matchStyle = "all"; value = "" }
            if matchMode == "tag" {
                Picker(L10n.text("标签"), selection: $tagID) {
                    Text(L10n.text("选择标签")).tag("")
                    ForEach(tags?.tags ?? [], id: \.id) { Text($0.name).tag($0.id) }
                }
                Button(L10n.text("标签管理")) { showTags = true }
            } else if matchMode == "type" {
                Picker(L10n.text("类型"), selection: $value) {
                    Text(L10n.text("选择类型")).tag("")
                    ForEach(catalog?.typeDescriptors ?? [], id: \.name) { Text(L10n.text($0.labelKey)).tag($0.name) }
                }
            } else {
                TextField(L10n.text("条件值"), text: $value).textFieldStyle(.roundedBorder)
            }
            if matchMode == "year" {
                HStack {
                    TextField(L10n.text("起始年份"), text: $yearFrom)
                    Text("…")
                    TextField(L10n.text("结束年份"), text: $yearTo)
                }.textFieldStyle(.roundedBorder)
                    .onChange(of: yearFrom) { _ in value = yearFrom + ".." + yearTo }
                    .onChange(of: yearTo) { _ in value = yearFrom + ".." + yearTo }
            }
            Picker(L10n.text("匹配方式"), selection: $matchStyle) {
                Text(L10n.text("包含全部关键词")).tag("all")
                Text(L10n.text("包含任意关键词")).tag("any")
                Text(L10n.text("包含完整短语")).tag("phrase")
            }.disabled(!supportsPhrase)
            HStack {
                Toggle(L10n.text("完整短语"), isOn: $phrase).disabled(!supportsPhrase)
                Toggle(L10n.text("排除"), isOn: $exclude)
                Picker(L10n.text("组合"), selection: $combineOr) { Text("AND").tag(false); Text("OR").tag(true) }.frame(width: 130)
            }
            HStack {
                Button(L10n.text("添加条件")) { addCondition() }
                    .disabled(busy || (matchMode == "tag" ? tagID.isEmpty : value.isEmpty))
                Button(L10n.text("添加条件组")) { addCondition() }
                    .disabled(busy || value.isEmpty || matchMode != "text")
                Button(L10n.text("替换条件")) { addCondition(replace: true) }
                    .disabled(busy || selected == nil || (matchMode == "tag" ? tagID.isEmpty : value.isEmpty))
            }
            if let analysis {
                Text(analysis.text).font(.system(.caption, design: .monospaced)).textSelection(.enabled)
                QueryDiagnostics(analysis: analysis)
            }
            if let error { Text(error).foregroundStyle(.red).font(.caption) }
            HStack {
                Spacer()
                Button(L10n.text("取消")) { dismiss() }.keyboardShortcut(.cancelAction)
                Button(L10n.text("应用")) {
                    guard let analysis, analysis.status == .valid else { return }
                    apply(analysis.text, SearchDraft(document: analysis.document, tagRevision: analysis.tagRevision ?? tags?.revision))
                    dismiss()
                }.keyboardShortcut(.defaultAction).disabled(analysis?.status != .valid || busy)
            }
        }.padding(18)
        }.frame(width: 560, height: 640)
            .task { load() }
            .sheet(isPresented: $showTags) { TagManagerView(path: "", citeKey: "") }
            .onReceive(NotificationCenter.default.publisher(for: .searchTagsChanged)) { _ in load() }
    }
    private func label(_ node: QueryNodeRecord) -> String {
        switch node.kind {
        case .and: return "AND"
        case .or: return "OR"
        case .not: return "NOT"
        case .scope: return catalog?.fieldDescriptors.first { $0.name == node.textValue }.map { L10n.text($0.labelKey) + ":" } ?? (node.textValue + ":")
        case .phrase: return "\"" + node.textValue + "\""
        case .word: return node.textValue
        }
    }
    private struct Row { let index: UInt32; let depth: Int }
    private func visibleNodes(_ doc: QueryDocumentRecord) -> [Row] {
        guard let root = doc.root else { return [] }
        var pending = [Row(index: root, depth: 0)], result: [Row] = []
        while let row = pending.popLast(), result.count < 4096 {
            guard Int(row.index) < doc.nodes.count else { continue }
            result.append(row)
            pending.append(contentsOf: doc.nodes[Int(row.index)].children.reversed().map { Row(index: $0, depth: row.depth + 1) })
        }
        return result
    }
    private func load() {
        editGeneration += 1
        let version = editGeneration
        let current = analysis
        Task {
            let loadedCatalog = await RustCore.shared.searchCatalog()
            do {
                let resolved: QueryAnalysis
                if let current, current.status == .valid, let document = current.document {
                    resolved = try await RustCore.shared.refreshQuery(document)
                } else {
                    let syntax = await RustCore.shared.analyzeQuery(text)
                    if syntax.status == .valid, let document = draft.document { resolved = try await RustCore.shared.refreshQuery(document) }
                    else { resolved = syntax }
                }
                guard version == editGeneration else { return }
                catalog = loadedCatalog; analysis = resolved
                let loadedTags = try await RustCore.shared.tagSnapshot(path: "", citeKey: "")
                guard version == editGeneration else { return }
                tags = loadedTags
            } catch {
                guard version == editGeneration else { return }
                catalog = loadedCatalog; self.error = error.localizedDescription
            }
        }
    }
    private func edit(_ action: QueryEditAction, operand: QueryDocumentRecord? = nil) {
        guard analysis?.status == .valid, let document = analysis?.document else { return }
        editGeneration += 1
        do {
            let result = BibCiTeXCore.editQuery(edit: QueryEdit(document: document, action: action, target: selected, operand: operand))
            analysis = result; selected = nil
        }
    }
    private func addCondition(replace: Bool = false) {
        editGeneration += 1
        busy = true
        do {
            defer { busy = false }
            let selectedTag = tags?.tags.first { $0.id == tagID }
            let result = BibCiTeXCore.buildCondition(condition: QueryCondition(fieldName: field, textValue: matchMode == "tag" ? (selectedTag?.name ?? "") : value,
                phrase: supportsPhrase && (phrase || matchStyle == "phrase"), exclude: exclude && matchStyle != "any", tagId: matchMode == "tag" ? selectedTag?.id : nil))
            guard result.status == .valid, var operand = result.document else { error = result.diagnostics.map(\.code).joined(separator: ", "); return }
            if matchStyle == "any", var root = operand.root {
                // The service constructed token boundaries; only the chosen group operator changes.
                while Int(root) < operand.nodes.count, [.scope, .not].contains(operand.nodes[Int(root)].kind), let child = operand.nodes[Int(root)].children.first { root = child }
                if Int(root) < operand.nodes.count, operand.nodes[Int(root)].kind == .and { operand.nodes[Int(root)].kind = .or }
            }
            if matchStyle == "any" && exclude {
                let negated = BibCiTeXCore.editQuery(edit: QueryEdit(document: operand, action: .negate, target: nil, operand: nil))
                guard negated.status == .valid, let document = negated.document else { error = negated.diagnostics.map(\.code).joined(separator: ", "); return }
                operand = document
            }
            if let document = analysis?.document {
                analysis = BibCiTeXCore.editQuery(edit: QueryEdit(document: document, action: replace ? .replace : (combineOr ? .appendOr : .appendAnd), target: selected, operand: operand))
            } else { analysis = BibCiTeXCore.renderQuery(document: operand) }
            value = ""; error = nil
        }
    }
}

/// Inline UTF-8 source ranges supplied by Rust; warnings do not block valid queries.
struct QueryDiagnostics: View {
    let analysis: QueryAnalysis?
    var compact = false
    var body: some View {
        if let analysis, !analysis.diagnostics.isEmpty {
            VStack(alignment: .leading, spacing: 3) {
                ForEach(Array(analysis.diagnostics.prefix(compact ? 3 : 32).enumerated()), id: \.offset) { _, item in
                    Text("\(L10n.text(item.code)) · \(item.start)–\(item.end)")
                        .foregroundStyle(item.warning ? Color.secondary : Color.red)
                }
            }.font(.caption).textSelection(.enabled)
        }
    }
}

/// Captures the originating library before a context menu outlives its result row.
struct TagAssignmentTarget: Identifiable {
    let path: String
    let citeKey: String
    var id: String { path + "\u{1f}" + citeKey }
}

/// Shared macOS tag directory, single-reference assignment, and explicit recovery UI.
struct TagManagerView: View {
    let path: String
    let citeKey: String
    @Environment(\.dismiss) private var dismiss
    @State private var snapshot: TagSnapshot?
    @State private var name = ""
    @State private var selected = ""
    @State private var mergeTarget = ""
    @State private var pendingID = ""
    @State private var targetPath = ""
    @State private var targetKey = ""
    @State private var relocateSource = ""
    @State private var relocatePath = ""
    @State private var libraries: [Bibliography] = []
    @State private var targets: [Reference] = []
    @State private var error: String?
    @State private var busy = false
    var body: some View {
        ScrollView {
        VStack(alignment: .leading, spacing: 12) {
            Text(L10n.text("标签管理")).font(.headline)
            if !citeKey.isEmpty { Text(citeKey).font(.subheadline) }
            List {
                ForEach(snapshot?.tags ?? [], id: \.id) { tag in
                    HStack {
                        if !citeKey.isEmpty {
                            Toggle("", isOn: Binding(get: { snapshot?.assignedIds.contains(tag.id) ?? false }, set: { mutation($0 ? .assign : .unassign, id: tag.id) })).labelsHidden()
                        }
                        Button { selected = tag.id; name = tag.name } label: { Text(tag.name).frame(maxWidth: .infinity, alignment: .leading) }
                            .buttonStyle(.plain).accessibilityAddTraits(selected == tag.id ? .isSelected : [])
                    }
                        .listRowBackground(selected == tag.id ? Color.accentColor.opacity(0.15) : Color.clear)
                }
            }.frame(height: 160)
            TextField(L10n.text("标签名称"), text: $name).textFieldStyle(.roundedBorder)
            HStack {
                Button(L10n.text("创建")) { mutation(.create) }.disabled(name.isEmpty)
                Button(L10n.text("重命名")) { mutation(.rename, id: selected) }.disabled(selected.isEmpty || name.isEmpty)
                Button(L10n.text("删除")) { mutation(.delete, id: selected) }.disabled(selected.isEmpty)
            }
            HStack {
                Picker(L10n.text("合并到"), selection: $mergeTarget) {
                    Text(L10n.text("选择标签")).tag("")
                    ForEach(snapshot?.tags.filter { $0.id != selected } ?? [], id: \.id) { Text($0.name).tag($0.id) }
                }
                Button(L10n.text("合并标签")) { mutation(.merge, id: selected, target: mergeTarget) }.disabled(selected.isEmpty || mergeTarget.isEmpty)
            }
            DisclosureGroup(L10n.text("待关联记录")) {
                Picker(L10n.text("旧记录"), selection: $pendingID) {
                    Text(L10n.text("选择记录")).tag("")
                    ForEach(snapshot?.pending ?? [], id: \.recordId) { Text("\($0.citeKey) · \($0.path)").tag($0.recordId) }
                }
                Picker(L10n.text("目标文献库"), selection: $targetPath) {
                    Text(L10n.text("选择文献库")).tag("")
                    ForEach(libraries) { Text($0.name).tag($0.path) }
                }.onChange(of: targetPath) { path in
                    targetKey = ""; targets = []
                    Task { do { let rows = try await RustCore.shared.search(path: path, query: ""); if targetPath == path { targets = rows } } catch { self.error = error.localizedDescription } }
                }
                Picker(L10n.text("目标记录"), selection: $targetKey) {
                    Text(L10n.text("选择记录")).tag("")
                    ForEach(targets) { Text($0.citeKey + " · " + $0.title.map(\.text).joined()).tag($0.citeKey) }
                }
                Button(L10n.text("重新关联")) { mutation(.relink, id: pendingID) }.disabled(pendingID.isEmpty || targetPath.isEmpty || targetKey.isEmpty)
            }
            DisclosureGroup(L10n.text("重新定位来源")) {
                Picker(L10n.text("原来源"), selection: $relocateSource) {
                    Text(L10n.text("选择来源")).tag("")
                    ForEach(snapshot?.sources ?? [], id: \.id) { Text($0.path).tag($0.id) }
                }
                TextField(L10n.text("目标文件路径"), text: $relocatePath).textFieldStyle(.roundedBorder)
                HStack {
                    Button(L10n.text("选择文件")) {
                        let panel = NSOpenPanel(); panel.canChooseDirectories = false; panel.allowsMultipleSelection = false
                        if panel.runModal() == .OK { relocatePath = panel.url?.path ?? "" }
                    }
                    Button(L10n.text("重新定位来源")) { mutation(.relocate, id: relocateSource) }
                        .disabled(relocateSource.isEmpty || relocatePath.isEmpty)
                }
            }
            if let error { Text(error).font(.caption).foregroundStyle(.red) }
            HStack { Spacer(); Button(L10n.text("关闭")) { dismiss() }.keyboardShortcut(.cancelAction) }
        }.padding(18)
        }.frame(width: 540, height: 620).disabled(busy)
        .task { await reload() }
    }
    private func reload() async {
        do { snapshot = try await RustCore.shared.tagSnapshot(path: path, citeKey: citeKey); libraries = try await RustCore.shared.libraries() }
        catch { self.error = error.localizedDescription }
    }
    private func mutation(_ action: TagAction, id: String = "", target: String = "") {
        guard let revision = snapshot?.revision else { return }
        busy = true
        Task {
            defer { busy = false }
            do {
                _ = try await RustCore.shared.mutateTag(TagMutation(action: action, id: id, name: action == .relocate ? "" : name, targetId: action == .relocate ? "" : target,
                    path: action == .relocate ? relocatePath : (action == .relink ? targetPath : path), citeKey: action == .relocate ? "" : (action == .relink ? targetKey : citeKey), expectedRevision: revision))
                await reload(); error = nil
                NotificationCenter.default.post(name: .searchTagsChanged, object: nil)
            } catch { self.error = error.localizedDescription; await reload() }
        }
    }
}

/// AppKit supplies the actual replaced UTF-16 range; convert it against the old
/// string before editing. No string diff is used to infer tag identity.
struct SearchInput: NSViewRepresentable {
    let text: String
    let placeholder: String
    var fontSize: CGFloat = 13
    var focusRequest: Int = 0
    var caretAtEndOnFocus = false
    var isEditable = true
    let changed: (String, UInt32, UInt32, String, Bool) -> Void
    var move: ((Int) -> Void)?
    var compositionChanged: (Bool) -> Void = { _ in }
    var triggerInserted: ((String, String, String) -> Bool)?
    /// Optional token deletion at an empty native selection at the beginning of this editor.
    var deleteBackwardAtStart: (() -> Bool)?
    func makeCoordinator() -> Coordinator { Coordinator(self) }
    func makeNSView(context: Context) -> SearchTextView {
        let view = SearchTextView()
        view.isRichText = false; view.isAutomaticQuoteSubstitutionEnabled = false
        view.isAutomaticDashSubstitutionEnabled = false; view.isAutomaticTextReplacementEnabled = false
        view.isContinuousSpellCheckingEnabled = false; view.drawsBackground = false
        view.textContainerInset = .zero; view.textContainer?.lineFragmentPadding = 0
        view.font = .systemFont(ofSize: fontSize); view.string = text
        view.delegate = context.coordinator; view.setAccessibilityLabel(placeholder)
        view.nativeCommit = { [weak coordinator = context.coordinator, weak view] in
            guard let view else { return }; coordinator?.commit(view)
        }
        view.compositionBegan = { [weak coordinator = context.coordinator] old, range in coordinator?.beginComposition(old, range: range) }
        view.placeholder = placeholder
        view.acceptsTrigger = triggerInserted != nil
        if fontSize >= 20 { context.coordinator.requestFocus(view, request: focusRequest, initial: true) }
        view.isVerticallyResizable = false; view.isHorizontallyResizable = false
        view.textContainer?.maximumNumberOfLines = 1
        view.textContainer?.lineBreakMode = .byTruncatingTail
        return view
    }
    func updateNSView(_ view: SearchTextView, context: Context) {
        let coordinator = context.coordinator
        coordinator.parent = self
        view.placeholder = placeholder
        view.setAccessibilityLabel(placeholder)
        view.needsDisplay = true
        view.acceptsTrigger = triggerInserted != nil
        guard !view.isComposing, !view.hasActiveMarkedText, view.nativeMutationDepth == 0 else {
            coordinator.cancelFocus(); return
        }
        view.isEditable = isEditable
        if view.string != text {
            coordinator.composition = nil; coordinator.edit = nil; coordinator.lastCommittedText = text
            let selection = view.selectedRange()
            view.string = text
            view.setSelectedRange(NSRange(location: min(selection.location, (text as NSString).length), length: 0))
        }
        view.needsDisplay = true
        if coordinator.focusRequest != focusRequest { coordinator.requestFocus(view, request: focusRequest) }
    }
    static func dismantleNSView(_ view: SearchTextView, coordinator: Coordinator) {
        view.nativeCommit = nil; view.compositionBegan = nil; view.delegate = nil
        coordinator.cancelFocus()
        coordinator.composition = nil; coordinator.edit = nil
    }
    final class Coordinator: NSObject, NSTextViewDelegate {
        var parent: SearchInput
        var lastCommittedText: String
        var focusRequest = 0
        private var focusGeneration = 0
        var edit: (String, UInt32, UInt32, String, Bool)?
        var composition: (String, UInt32, UInt32, Bool)?
        init(_ parent: SearchInput) { self.parent = parent; lastCommittedText = parent.text }
        /// Deferred focus is valid only until another focus request or native edit begins.
        func cancelFocus() { focusGeneration += 1 }
        func requestFocus(_ view: SearchTextView, request: Int, initial: Bool = false) {
            focusRequest = request; focusGeneration += 1
            let generation = focusGeneration
            DispatchQueue.main.async { [weak self, weak view] in
                guard let self, let view, generation == self.focusGeneration,
                      request == self.parent.focusRequest, !view.isComposing, !view.hasActiveMarkedText,
                      view.nativeMutationDepth == 0, view.isEditable, let window = view.window else { return }
                if initial, window.firstResponder is NSTextView { return }
                if self.parent.caretAtEndOnFocus { window.makeKey() }
                window.makeFirstResponder(view)
                if self.parent.caretAtEndOnFocus {
                    let caret = NSRange(location: (view.string as NSString).length, length: 0)
                    view.setSelectedRange(caret); view.scrollRangeToVisible(caret)
                }
            }
        }
        func beginComposition(_ old: String, range: NSRange) {
            cancelFocus()
            guard composition == nil, let selected = Range(range, in: old) else { return }
            let start = UInt32(old[..<selected.lowerBound].utf8.count), end = UInt32(old[..<selected.upperBound].utf8.count)
            composition = (old, start, end, !old.isEmpty && range.location == 0 && range.length == (old as NSString).length)
            parent.compositionChanged(true)
        }
        func textView(_ textView: NSTextView, shouldChangeTextIn affectedCharRange: NSRange, replacementString: String?) -> Bool {
            cancelFocus()
            guard let replacementString, let range = Range(affectedCharRange, in: textView.string) else { return true }
            let old = textView.string
            let start = UInt32(old[..<range.lowerBound].utf8.count)
            let end = UInt32(old[..<range.upperBound].utf8.count)
            let full = !old.isEmpty && affectedCharRange.location == 0 && affectedCharRange.length == (old as NSString).length
            edit = (old, start, end, replacementString, full)
            return true
        }
        func textDidChange(_ notification: Notification) {
            guard let view = notification.object as? SearchTextView, view.nativeMutationDepth == 0 else { return }
            commit(view)
        }
        func commit(_ view: SearchTextView) {
            guard !view.isComposing, !view.hasActiveMarkedText, view.nativeMutationDepth == 0 else { return }
            guard edit != nil || composition != nil || view.pendingTrigger != nil || view.string != lastCommittedText else { return }
            lastCommittedText = view.string
            if let trigger = view.pendingTrigger {
                view.pendingTrigger = nil
                if parent.triggerInserted?(trigger.symbol, trigger.before, view.string) == true { edit = nil; return }
            }
            if let composition {
                if view.string == composition.0 {
                    self.composition = nil; edit = nil; parent.compositionChanged(false); return
                }
                let before = Array(composition.0.utf8), after = Array(view.string.utf8)
                let prefix = Array(before.prefix(Int(composition.1))), suffix = Array(before.dropFirst(Int(composition.2)))
                if after.starts(with: prefix), after.suffix(suffix.count).elementsEqual(suffix), after.count >= prefix.count + suffix.count {
                    let replacement = String(decoding: after[prefix.count..<(after.count - suffix.count)], as: UTF8.self)
                    parent.changed(view.string, composition.1, composition.2, replacement, composition.3)
                } else { parent.changed(view.string, 0, UInt32(before.count), view.string, true) }
                self.composition = nil
            } else if let edit { parent.changed(view.string, edit.1, edit.2, edit.3, edit.4) }
            else { parent.changed(view.string, 0, UInt32(parent.text.utf8.count), view.string, true) }
            edit = nil
        }
        func textView(_ textView: NSTextView, doCommandBy commandSelector: Selector) -> Bool {
            if let view = textView as? SearchTextView, view.keyWasComposing {
                if view.isComposing || view.hasActiveMarkedText { return false }
                return commandSelector == #selector(NSResponder.insertNewline(_:)) || commandSelector == #selector(NSResponder.cancelOperation(_:))
            }
            guard !((textView as? SearchTextView)?.hasActiveMarkedText ?? textView.hasMarkedText()), (textView as? SearchTextView)?.isComposing != true else { return false }
            if commandSelector == #selector(NSResponder.deleteBackward(_:)),
               textView.selectedRange().location == 0, textView.selectedRange().length == 0,
               let delete = parent.deleteBackwardAtStart { return delete() }
            if commandSelector == #selector(NSResponder.moveDown(_:)), let move = parent.move { move(1); return true }
            if commandSelector == #selector(NSResponder.moveUp(_:)), let move = parent.move { move(-1); return true }
            if commandSelector == #selector(NSResponder.insertNewline(_:)) { return true }
            return false
        }
    }
}
/// Plain single-line editor that distinguishes paste from physical insertion.
final class SearchTextView: NSTextView {
    var placeholder = ""
    var acceptsTrigger = false
    private(set) var nativeMutationDepth = 0
    /// A zero-length marked range contains no pending IME text and must not trap navigation.
    var hasActiveMarkedText: Bool {
        let range = markedRange()
        return range.location != NSNotFound && range.length > 0
    }
    private(set) var isComposing = false
    private(set) var keyWasComposing = false
    private var physicalInsertion = false
    var pendingTrigger: (symbol: String, before: String)?
    var nativeCommit: (() -> Void)?
    var compositionBegan: ((String, NSRange) -> Void)?

    override func keyDown(with event: NSEvent) {
        keyWasComposing = isComposing || hasActiveMarkedText
        physicalInsertion = event.modifierFlags.intersection([.command, .control, .option]).isEmpty
        defer { keyWasComposing = false; physicalInsertion = false }
        super.keyDown(with: event)
    }
    override func setMarkedText(_ string: Any, selectedRange: NSRange, replacementRange: NSRange) {
        if !isComposing {
            let range = replacementRange.location == NSNotFound ? self.selectedRange() : replacementRange
            isComposing = true
            compositionBegan?(self.string, range)
        }
        nativeMutationDepth += 1
        super.setMarkedText(string, selectedRange: selectedRange, replacementRange: replacementRange)
        nativeMutationDepth -= 1
        // Some input methods finish through an empty marked-text update instead
        // of insertText/unmarkText. AppKit owns whether composition still exists.
        isComposing = hasActiveMarkedText
        if !isComposing, nativeMutationDepth == 0 { nativeCommit?() }
    }
    override func insertText(_ inserted: Any, replacementRange: NSRange) {
        let committingComposition = isComposing || hasActiveMarkedText || keyWasComposing
        let value = (inserted as? NSAttributedString)?.string ?? inserted as? String ?? ""
        let range = replacementRange.location == NSNotFound ? selectedRange() : replacementRange
        if acceptsTrigger, physicalInsertion, !committingComposition,
           ["@", "$", "#"].contains(value), range.length == 0, range.location == (string as NSString).length,
           string.isEmpty || string.last?.isWhitespace == true {
            var quoted = false, escaped = false, math = false
            for character in string {
                if escaped { escaped = false; continue }
                if character == "\\" { escaped = true } else if character == "\"" { quoted.toggle() } else if character == "$", !quoted { math.toggle() }
            }
            if !quoted && !escaped && !math && !string.trimmingCharacters(in: .whitespacesAndNewlines).hasSuffix(":") { pendingTrigger = (value, string) }
        }
        nativeMutationDepth += 1
        super.insertText(inserted, replacementRange: replacementRange)
        nativeMutationDepth -= 1
        isComposing = hasActiveMarkedText
        if nativeMutationDepth == 0 { nativeCommit?() }
    }
    override func unmarkText() {
        super.unmarkText()
        isComposing = hasActiveMarkedText
        if nativeMutationDepth == 0 { nativeCommit?() }
    }
    override func draw(_ dirtyRect: NSRect) {
        super.draw(dirtyRect)
        if string.isEmpty, bounds.width > 1 {
            NSGraphicsContext.saveGraphicsState()
            defer { NSGraphicsContext.restoreGraphicsState() }
            NSBezierPath(rect: bounds).addClip()
            let paragraph = NSMutableParagraphStyle()
            paragraph.lineBreakMode = .byTruncatingTail
            (placeholder as NSString).draw(in: bounds, withAttributes: [.font: font ?? NSFont.systemFont(ofSize: 13), .foregroundColor: NSColor.placeholderTextColor, .paragraphStyle: paragraph])
        }
    }
}

/// Maps service byte offsets onto original Swift text, rejecting non-scalar boundaries.
/// Repeated author entries retain separate offsets before joining for presentation.
enum SearchHighlight {
    /// Adds the renderer's math delimiters while mapping original chunk ranges;
    /// any overlap with math marks that entire equation as an atomic unit.
    static func latexProjection(_ chunks: [TextChunk], ranges: [Range<Int>]) -> (text: String, ranges: [Range<Int>]) {
        var text = "", mapped: [Range<Int>] = [], rawOffset = 0
        for chunk in chunks {
            let count = chunk.text.utf8.count
            let raw = rawOffset..<(rawOffset + count)
            let target = text.utf8.count
            let overlaps = ranges.compactMap { range -> Range<Int>? in
                let lower = max(raw.lowerBound, range.lowerBound), upper = min(raw.upperBound, range.upperBound)
                return lower < upper ? lower..<upper : nil
            }
            if chunk.kind == .math {
                text += "$" + chunk.text + "$"
                if !overlaps.isEmpty { mapped.append(target..<(target + count + 2)) }
            } else {
                text += chunk.text
                mapped += overlaps.map { (target + $0.lowerBound - rawOffset)..<(target + $0.upperBound - rawOffset) }
            }
            rawOffset += count
        }
        return (text, mapped)
    }

    static func attributed(_ text: String, ranges: [Range<Int>]) -> AttributedString {
        var result = AttributedString(text)
        for range in ranges where range.lowerBound >= 0 && range.upperBound <= text.utf8.count && !range.isEmpty {
            let a = text.utf8.index(text.utf8.startIndex, offsetBy: range.lowerBound)
            let b = text.utf8.index(text.utf8.startIndex, offsetBy: range.upperBound)
            guard let start = String.Index(a, within: text), let end = String.Index(b, within: text),
                  let lower = AttributedString.Index(start, within: result), let upper = AttributedString.Index(end, within: result) else { continue }
            result[lower..<upper].backgroundColor = Color.accentColor.opacity(0.18)
            result[lower..<upper].inlinePresentationIntent = .stronglyEmphasized
        }
        return result
    }
    static func ranges(_ matches: [SearchMatchRange], field: String, entry: UInt32 = 0) -> [Range<Int>] {
        matches.filter { $0.fieldName == field && $0.entryIndex == entry && $0.start < $0.end }.map { Int($0.start)..<Int($0.end) }
    }
    static func authors(_ names: [String], matches: [SearchMatchRange]) -> AttributedString {
        var result = AttributedString()
        for (index, name) in names.enumerated() {
            if index > 0 { result.append(AttributedString(", ")) }
            result.append(attributed(name, ranges: ranges(matches, field: "author", entry: UInt32(index))))
        }
        return result
    }
}

/// Reads only a stored property-list Boolean; malformed or numeric preferences default to ordinary search.
func advancedSearchPreference(_ preferences: UserDefaults, key: String) -> Bool {
    guard let number = preferences.object(forKey: key) as? NSNumber,
          CFGetTypeID(number) == CFBooleanGetTypeID() else { return false }
    return number.boolValue
}
