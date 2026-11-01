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
            }.onChange(of: field) { _ in matchStyle = "all"; value = "" }
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
                Toggle(L10n.text("排除"), isOn: $exclude)
                Picker(L10n.text("组合"), selection: $combineOr) { Text("AND").tag(false); Text("OR").tag(true) }.frame(width: 130)
            }
            HStack {
                Button(L10n.text("添加条件")) { addCondition() }
                    .disabled(busy || (matchMode == "tag" ? tagID.isEmpty : value.isEmpty))
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
                phrase: supportsPhrase && matchStyle == "phrase", exclude: exclude && matchStyle != "any", tagId: matchMode == "tag" ? selectedTag?.id : nil))
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

/// Stable UUIDs captured before a result menu outlives its row.
struct TagAssignmentTarget: Identifiable {
    let libraryUID: String
    let recordUID: String
    let citeKey: String
    var id: String { recordUID }
}

/// Each result keeps its presentation attached to its own tag area.
struct TagAssignmentPopover: ViewModifier {
    let reference: Reference
    @Binding var target: TagAssignmentTarget?

    private var matchesTarget: Bool {
        target?.libraryUID == reference.libraryUID && target?.recordUID == reference.id
    }

    func body(content: Content) -> some View {
        content.popover(item: Binding(
            get: { matchesTarget ? target : nil },
            set: { value in if matchesTarget { target = value } }
        ), attachmentAnchor: .rect(.bounds), arrowEdge: .top) { reference in
            TagManagerView(citeKey: reference.citeKey, libraryUID: reference.libraryUID, recordUID: reference.recordUID)
        }
    }
}

extension TagRecord {
    var displayColor: Color {
        let value = UInt32(color.dropFirst(), radix: 16) ?? 0x64748B
        return Color(red: Double((value >> 16) & 255) / 255, green: Double((value >> 8) & 255) / 255, blue: Double(value & 255) / 255)
    }
}

/// One bounded line of readable tags, including an overflow menu.
struct TagChips: View {
    let tags: [TagRecord]
    var edit: (() -> Void)? = nil
    var body: some View {
        if !tags.isEmpty {
            HStack(spacing: 4) {
                ForEach(Array(tags.prefix(2)), id: \.id) { tag in
                    if let edit { Button(action: edit) { chip(tag) }.buttonStyle(.plain) }
                    else { chip(tag) }
                }
                if tags.count > 2 {
                    if let edit {
                        Menu("+\(tags.count - 2)") {
                            ForEach(tags, id: \.id) { tag in Button(tag.name, action: edit) }
                        }.menuStyle(.borderlessButton).fixedSize()
                    } else {
                        Text("+\(tags.count - 2)").fixedSize()
                            .help(tags.map(\.name).joined(separator: ", "))
                    }
                }
            }.font(.system(size: 10)).frame(maxWidth: 160, alignment: .trailing)
        }
    }
    private func chip(_ tag: TagRecord) -> some View {
        HStack(spacing: 4) {
            Circle().fill(tag.displayColor).frame(width: 6, height: 6)
            Text(tag.name).lineLimit(1).truncationMode(.tail)
        }.padding(.horizontal, 6).padding(.vertical, 2)
            .background(tag.displayColor.opacity(0.14), in: Capsule())
            .foregroundStyle(Color.primary).help(tag.name)
    }
}

/// The detail pane shows every assigned tag and keeps assignment one click away.
struct DetailTagChips: View {
    let tags: [TagRecord]
    let edit: () -> TagAssignmentTarget?
    @State private var taggingReference: TagAssignmentTarget?

