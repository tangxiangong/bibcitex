import SwiftUI
import BibCiTeXCore

struct TagFilterButton: View {
    let filter: TagFilter
    let apply: (TagFilter) -> Void
    @State private var presented = false

    var body: some View {
        trigger
            .help(L10n.text("标签筛选"))
            .accessibilityValue(filter.isEmpty ? L10n.text("未启用") : L10n.text("已启用"))
            .popover(isPresented: $presented, arrowEdge: .bottom) {
                TagFilterEditor(filter: filter, apply: apply)
            }
    }
    @ViewBuilder private var trigger: some View {
        if #available(macOS 26.0, *) {
            if filter.isEmpty { button.buttonStyle(.glass) }
            else { button.buttonStyle(.glassProminent) }
        } else {
            button.buttonStyle(.bordered).tint(filter.isEmpty ? nil : .accentColor)
        }
    }
    private var button: some View {
        Button { presented = true } label: {
            Label(L10n.text("标签筛选"), systemImage: "line.3.horizontal.decrease")
        }.controlSize(.small)
    }
}

private struct TagFilterEditor: View {
    let apply: (TagFilter) -> Void
    @Environment(\.dismiss) private var dismiss
    @State private var draft: TagFilter
    @State private var target: UUID
    @State private var replacing: UUID?
    @State private var tags: [TagRecord] = []
    @State private var search = ""
    @State private var loading = true
    @State private var error: String?
    @State private var loadVersion = 0

