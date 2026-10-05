import SwiftUI
import AppKit
import UniformTypeIdentifiers

struct WorkbenchView: View {
    @ObservedObject var model: WorkbenchModel
    @FocusState private var searchFocused: Bool
    private let types = [("all", "全部类型"), ("Article", "期刊论文"), ("Book", "图书"), ("Thesis", "学位论文"), ("TechReport", "技术报告"), ("Misc", "其他"), ("Booklet", "小册子"), ("InBook", "书籍章节"), ("InCollection", "文集章节"), ("InProceedings", "会议论文")]
    private let fields = [("all", "全部字段"), ("author", "作者"), ("title", "标题"), ("journal", "期刊"), ("year", "年份")]

    var body: some View {
        HSplitView {
            if model.showSidebar { sidebar.frame(minWidth: 180, idealWidth: 220, maxWidth: 300) }
            references.frame(minWidth: 330, maxWidth: .infinity)
            if model.showInspector { inspector.frame(minWidth: 280, idealWidth: 350, maxWidth: 520) }
        }
        .background { WindowMaterial().ignoresSafeArea() }
        .frame(minWidth: 800, minHeight: 520)
        .toolbar {
            ToolbarItem(placement: .navigation) {
                Button { model.showSidebar.toggle() } label: { SVGIcon(model.showSidebar ? "panelLeftClose" : "panelLeftOpen") }.help("文献库").accessibilityLabel("文献库")
            }
            ToolbarItemGroup(placement: .primaryAction) {
                Button { model.adding = true } label: { SVGIcon("folderAdd") }.help("新增文献库").accessibilityLabel("新增文献库")
                Button { HelperPanelController.shared.showPanel() } label: { SVGIcon("search") }.help("快捷助手").accessibilityLabel("快捷助手")
                Button { Task { await model.reload() } } label: { SVGIcon("refresh") }.help("刷新").accessibilityLabel("刷新")
                Button { model.showInspector.toggle() } label: { SVGIcon(model.showInspector ? "panelRightClose" : "panelRightOpen") }.help("文献详情").accessibilityLabel("文献详情")
            }
        }
        .sheet(isPresented: $model.adding) { AddLibrarySheet(model: model) }
        .alert("错误", isPresented: Binding(get: { model.error != nil }, set: { if !$0 { model.error = nil } })) {
            Button("确定") { model.error = nil }
        } message: { Text(model.error ?? "") }
        .task { await model.reload() }
    }