    var body: some View {
        DetailTagLayout {
            ForEach(tags, id: \.id) { tag in
                HStack(spacing: 5) {
                    Circle().fill(tag.displayColor).frame(width: 6, height: 6)
                        .accessibilityHidden(true)
                    Text(tag.name).lineLimit(1).truncationMode(.tail)
                }
                .modifier(DetailTagAppearance(background: tag.displayColor.opacity(0.14)))
                .help(tag.name).accessibilityLabel(tag.name)
            }
            Button { taggingReference = edit() } label: { NativeIcon("add", size: 12) }
                .help(L10n.text("添加")).accessibilityLabel(L10n.text("添加"))
                .popover(item: $taggingReference, attachmentAnchor: .rect(.bounds), arrowEdge: .top) { reference in
                    TagManagerView(citeKey: reference.citeKey, libraryUID: reference.libraryUID, recordUID: reference.recordUID)
                }
        }
        .buttonStyle(DetailTagButtonStyle())
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

private struct DetailTagButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .modifier(DetailTagAppearance())
            .contentShape(Capsule())
            .opacity(configuration.isPressed ? 0.7 : 1)
    }
}

private struct DetailTagAppearance: ViewModifier {
    var background = PanelPalette.controlSurface

    func body(content: Content) -> some View {
        content
            .font(.caption)
            .foregroundStyle(Color.primary)
            .padding(.horizontal, 8)
            .frame(height: 24)
            .background(background, in: Capsule())
    }
}

struct DetailTagLayout: Layout {
    private let spacing: CGFloat = 6

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        arrangement(width: proposal.width ?? .infinity, subviews: subviews).size
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let layout = arrangement(width: bounds.width, subviews: subviews)
        for (view, frame) in zip(subviews, layout.frames) {
            view.place(at: CGPoint(x: bounds.minX + frame.minX, y: bounds.minY + frame.minY),
                       anchor: .topLeading, proposal: ProposedViewSize(frame.size))
        }
    }

    private func arrangement(width: CGFloat, subviews: Subviews) -> (size: CGSize, frames: [CGRect]) {
        let limit = max(0, width)
        var frames: [CGRect] = []
        var x: CGFloat = 0
        var y: CGFloat = 0
        var rowHeight: CGFloat = 0
        var contentWidth: CGFloat = 0
        var rowStart = 0
        func centerRow() {
            for index in rowStart..<frames.count {
                frames[index].origin.y += (rowHeight - frames[index].height) / 2
            }
        }
        for view in subviews {
            let size = view.sizeThatFits(ProposedViewSize(width: limit.isFinite ? limit : nil, height: nil))
            if x > 0 && x + size.width > limit {
                centerRow()
                rowStart = frames.count
                x = 0
                y += rowHeight + spacing
                rowHeight = 0
            }
            frames.append(CGRect(origin: CGPoint(x: x, y: y), size: size))
            contentWidth = max(contentWidth, x + size.width)
            x += size.width + spacing
            rowHeight = max(rowHeight, size.height)
        }
        centerRow()
        return (CGSize(width: limit.isFinite ? limit : contentWidth, height: y + rowHeight), frames)
    }
}

private struct TagColorEditor: View {
    let tag: TagRecord
    let save: (String) -> Void
    @State private var color: Color
    init(tag: TagRecord, save: @escaping (String) -> Void) {
        self.tag = tag; self.save = save; _color = State(initialValue: tag.displayColor)
    }
    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 6) {
                ForEach(["#64748B", "#EF4444", "#F59E0B", "#22C55E", "#14B8A6", "#3B82F6", "#8B5CF6", "#EC4899"], id: \.self) { value in
                    Button { save(value) } label: {
                        Circle().fill(TagRecord(color: value, id: "", name: "").displayColor)
                            .overlay(Circle().stroke(Color.primary.opacity(0.25))).frame(width: 20, height: 20)
                    }.buttonStyle(.plain).accessibilityLabel(value)
                }
            }
            ColorPicker(L10n.text("自定义颜色"), selection: $color, supportsOpacity: false)
            Button(L10n.text("保存")) {
                guard let rgb = NSColor(color).usingColorSpace(.sRGB) else { return }
                save(String(format: "#%02X%02X%02X", Int((rgb.redComponent * 255).rounded()), Int((rgb.greenComponent * 255).rounded()), Int((rgb.blueComponent * 255).rounded())))
            }.keyboardShortcut(.defaultAction)
        }.padding(12).frame(width: 240)
    }
}