    init(filter: TagFilter, apply: @escaping (TagFilter) -> Void) {
        self.apply = apply
        _draft = State(initialValue: filter)
        _target = State(initialValue: filter.id)
    }
    private var names: [String: String] { Dictionary(uniqueKeysWithValues: tags.map { ($0.id, $0.name) }) }
    private var missing: Bool { !loading && error == nil && !draft.tagIDs.isSubset(of: Set(tags.map(\.id))) }
    private var valid: Bool { draft.isEmpty || (draft.isComplete && !missing) }

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(L10n.text("标签筛选")).font(.headline)
            TagFilterScroll(maxHeight: 440) {
                VStack(alignment: .leading, spacing: 10) {
                    TagFilterScroll(maxHeight: 270, axes: [.horizontal, .vertical]) {
                        group(draft).frame(minWidth: 448, alignment: .leading)
                    }
                    if !draft.isEmpty {
                        Text(draft.expression(names: names))
                            .font(.system(.caption, design: .monospaced)).foregroundStyle(.secondary)
                            .lineLimit(3).textSelection(.enabled)
                    }
                    if !valid {
                        Text(L10n.text(missing ? "筛选标签已删除，请替换或移除" : "请为每个分组添加条件"))
                            .font(.caption).foregroundStyle(.red)
                    }
                    Divider()
                    Text(L10n.text(replacing == nil ? "添加到当前分组" : "替换标签"))
                        .font(.caption.weight(.medium))
                    TextField(L10n.text("搜索标签"), text: $search).textFieldStyle(.roundedBorder)
                    if loading { ProgressView().controlSize(.small) }
                    if let error {
                        HStack {
                            Text(error).font(.caption).foregroundStyle(.red)
                            Button(L10n.text("重试")) { Task { await load() } }
                        }
                    }
                    VStack(alignment: .leading, spacing: 6) {
                        DetailTagLayout {
                            ForEach(tags.filter { search.isEmpty || $0.name.localizedCaseInsensitiveContains(search) }, id: \.id) { tag in
                                Button { choose(tag) } label: { chip(tag.id) }
                                    .buttonStyle(.plain).accessibilityLabel(tag.name)
                            }
                        }.frame(maxWidth: .infinity, alignment: .leading)
                        if !loading && error == nil && tags.filter({ search.isEmpty || $0.name.localizedCaseInsensitiveContains(search) }).isEmpty {
                            Text(L10n.text("无匹配标签")).font(.caption).foregroundStyle(.secondary)
                        }
                    }
                }.frame(maxWidth: .infinity, alignment: .leading)
            }
            Divider()
            HStack {
                Button(L10n.text("清空")) {
                    draft = TagFilter(); target = draft.id; replacing = nil
                }
                Spacer()
                Button(L10n.text("取消")) { dismiss() }.keyboardShortcut(.cancelAction)
                applyButton.disabled(!valid || (!draft.isEmpty && (loading || error != nil)))
            }
        }
        .padding(14).frame(width: 480).fixedSize(horizontal: false, vertical: true).controlSize(.small)
        // Native popover chrome supplies Liquid Glass; no opaque fill or second glass layer.
        .task { await load() }
        .onReceive(NotificationCenter.default.publisher(for: .searchTagsChanged)) { _ in
            Task { await load() }
        }
    }

    @ViewBuilder private var applyButton: some View {
        if #available(macOS 26.0, *) { commitButton.buttonStyle(.glassProminent) }
        else { commitButton.buttonStyle(.borderedProminent) }
    }
    private var commitButton: some View {
        Button(L10n.text("应用")) {
            apply(draft.isEmpty ? TagFilter() : draft); dismiss()
        }.keyboardShortcut(.defaultAction)
    }
    private func toggle(_ node: TagFilter) -> some View {
        Toggle("NOT", isOn: Binding(get: { draft.node(node.id)?.negated ?? false }, set: { value in
            draft.update(node.id) { $0.negated = value }
        })).toggleStyle(.checkbox).fixedSize()
            .help(L10n.text(node.tagID == nil ? "整组取反" : "排除"))
            .accessibilityLabel(node.tagID.flatMap { names[$0] }.map { L10n.text("排除") + " " + $0 } ?? L10n.text("整组取反"))
    }
    private func group(_ node: TagFilter) -> AnyView {
        AnyView(VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 8) {
                Picker(L10n.text("组合"), selection: Binding(get: { draft.node(node.id)?.usesOr ?? false }, set: { value in
                    draft.update(node.id) { $0.usesOr = value }
                })) {
                    Text("AND").tag(false)
                    Text("OR").tag(true)
                }.pickerStyle(.segmented).labelsHidden().frame(width: 100)
                toggle(node)
                Spacer(minLength: 6)
                Button(L10n.text("＋条件")) { target = node.id; replacing = nil }
                Button(L10n.text("＋分组")) {
                    let child = TagFilter()
                    draft.update(node.id) { $0.children.append(child) }
                    target = child.id; replacing = nil
                }
                if node.id != draft.id { removeButton(node.id) }
            }
            ForEach(node.children) { child in
                if let tagID = child.tagID {
                    HStack {
                        Button {
                            target = node.id; replacing = child.id
                        } label: { chip(tagID) }
                        .buttonStyle(.plain)
                        .overlay(Capsule().stroke(replacing == child.id ? Color.accentColor : .clear))
                        Spacer()
                        toggle(child)
                        removeButton(child.id)
                    }.padding(.horizontal, 4)
                } else { group(child).padding(.leading, 12) }
            }
        }.padding(8)
            .overlay(RoundedRectangle(cornerRadius: 8)
                .stroke(target == node.id ? Color.accentColor.opacity(0.6) : Color.secondary.opacity(0.25)))
        )
    }
    private func removeButton(_ id: UUID) -> some View {
        Button {
            draft.remove(id)
            if draft.node(target) == nil { target = draft.id }
            if let replacing, draft.node(replacing) == nil { self.replacing = nil }
        } label: { Image(systemName: "minus.circle") }
        .buttonStyle(.borderless).help(L10n.text("移除")).accessibilityLabel(L10n.text("移除"))
    }
    private func chip(_ id: String) -> some View {
        let tag = tags.first { $0.id == id }
        let color = tag?.displayColor ?? .secondary
        return HStack(spacing: 4) {
            Circle().fill(color).frame(width: 6, height: 6)
            Text(tag?.name ?? (loading ? "…" : L10n.text("已删除标签"))).lineLimit(1)
        }.font(.caption).padding(.horizontal, 8).padding(.vertical, 4)
            .background(color.opacity(0.14), in: Capsule())
            .foregroundStyle(Color.primary)
    }
    private func choose(_ tag: TagRecord) {
        if let replacing {
            draft.update(replacing) { $0.tagID = tag.id }
            self.replacing = nil
        } else {
            draft.update(target) { $0.children.append(TagFilter(tagID: tag.id)) }
        }
    }
    private func load() async {
        loadVersion += 1
        let version = loadVersion
        loading = true; error = nil
        do {
            let snapshot = try await RustCore.shared.tagSnapshot(path: "", citeKey: "")
            guard version == loadVersion, !Task.isCancelled else { return }
            let catalogIDs = Set(snapshot.tags.map(\.id))
            var redirects: [String: String] = [:]
            // A missing catalog ID can be a merge alias, not a deleted condition.
            for id in draft.tagIDs.subtracting(catalogIDs) {
                let refreshed = try await RustCore.shared.refreshQuery(TagFilter(tagID: id).document)
                guard version == loadVersion, !Task.isCancelled else { return }
                if refreshed.status == .valid,
                   let resolved = refreshed.document?.nodes.compactMap(\.tagId).first,
                   catalogIDs.contains(resolved) {
                    redirects[id] = resolved
                }
            }
            tags = snapshot.tags
            for (oldID, newID) in redirects { draft.replaceTagID(oldID, with: newID) }
        } catch {
            guard version == loadVersion, !Task.isCancelled else { return }
            self.error = LocalizedMessage(error: error).text
        }
        loading = false
    }
}

/// Measure natural content height; only overflow needs a scroll viewport.
private struct TagFilterScroll<Content: View>: View {
    let maxHeight: CGFloat
    let axes: Axis.Set
    let content: Content
    @State private var contentHeight: CGFloat = 1

    init(maxHeight: CGFloat, axes: Axis.Set = .vertical, @ViewBuilder content: () -> Content) {
        self.maxHeight = maxHeight
        self.axes = axes
        self.content = content()
    }

    var body: some View {
        ScrollView(axes) {
            content.fixedSize(horizontal: false, vertical: true)
                .background(GeometryReader { geometry in
                    Color.clear
                        .onAppear { updateHeight(geometry.size.height) }
                        .onChange(of: geometry.size.height) { updateHeight($0) }
                })
        }
        .frame(height: min(maxHeight, max(1, contentHeight)), alignment: .topLeading)
    }

    private func updateHeight(_ height: CGFloat) {
        let measured = max(1, ceil(height))
        if contentHeight != measured { contentHeight = measured }
    }
}