    private var sidebar: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("文献库").font(.headline).padding(16)
            if model.libraries.isEmpty {
                Text("暂无文献库").foregroundStyle(.secondary).frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                List(selection: Binding(get: { model.selectedLibrary }, set: model.selectLibrary)) {
                    ForEach(model.libraries) { library in
                        HStack(spacing: 9) {
                            SVGIcon("library")
                            VStack(alignment: .leading, spacing: 4) {
                                Text(library.name).lineLimit(1)
                                if let description = library.descriptionText, !description.isEmpty {
                                    Text(description).font(.caption).foregroundStyle(.secondary).lineLimit(1)
                                }
                            }
                        }
                        .padding(.vertical, 5).tag(library.name)
                        .contextMenu {
                            Button("打开文件") { NSWorkspace.shared.open(URL(fileURLWithPath: library.path)) }
                            Button("删除", role: .destructive) { model.remove(library) }
                        }
                    }
                }.listStyle(.sidebar).scrollContentBackground(.hidden)
            }
        }
    }
    private var references: some View {
        let selectionContext = model.referenceSelectionContext
        return VStack(spacing: 0) {
            VStack(alignment: .leading, spacing: 12) {
                HStack {
                    Text(model.library?.name ?? "文献工作台").font(.title3.weight(.semibold))
                    Spacer()
                    Text("\(model.references.count) 条文献").font(.caption).foregroundStyle(.secondary)
                    if model.loading { ProgressView().controlSize(.small) }
                }
                HStack(spacing: 8) {
                    SVGIcon("search", size: 15).foregroundStyle(.secondary)
                    TextField("搜索文献", text: Binding(get: { model.query }, set: model.setQuery)).textFieldStyle(.plain)
                        .focused($searchFocused)
                        .background {
                            WorkbenchSearchNavigation(enabled: searchFocused, move: model.moveReference)
                        }
                }.padding(.horizontal, 8).padding(.vertical, 7).background(.quaternary.opacity(0.5), in: RoundedRectangle(cornerRadius: 7))
                HStack(spacing: 8) {
                    Picker("类型筛选", selection: Binding(get: { model.type }, set: model.setType)) { ForEach(types, id: \.0) { Text($0.1).tag($0.0) } }.labelsHidden()
                        .accessibilityLabel("类型筛选")
                    Picker("字段筛选", selection: Binding(get: { model.field }, set: model.setField)) { ForEach(fields, id: \.0) { Text($0.1).tag($0.0) } }.labelsHidden()
                        .accessibilityLabel("字段筛选")
                }.controlSize(.small)
            }.padding(16)
            Divider()
            if model.references.isEmpty && !model.loading {
                Text("暂无可显示的文献").foregroundStyle(.secondary).frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollViewReader { proxy in
                    List(selection: Binding(get: { model.selectedReference }, set: { model.selectReference($0, context: selectionContext) })) {
                        ForEach(model.references) { reference in
                            // The cite key is the value users retype and paste, so it sits on
                            // its own line at the top right and copies on click — off the
                            // metadata lines, where it competed with the title for width.
                            HStack(alignment: .top, spacing: 10) {
                                VStack(alignment: .leading, spacing: 5) {
                                    MathChunkText(chunks: reference.title).lineLimit(2)
                                    if !reference.author.isEmpty {
                                        Text(reference.author.joined(separator: ", ")).font(.callout).foregroundStyle(.secondary).lineLimit(1)
                                    }
                                    HStack(spacing: 6) {
                                        Text(reference.displayType)
                                        if let year = reference.year { Text("·"); Text(String(year)) }
                                        if !reference.venueText.isEmpty {
                                            Text("·")
                                            Text(reference.venueText).lineLimit(1)
                                        }
                                    }.font(.caption).foregroundStyle(.secondary)
                                }
                                Spacer(minLength: 8)
                                ListCiteKey(reference: reference, model: model)
                            }.padding(.vertical, 8).tag(reference.citeKey).id(reference.citeKey)
                            .contextMenu {
                                Button("复制引用键") { model.copy(reference.citeKey) }
                                Button("复制 BibTeX") { model.copy(reference.source) }
                            }
                        }
                    }.listStyle(.inset).scrollContentBackground(.hidden)
                    .onChange(of: model.selectedReference) { key in
                        if let key { proxy.scrollTo(key) }
                    }
                }
            }
        }
    }
    private var inspector: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("文献详情").font(.headline).padding(16)
            Divider()
            if let reference = model.reference {
                ReferenceInspector(reference: reference, context: model.referenceSelectionContext, model: model)
            } else {
                Text("选择一条文献查看字段和操作").foregroundStyle(.secondary).padding().frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
    }
}

/// Observe keys without replacing the SwiftUI field or changing its layout.
private struct WorkbenchSearchNavigation: NSViewRepresentable {
    let enabled: Bool
    let move: (Int) -> Void

    func makeCoordinator() -> Coordinator { Coordinator(self) }
    func makeNSView(context: Context) -> NSView {
        let view = NSView()
        context.coordinator.install(on: view)
        return view
    }
    func updateNSView(_ view: NSView, context: Context) {
        context.coordinator.parent = self
    }
    static func dismantleNSView(_ view: NSView, coordinator: Coordinator) {
        coordinator.removeMonitor()
    }
    final class Coordinator {
        var parent: WorkbenchSearchNavigation
        private var monitor: Any?
        init(_ parent: WorkbenchSearchNavigation) { self.parent = parent }
        func install(on view: NSView) {
            monitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self, weak view] event in
                guard let self, let window = view?.window,
                      parent.enabled, window.isKeyWindow, event.window === window,
                      let editor = window.firstResponder as? NSTextView,
                      editor.isFieldEditor, !editor.hasMarkedText(),
                      event.modifierFlags.intersection([.command, .control, .option, .shift]).isEmpty
                else { return event }
                switch event.keyCode {
                case 126: parent.move(-1)
                case 125: parent.move(1)
                default: return event
                }
                return nil
            }
        }
        func removeMonitor() {
            if let monitor { NSEvent.removeMonitor(monitor) }
            monitor = nil
        }
        deinit { removeMonitor() }
    }
}