/// Compact assignment picker; catalog maintenance is revealed only when requested.
struct TagManagerView: View {
    var path: String = ""
    var citeKey: String = ""
    var libraryUID: String = ""
    var recordUID: String = ""
    @Environment(\.dismiss) private var dismiss
    @State private var snapshot: TagSnapshot?
    @State private var filter = ""
    @State private var colorID = ""
    @State private var editingID = ""
    @State private var name = ""
    @State private var error: String?
    @State private var busy = false
    @State private var mergeTarget: (id: String, target: String, label: String)?
    @State private var deleteID: String?
    @State private var showRecovery = false
    private var assignment: Bool { !recordUID.isEmpty || !citeKey.isEmpty }
    private var filtered: [TagRecord] {
        (snapshot?.tags ?? []).filter { filter.isEmpty || $0.name.localizedCaseInsensitiveContains(filter) }
            .sorted { a, b in
                let aa = snapshot?.assignedIds.contains(a.id) ?? false
                let bb = snapshot?.assignedIds.contains(b.id) ?? false
                return aa != bb ? aa : a.name.localizedStandardCompare(b.name) == .orderedAscending
            }
    }
    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Text(L10n.text(assignment ? "标签" : "标签管理")).font(.headline)
                if assignment { Text(citeKey).font(.caption).foregroundStyle(.secondary).lineLimit(1) }
                Spacer()
                if busy { ProgressView().controlSize(.small) }
                Button(L10n.text("关闭")) { dismiss() }.keyboardShortcut(.cancelAction)
            }
            TextField(L10n.text("搜索或创建标签"), text: $filter).textFieldStyle(.roundedBorder)
                .onSubmit { if canCreate { mutate(.create, name: filter) } }
            ScrollView {
                LazyVStack(spacing: 4) {
                    ForEach(filtered, id: \.id) { tag in
                        HStack(spacing: 8) {
                            if assignment {
                                Toggle(tag.name, isOn: Binding(get: { snapshot?.assignedIds.contains(tag.id) ?? false }, set: { mutate($0 ? .assign : .unassign, id: tag.id) }))
                                    .toggleStyle(.checkbox).labelsHidden().accessibilityLabel(tag.name)
                            }
                            Button { colorID = tag.id } label: {
                                Circle().fill(tag.displayColor).overlay(Circle().stroke(Color.primary.opacity(0.25))).frame(width: 14, height: 14)
                            }.buttonStyle(.plain).frame(width: 24, height: 24).accessibilityLabel(L10n.text("标签颜色") + " " + tag.name)
                                .popover(isPresented: Binding(get: { colorID == tag.id }, set: { if !$0 { colorID = "" } })) {
                                    TagColorEditor(tag: tag) { value in mutate(.recolor, id: tag.id, color: value); colorID = "" }
                                }
                            if editingID == tag.id {
                                TextField(L10n.text("标签名称"), text: $name).textFieldStyle(.roundedBorder)
                                    .onSubmit { mutate(.rename, id: tag.id, name: name) }
                                Button(L10n.text("保存")) { mutate(.rename, id: tag.id, name: name) }
                            } else {
                                Button { if assignment { mutate(snapshot?.assignedIds.contains(tag.id) == true ? .unassign : .assign, id: tag.id) } } label: {
                                    Text(tag.name).lineLimit(1).frame(maxWidth: .infinity, alignment: .leading)
                                }.buttonStyle(.plain)
                            }
                            Menu {
                                Button(L10n.text("重命名")) { editingID = tag.id; name = tag.name }
                                Button(L10n.text("标签颜色")) { colorID = tag.id }
                                if !assignment {
                                    Menu(L10n.text("合并到")) {
                                        ForEach((snapshot?.tags ?? []).filter { $0.id != tag.id }, id: \.id) { target in
                                            Button(target.name) { mergeTarget = (tag.id, target.id, tag.name + " → " + target.name) }
                                        }
                                    }
                                    Button(L10n.text("删除"), role: .destructive) { deleteID = tag.id }
                                }
                            } label: { Image(systemName: "ellipsis") }.menuStyle(.borderlessButton).menuIndicator(.hidden).frame(width: 28, height: 24)
                                .accessibilityLabel(L10n.text("标签管理") + " " + tag.name)
                        }.padding(.vertical, 4)
                    }
                    if canCreate {
                        Button { mutate(.create, name: filter) } label: { Label(L10n.text(assignment ? "创建并添加" : "创建") + " “" + filter.trimmingCharacters(in: .whitespacesAndNewlines) + "”", systemImage: "plus") }
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            }
            if let error { Text(error).font(.caption).foregroundStyle(.red) }
            if !assignment, let snapshot, !snapshot.pending.isEmpty {
                Button(L10n.text("失效标签关联")) { showRecovery = true }
                    .sheet(isPresented: $showRecovery) { TagRecoveryView() }
            }
        }.padding(14).frame(width: assignment ? 360 : 460, height: assignment ? 340 : 440)
            .disabled(busy).task { await reload() }
            .onChange(of: showRecovery) { shown in if !shown { Task { await reload() } } }
            .alert(L10n.text("合并到"), isPresented: Binding(get: { mergeTarget != nil }, set: { if !$0 { mergeTarget = nil } })) {
                Button(L10n.text("确认")) { if let merge = mergeTarget { mutate(.merge, id: merge.id, target: merge.target) }; mergeTarget = nil }
                Button(L10n.text("取消"), role: .cancel) { mergeTarget = nil }
            } message: { Text(mergeTarget?.label ?? "") }
            .alert(L10n.text("删除标签"), isPresented: Binding(get: { deleteID != nil }, set: { if !$0 { deleteID = nil } })) {
                Button(L10n.text("删除"), role: .destructive) { if let id = deleteID { mutate(.delete, id: id) }; deleteID = nil }
                Button(L10n.text("取消"), role: .cancel) { deleteID = nil }
            }
    }
    private var canCreate: Bool {
        let value = filter.trimmingCharacters(in: .whitespacesAndNewlines)
        return !value.isEmpty && !(snapshot?.tags.contains { $0.name.compare(value, options: [.caseInsensitive]) == .orderedSame } ?? false)
    }
    private func reload() async {
        do {
            if !libraryUID.isEmpty { snapshot = try await RustCore.shared.tagSnapshot(libraryUID: libraryUID, recordUID: recordUID) }
            else { snapshot = try await RustCore.shared.tagSnapshot(path: path, citeKey: citeKey) }
        } catch { self.error = error.localizedDescription }
    }
    private func mutate(_ action: TagAction, id: String = "", name: String = "", color: String? = nil, target: String = "") {
        guard !busy, let state = snapshot else { return }
        busy = true
        Task {
            defer { busy = false }
            do {
                snapshot = try await RustCore.shared.mutateTag(TagMutation(libraryUid: state.libraryUid, recordUid: state.recordUid, color: color, action: action, id: id, name: name, targetId: target, path: "", citeKey: "", expectedRevision: state.revision))
                if action == .create { filter = "" }
                if action == .rename { editingID = "" }
                error = nil
                NotificationCenter.default.post(name: .searchTagsChanged, object: nil)
            } catch { self.error = error.localizedDescription; await reload() }
        }
    }
}

/// Exceptional record recovery is separate from everyday tag assignment.
struct TagRecoveryView: View {
    @Environment(\.dismiss) private var dismiss
    @State private var snapshot: TagSnapshot?
    @State private var pendingID = ""
    @State private var libraryUID = ""
    @State private var recordUID = ""
    @State private var libraries: [Bibliography] = []
    @State private var targets: [Reference] = []
    @State private var error: String?
    @State private var busy = false
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(L10n.text("失效标签关联")).font(.headline)
            Picker(L10n.text("旧记录"), selection: $pendingID) {
                Text(L10n.text("选择记录")).tag("")
                ForEach(snapshot?.pending ?? [], id: \.recordId) { Text($0.citeKey).tag($0.recordId) }
            }
            Picker(L10n.text("目标文献库"), selection: $libraryUID) {
                Text(L10n.text("选择文献库")).tag("")
                ForEach(libraries, id: \.name) { Text($0.name).tag($0.uid) }
            }.onChange(of: libraryUID) { uid in
                recordUID = ""; targets = []
                guard let path = libraries.first(where: { $0.uid == uid })?.path else { return }
                Task { do { let rows = try await RustCore.shared.search(path: path, query: ""); if libraryUID == uid { targets = rows } } catch { self.error = error.localizedDescription } }
            }
            Picker(L10n.text("目标记录"), selection: $recordUID) {
                Text(L10n.text("选择记录")).tag("")
                ForEach(targets) { Text($0.citeKey).tag($0.id) }
            }
            if let error { Text(error).font(.caption).foregroundStyle(.red) }
            HStack {
                Button(L10n.text("重新关联")) { relink() }.disabled(pendingID.isEmpty || recordUID.isEmpty)
                Spacer(); Button(L10n.text("关闭")) { dismiss() }
            }
        }.padding(18).frame(width: 420).disabled(busy).task { await reload() }
    }
    private func reload() async {
        do { snapshot = try await RustCore.shared.tagSnapshot(path: "", citeKey: ""); var seen = Set<String>(); libraries = try await RustCore.shared.libraries().filter { $0.available && seen.insert($0.uid).inserted } }
        catch { self.error = error.localizedDescription }
    }
    private func relink() {
        busy = true
        Task {
            defer { busy = false }
            do {
                await reload()
                guard let revision = snapshot?.revision else { return }
                _ = try await RustCore.shared.mutateTag(TagMutation(libraryUid: libraryUID, recordUid: recordUID, color: nil, action: .relink, id: pendingID, name: "", targetId: "", path: "", citeKey: "", expectedRevision: revision))
                pendingID = ""
                await reload(); error = nil
                NotificationCenter.default.post(name: .searchTagsChanged, object: nil)
            } catch { self.error = error.localizedDescription }
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
    var focusChanged: (Bool) -> Void = { _ in }
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
        view.focusChanged = { [weak coordinator = context.coordinator] focused in coordinator?.parent.focusChanged(focused) }
        view.placeholder = placeholder
        view.acceptsTrigger = triggerInserted != nil
        // Only the main helper editor requests opportunistic initial focus.
        // Candidate and tray requests must reach updateNSView as explicit transfers.
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
        view.nativeCommit = nil; view.compositionBegan = nil; view.focusChanged = nil; view.delegate = nil
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
    var focusChanged: ((Bool) -> Void)?

    override func becomeFirstResponder() -> Bool {
        let accepted = super.becomeFirstResponder()
        if accepted { focusChanged?(true) }
        return accepted
    }
    override func resignFirstResponder() -> Bool {
        let accepted = super.resignFirstResponder()
        if accepted { focusChanged?(false) }
        return accepted
    }

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


/// Selection is local to the displayed result set. Editors capture immutable UUID pairs.
final class BatchTagSelection: ObservableObject {
    @Published var active = false { didSet { invalidateListSelection() } }
    @Published var ids = Set<String>() { didSet { invalidateListSelection() } }
    private var selectionRevision: UInt64 = 0
    private var pendingListSelection: Set<String>?
    var listIDs: Set<String> { pendingListSelection ?? ids }
    var hasPendingListSelection: Bool { pendingListSelection != nil }

    private func invalidateListSelection() {
        selectionRevision &+= 1
        pendingListSelection = nil
    }

    // AppKit can write List selection while SwiftUI is reconciling its rows.
    // Coalesce those writes and discard callbacks invalidated by another action.
    func deferListSelection(_ values: Set<String>, _ apply: @escaping () -> Void) {
        selectionRevision &+= 1
        // Native List must read its new selection during reconciliation, before
        // publishing it on the next turn. Returning the old singleton resets
        // Shift/Command selection before it can enter batch mode.
        pendingListSelection = values
        let revision = selectionRevision
        DispatchQueue.main.async { [weak self] in
            guard let self, self.selectionRevision == revision else { return }
            self.pendingListSelection = nil
            apply()
        }
    }

    @discardableResult func end(focusedID: String?) -> Bool {
        guard active || ids.count > 1 || listIDs.count > 1 else { return false }
        active = false
        ids = Set(focusedID.map { [$0] } ?? [])
        return true
    }

    func toggle(_ reference: Reference) {
        if !ids.insert(reference.id).inserted { ids.remove(reference.id) }
    }
}

/// Search fields and native Lists both route Escape through AppKit.
struct BatchTagEscapeHandler: NSViewRepresentable {
    @ObservedObject var selection: BatchTagSelection
    let focusedID: String?

    func makeCoordinator() -> Coordinator { Coordinator(self) }
    func makeNSView(context: Context) -> NSView {
        let view = NSView()
        context.coordinator.install(on: view)
        return view
    }
    func updateNSView(_ view: NSView, context: Context) { context.coordinator.parent = self }
    static func dismantleNSView(_ view: NSView, coordinator: Coordinator) { coordinator.removeMonitor() }

    final class Coordinator {
        var parent: BatchTagEscapeHandler
        private var monitor: Any?
        init(_ parent: BatchTagEscapeHandler) { self.parent = parent }
        func install(on view: NSView) {
            monitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self, weak view] event in
                guard let self, let window = view?.window,
                      window.isKeyWindow, event.window === window, window.attachedSheet == nil,
                      event.keyCode == 53,
                      event.modifierFlags.intersection([.command, .control, .option, .shift]).isEmpty
                else { return event }
                if let editor = window.firstResponder as? NSTextView {
                    // SwiftUI text fields use the field editor for inline rename;
                    // their Escape must cancel editing before ending batch mode.
                    if editor.isFieldEditor || editor.hasMarkedText() { return event }
                    if let search = editor as? SearchTextView,
                       search.isComposing || search.hasActiveMarkedText { return event }
                }
                return parent.selection.end(focusedID: parent.focusedID) ? nil : event
            }
        }
        func removeMonitor() {
            if let monitor { NSEvent.removeMonitor(monitor) }
            monitor = nil
        }
        deinit { removeMonitor() }
    }
}