/// The cite key on a workbench list row, pinned to the right of the title. It
/// copies on click and confirms on itself for a moment, so taking a key out of a
/// long result list never opens an alert or moves the selection.
private struct ListCiteKey: View {
    let reference: Reference
    @ObservedObject var model: WorkbenchModel

    var body: some View {
        let copied = model.copied == reference.citeKey
        Button {
            model.copy(reference.citeKey)
        } label: {
            HStack(spacing: 4) {
                SVGIcon(copied ? "check" : "copy", size: 10)
                Text(copied ? "已复制" : reference.citeKey)
                    .font(.system(.caption, design: copied ? .default : .monospaced))
                    .lineLimit(1)
            }
            .foregroundStyle(copied ? Color.green : Color.accentColor)
            .padding(.horizontal, 7).padding(.vertical, 3)
            .background(.primary.opacity(0.06), in: Capsule())
        }
        .buttonStyle(.plain)
        .help("复制引用键")
        .accessibilityLabel("复制引用键 \(reference.citeKey)")
    }
}

struct ReferenceInspector: View {
    let reference: Reference
    let context: WorkbenchModel.ReferenceSelectionContext
    @ObservedObject var model: WorkbenchModel
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                MathChunkText(chunks: reference.title).font(.title3)
                HStack {
                    copyButton(reference.citeKey, icon: "copy", label: "复制引用键")
                    copyButton(reference.source, icon: "clipboard", label: "复制 BibTeX")
                    if !reference.file.isEmpty { Button { model.openFile(reference, context: context) } label: { SVGIcon("folderOpen") }.help("打开文件").accessibilityLabel("打开文件") }
                    if !reference.url.isEmpty { Button { model.openURL(reference.url) } label: { SVGIcon("externalLink") }.help("打开 URL").accessibilityLabel("打开 URL") }
                    if !reference.doi.isEmpty { Button { model.openURL(reference.doi.hasPrefix("http") ? reference.doi : "https://doi.org/" + reference.doi) } label: { SVGIcon("link") }.help("打开 DOI").accessibilityLabel("打开 DOI") }
                }.buttonStyle(.bordered)
                ForEach(metadata, id: \.0) { item in
                    if !item.1.isEmpty {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(item.0).font(.caption).foregroundStyle(.secondary)
                            Text(item.1).textSelection(.enabled).frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
                rich("摘要", reference.abstractChunks)
                rich("书名", reference.bookTitle)
                rich("期号", reference.issue)
                rich("备注", reference.note)
                DisclosureGroup("BibTeX") {
                    Text(reference.source).font(.system(.caption, design: .monospaced)).textSelection(.enabled).frame(maxWidth: .infinity, alignment: .leading).padding(.top, 8)
                }
            }.padding(16)
        }
    }
    /// Both copy actions answer on the button itself: the icon becomes a check for
    /// a moment, so a copy that succeeded says so without an alert to dismiss.
    private func copyButton(_ text: String, icon: String, label: String) -> some View {
        let copied = model.copied == text
        return Button {
            model.copy(text)
        } label: {
            if copied {
                HStack(spacing: 4) {
                    SVGIcon("check", size: 14)
                    Text("已复制").font(.caption)
                }
            } else {
                SVGIcon(icon)
            }
        }
        .help(label)
        .accessibilityLabel(label)
    }

    @ViewBuilder private func rich(_ label: String, _ chunks: [TextChunk]) -> some View {
        if !chunks.isEmpty {
            VStack(alignment: .leading, spacing: 4) {
                Text(label).font(.caption).foregroundStyle(.secondary)
                MathChunkText(chunks: chunks).textSelection(.enabled)
            }
        }
    }
    private var metadata: [(String, String)] {
        [("引用键", reference.citeKey), ("类型", reference.displayType), ("作者", reference.author.joined(separator: ", ")),
         ("年份", reference.year.map(String.init) ?? ""), ("月份", reference.month), ("期刊", reference.journal), ("期刊全称", reference.fullJournal),
         ("卷号", reference.volume.map(String.init) ?? ""), ("编号", reference.number), ("页码", reference.pagesText), ("总页数", reference.bookPages),
         ("出版社", reference.publisher.joined(separator: ", ")), ("版本", reference.edition.map(String.init) ?? ""), ("丛书", reference.series),
         ("编辑", reference.editor.map { $0.1.isEmpty ? $0.0 : "\($0.0) (\($0.1))" }.joined(separator: ", ")),
         ("学校", reference.school), ("地址", reference.address), ("组织", reference.organization.joined(separator: ", ")), ("机构", reference.institution),
         ("DOI", reference.doi), ("ISBN", reference.isbn), ("MR 分类", reference.mrclass), ("URL", reference.url), ("文件", reference.file),
         ("Eprint", reference.eprint), ("Archive Prefix", reference.archivePrefix), ("arXiv 分类", reference.arxivPrimaryClass), ("发表方式", reference.howPublished)]
    }
}