struct BatchTagCheck: View {
    @ObservedObject var selection: BatchTagSelection
    let reference: Reference
    var body: some View {
        if selection.active {
            Toggle(reference.citeKey, isOn: Binding(get: { selection.ids.contains(reference.id) }, set: { value in
                if value { selection.ids.insert(reference.id) } else { selection.ids.remove(reference.id) }
            })).toggleStyle(.checkbox).labelsHidden().accessibilityLabel(reference.citeKey)
        }
    }
}

private struct BatchTagPresentation: Identifiable {
    let id = UUID()
    let targets: [TagTarget]

    init?(references: [Reference], selectedIDs: Set<String>) {
        let targets = references.filter { selectedIDs.contains($0.id) }
            .map { TagTarget(libraryUid: $0.libraryUID, recordUid: $0.id) }
        guard !targets.isEmpty else { return nil }
        self.targets = targets
    }
}

struct BatchTagControls: View {
    @ObservedObject var selection: BatchTagSelection
    let references: [Reference]
    var showsText = true
    var cite: (([Reference]) -> Void)?
    var copied: String?
    @State private var presentation: BatchTagPresentation?
    var body: some View {
        Button {
            selection.ids.removeAll()
            selection.active.toggle()
        } label: { controlLabel }
            .disabled(!selection.active && references.isEmpty)
            .fixedSize()
            .help(title).accessibilityLabel(title)
            .background(BatchTagActionPopover(isPresented: selection.active && presentation == nil, content: AnyView(actions)))
            .sheet(item: $presentation) { item in BatchTagEditor(targets: item.targets) }
            .onChange(of: references.map(\.id)) { _ in selection.ids.removeAll() }
    }
    private var actions: some View {
        HStack(spacing: 6) {
            action("全选", icon: "check") { selection.ids = Set(references.map(\.id)) }
            action("清空", icon: "trash") { selection.ids.removeAll() }.disabled(selection.ids.isEmpty)
            action("批量", icon: "tag") {
                presentation = BatchTagPresentation(references: references, selectedIDs: selection.ids)
            }.disabled(selection.ids.isEmpty)
            if let cite {
                let selected = references.filter { selection.ids.contains($0.id) }
                let didCopy = !selected.isEmpty && copied == selected.map(\.citeKey).joined(separator: ",")
                action("引用", icon: didCopy ? "check" : "copy") {
                    cite(references.filter { selection.ids.contains($0.id) })
                }.disabled(selected.isEmpty)
            }
            Divider().frame(height: 20)
            action("结束", icon: "x") { selection.active = false; selection.ids.removeAll() }
        }
        .buttonStyle(.borderless)
        .padding(10)
        .fixedSize()
        .onExitCommand { selection.active = false; selection.ids.removeAll() }
    }
    private func action(_ title: String, icon: String, perform: @escaping () -> Void) -> some View {
        Button(action: perform) {
            Label { Text(L10n.text(title)) } icon: { NativeIcon(icon, size: 14) }
                .padding(.horizontal, 6).padding(.vertical, 5)
                .contentShape(Rectangle())
        }.help(L10n.text(title))
    }
    private var title: String {
        selection.active ? "\(L10n.text("已选")) \(selection.ids.count)" : L10n.text("多选")
    }
    @ViewBuilder private var controlLabel: some View {
        if showsText || selection.active { Label(title, systemImage: "checklist").labelStyle(.titleAndIcon) }
        else { Label(title, systemImage: "checklist").labelStyle(.iconOnly) }
    }
}

/// Application-defined dismissal keeps the toolbar available while selecting rows.
private struct BatchTagActionPopover: NSViewRepresentable {
    let isPresented: Bool
    let content: AnyView

    func makeCoordinator() -> Coordinator { Coordinator(content: content) }
    func makeNSView(context: Context) -> AnchorView {
        let view = AnchorView()
        view.refresh = { [weak coordinator = context.coordinator] view in coordinator?.refresh(view) }
        context.coordinator.observe(view)
        return view
    }
    func updateNSView(_ view: AnchorView, context: Context) {
        context.coordinator.isPresented = isPresented
        context.coordinator.host.rootView = content
        view.scheduleRefresh()
    }
    static func dismantleNSView(_ view: AnchorView, coordinator: Coordinator) {
        view.refresh = nil
        coordinator.close()
    }

    final class AnchorView: NSView {
        var refresh: ((AnchorView) -> Void)?
        private var refreshScheduled = false
        override func viewDidMoveToWindow() { super.viewDidMoveToWindow(); scheduleRefresh() }
        override func layout() { super.layout(); scheduleRefresh() }
        func scheduleRefresh() {
            guard !refreshScheduled else { return }
            refreshScheduled = true
            DispatchQueue.main.async { [weak self] in
                guard let self else { return }
                refreshScheduled = false
                refresh?(self)
            }
        }
    }