struct AddLibrarySheet: View {
    @ObservedObject var model: WorkbenchModel
    @Environment(\.dismiss) private var dismiss
    @State private var name = ""
    @State private var path = ""
    @State private var description = ""
    @State private var error: String?
    @State private var saving = false
    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            Text("新增文献库").font(.title2.weight(.semibold))
            Text("添加一个 .bib 文件到你的工作空间").foregroundStyle(.secondary)
            Form {
                TextField("文献库名称", text: $name, prompt: Text("为文献库起一个名字"))
                LabeledContent("文件路径") {
                    Text(path.isEmpty ? "尚未选择文件" : path).lineLimit(1).truncationMode(.middle)
                    Button("选择文件", action: selectFile)
                }
                TextField("描述", text: $description, prompt: Text("简单描述一下这个文献库..."))
            }
            if let error { Text(error).foregroundStyle(.red).font(.callout) }
            HStack {
                Spacer()
                Button("取消") { dismiss() }.keyboardShortcut(.cancelAction)
                Button("保存", action: save).keyboardShortcut(.defaultAction).disabled(name.trimmingCharacters(in: .whitespaces).isEmpty || path.isEmpty || saving)
            }
        }.padding(24).frame(width: 480).interactiveDismissDisabled(saving)
    }
    private func selectFile() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [UTType(filenameExtension: "bib") ?? .plainText]
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        if panel.runModal() == .OK, let url = panel.url { path = url.path; if name.isEmpty { name = url.deletingPathExtension().lastPathComponent } }
    }
    private func save() {
        let trimmed = name.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !model.libraries.contains(where: { $0.name == trimmed }) else { error = "该名称已存在，请换一个"; return }
        saving = true
        Task {
            do {
                try await RustCore.shared.addLibrary(name: trimmed, path: path, description: description)
                await model.reload()
                model.selectLibrary(trimmed)
                dismiss()
            } catch { self.error = error.localizedDescription; saving = false }
        }
    }
}

struct WindowMaterial: NSViewRepresentable {
    func makeNSView(context: Context) -> NSView {
        if #available(macOS 26.0, *) {
            let glass = NSGlassEffectView(); glass.style = .regular; glass.cornerRadius = 0
            return glass
        }
        let effect = NSVisualEffectView()
        effect.material = .underWindowBackground
        effect.blendingMode = .behindWindow
        effect.state = .followsWindowActiveState
        return effect
    }
    func updateNSView(_ view: NSView, context: Context) {}
}