    final class Coordinator {
        let popover = NSPopover()
        let host: NSHostingController<AnyView>
        var isPresented = false
        private var observers: [NSObjectProtocol] = []
        init(content: AnyView) {
            host = NSHostingController(rootView: content)
            popover.behavior = .applicationDefined
            popover.animates = false
            popover.contentViewController = host
        }
        func observe(_ view: AnchorView) {
            for name in [NSApplication.didBecomeActiveNotification, NSApplication.didResignActiveNotification,
                         NSWindow.didMiniaturizeNotification, NSWindow.didDeminiaturizeNotification] {
                observers.append(NotificationCenter.default.addObserver(forName: name, object: nil, queue: .main) { [weak view] _ in
                    view?.scheduleRefresh()
                })
            }
        }
        func close() {
            observers.forEach { NotificationCenter.default.removeObserver($0) }
            observers.removeAll()
            popover.close()
        }
        deinit { observers.forEach { NotificationCenter.default.removeObserver($0) } }
        func refresh(_ view: NSView) {
            guard isPresented, NSApp.isActive, let window = view.window, window.isVisible, !window.isMiniaturized,
                  !view.isHiddenOrHasHiddenAncestor, window.attachedSheet == nil else {
                popover.close()
                return
            }
            host.view.layoutSubtreeIfNeeded()
            let size = host.view.fittingSize
            guard size.width > 0, size.height > 0, !view.bounds.isEmpty else { return }
            if popover.contentSize != size { popover.contentSize = size }
            if !popover.isShown { popover.show(relativeTo: view.bounds, of: view, preferredEdge: .minY) }
        }
    }
}

struct BatchTagEditor: View {
    let targets: [TagTarget]
    @Environment(\.dismiss) private var dismiss
    @State private var snapshot: BatchTagSnapshot?
    @State private var filter = ""
    @State private var error: String?
    @State private var busy = false
    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Text(L10n.text("批量标签")).font(.headline)
                Text("\(L10n.text("已选")) \(snapshot?.count ?? UInt64(targets.count))").foregroundStyle(.secondary)
                Spacer()
                Button(L10n.text("关闭")) { dismiss() }.keyboardShortcut(.cancelAction)
            }
            TextField(L10n.text("搜索或创建标签"), text: $filter).textFieldStyle(.roundedBorder)
            ScrollView {
                LazyVStack(spacing: 8) {
                    ForEach((snapshot?.tags ?? []).filter { filter.isEmpty || $0.name.localizedCaseInsensitiveContains(filter) }, id: \.id) { tag in
                        HStack {
                            Circle().fill(tag.displayColor).frame(width: 12, height: 12)
                            Text(tag.name).lineLimit(1)
                            Spacer()
                            Text(L10n.text(snapshot?.assignedIds.contains(tag.id) == true ? "全部已添加" : snapshot?.partialIds.contains(tag.id) == true ? "部分已添加" : "未添加")).font(.caption).foregroundStyle(.secondary)
                            Button(L10n.text("添加")) { mutate(.assign, id: tag.id) }.disabled(snapshot?.assignedIds.contains(tag.id) == true)
                            Button(L10n.text("移除")) { mutate(.unassign, id: tag.id) }.disabled(snapshot?.assignedIds.contains(tag.id) != true && snapshot?.partialIds.contains(tag.id) != true)
                        }
                    }
                    if !filter.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                        Button(L10n.text("创建并添加")) { mutate(.create, name: filter) }
                    }
                }
            }.disabled(snapshot == nil)
            if let error { Text(error).font(.caption).foregroundStyle(.red) }
            if busy { ProgressView().controlSize(.small) }
        }.padding(14).frame(width: 460, height: 340).disabled(busy).task { await reload() }
    }
    private func reload() async {
        do { snapshot = try await RustCore.shared.batchTagSnapshot(targets) }
        catch { snapshot = nil; self.error = error.localizedDescription }
    }
    private func mutate(_ action: TagAction, id: String = "", name: String = "") {
        guard !busy, let snapshot else { return }
        busy = true
        Task {
            defer { busy = false }
            do {
                self.snapshot = try await RustCore.shared.mutateBatchTag(BatchTagMutation(targets: targets, action: action, id: id, name: name, expectedRevision: snapshot.revision))
                error = nil
                if action == .create { filter = "" }
                NotificationCenter.default.post(name: .searchTagsChanged, object: nil)
            } catch { self.error = error.localizedDescription; await reload() }
        }
    }
}
