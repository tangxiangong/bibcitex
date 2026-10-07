import SwiftUI
import AppKit
import UniformTypeIdentifiers

struct WorkbenchView: View {
    @AppStorage("language") private var language = "system"
    @Environment(\.openWindow) private var openWindow
    @ObservedObject var model: WorkbenchModel
    @FocusState private var searchFocused: Bool
    @State private var editingLibrary: Bibliography?
    private var types: [(String, String)] { [("all", L10n.text("全部类型")), ("Article", L10n.text("期刊论文")), ("Book", L10n.text("图书")), ("Thesis", L10n.text("学位论文")), ("TechReport", L10n.text("技术报告")), ("Misc", L10n.text("其他")), ("Booklet", L10n.text("小册子")), ("InBook", L10n.text("书籍章节")), ("InCollection", L10n.text("文集章节")), ("InProceedings", L10n.text("会议论文"))] }
    private var fields: [(String, String)] { [("all", L10n.text("全部字段")), ("author", L10n.text("作者")), ("title", L10n.text("标题")), ("journal", L10n.text("期刊")), ("year", L10n.text("年份"))] }

    var body: some View {
        panes
            .background(AutoHidingScrollbars())
            .background(NeutralListSelection())
            .frame(minWidth: 800, minHeight: 520)
            .toolbar {
                // NavigationSplitView supplies its own sidebar toggle, so the
                // hand-written one is only needed by the 13 fallback layout.
                if #unavailable(macOS 14.0) {
                    ToolbarItem(placement: .navigation) {
                        Button { model.showSidebar.toggle() } label: { NativeIcon(model.showSidebar ? "panelLeftClose" : "panelLeftOpen") }.help(L10n.text("文献库")).accessibilityLabel(L10n.text("文献库"))
                    }
                }
                if #available(macOS 26.0, *) {
                    ToolbarItem(placement: .navigation) { toolbarLogo }
                        .sharedBackgroundVisibility(.hidden)
                } else {
                    ToolbarItem(placement: .navigation) { toolbarLogo }
                }
                if #unavailable(macOS 14.0) {
                    ToolbarItemGroup {
                        Spacer()
                        toolbarActions
                    }
                }
            }
        .sheet(isPresented: $model.adding) { AddLibrarySheet(model: model) }
        .sheet(item: $editingLibrary) { library in AddLibrarySheet(model: model, library: library) }
        .alert(L10n.text("错误"), isPresented: Binding(get: { model.error != nil }, set: { if !$0 { model.error = nil } })) {
            Button(L10n.text("确定")) { model.error = nil }
        } message: { Text(model.error ?? "") }
        .task { await model.reload() }
    }

    @ViewBuilder private var toolbarActions: some View {
        Button { HelperPanelController.shared.showPanel() } label: { NativeIcon("search") }.help(L10n.text("快捷助手")).accessibilityLabel(L10n.text("快捷助手"))
        Button { Task { await model.reload() } } label: { NativeIcon("refresh") }.help(L10n.text("刷新")).accessibilityLabel(L10n.text("刷新"))
        Button { model.showInspector.toggle() } label: { NativeIcon(model.showInspector ? "panelRightClose" : "panelRightOpen") }.help(L10n.text("文献详情")).accessibilityLabel(L10n.text("文献详情"))
        Button { openWindow(id: "settings") } label: { NativeIcon("settings") }.help(L10n.text("设置")).accessibilityLabel(L10n.text("设置"))
    }

    /// The standard three-pane idiom where the system provides it. On macOS 14+
    /// `NavigationSplitView` owns the sidebar and `.inspector` owns the trailing
    /// pane, so both panes carry system material and the system draws the boundary.
    /// The macOS 13 fallback keeps `HSplitView` and applies the same semantics by
    /// hand: the sidebar material, the content background, and one separator.
    @ViewBuilder private var panes: some View {
        if #available(macOS 14.0, *) {
            NavigationSplitView(columnVisibility: sidebarVisibility) {
                sidebar.navigationSplitViewColumnWidth(min: 180, ideal: 220, max: 300)
            } detail: {
                references.frame(minWidth: 330, maxWidth: .infinity, maxHeight: .infinity)
            }
            .inspector(isPresented: $model.showInspector) {
                inspector.inspectorColumnWidth(min: 280, ideal: 350, max: 520)
                    .toolbar {
                        if #available(macOS 26.0, *) {
                            ToolbarSpacer(.flexible)
                        }
                        ToolbarItemGroup {
                            if #unavailable(macOS 26.0) { Spacer() }
                            toolbarActions
                        }
                    }
            }
        } else {
            HSplitView {
                if model.showSidebar {
                    sidebar.frame(minWidth: 180, idealWidth: 220, maxWidth: 300, maxHeight: .infinity)
                        .background(SystemSidebarBackground())
                    SystemSeparator(vertical: true)
                }
                references.frame(minWidth: 330, maxWidth: .infinity, maxHeight: .infinity)
                    .background(SystemPaneBackground())
                if model.showInspector {
                    SystemSeparator(vertical: true)
                    inspector.frame(minWidth: 280, idealWidth: 350, maxWidth: 520, maxHeight: .infinity)
                        .background(SystemSidebarBackground())
                }
            }
        }
    }

    /// Sidebar and inspector stay independent choices, so "sidebar hidden with the
    /// inspector shown" stays expressible; `.detailOnly` is the only value either
    /// column can report as hidden on its own.
    private var sidebarVisibility: Binding<NavigationSplitViewVisibility> {
        Binding(
            get: { model.showSidebar ? .all : .detailOnly },
            set: { model.showSidebar = $0 != .detailOnly }
        )
    }

    @ViewBuilder private var toolbarLogo: some View {
        if let logo = AppImages.logo {
            Image(nsImage: logo).resizable().scaledToFit()
                .frame(width: 48, height: 48).accessibilityLabel("BibCiTeX")
        }
    }

    private var sidebar: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Text(L10n.text("文献库")).font(.headline)
                Spacer()
                Button { model.adding = true } label: { NativeIcon("folderAdd", size: 18).frame(width: 28, height: 28) }
                    .buttonStyle(.borderless).help(L10n.text("新增文献库")).accessibilityLabel(L10n.text("新增文献库"))
            }.padding(.horizontal, 16).padding(.vertical, 12)
            if model.libraries.isEmpty {
                Text(L10n.text("暂无文献库")).foregroundStyle(.secondary).frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                List(selection: Binding(get: { model.selectedLibrary }, set: model.selectLibrary)) {
                    ForEach(model.libraries) { library in
                        LibrarySidebarRow(model: model, library: library) {
                            editingLibrary = library
                        }.tag(library.name)
                        .foregroundStyle(Color(nsColor: .labelColor))
                        .listRowBackground(model.selectedLibrary == library.name
                            ? Color(nsColor: .unemphasizedSelectedContentBackgroundColor) : Color.clear)
                    }
                }.listStyle(.sidebar)
            }
        }
    }
    private var references: some View {
        let selectionContext = model.referenceSelectionContext
        return VStack(spacing: 0) {
            if !model.libraries.isEmpty {
                VStack(alignment: .leading, spacing: 12) {
                    HStack {
                        Text(model.library?.name ?? L10n.text("文献工作台")).font(.title3.weight(.semibold))
                        Spacer()
                        Text(L10n.references(model.references.count)).font(.caption).foregroundStyle(.secondary)
                        if model.loading { ProgressView().controlSize(.small) }
                    }
                    HStack(spacing: 8) {
                        NativeIcon("search", size: 15).foregroundStyle(.secondary)
                        TextField(L10n.text("搜索文献"), text: Binding(get: { model.query }, set: model.setQuery)).textFieldStyle(.plain)
                            .focused($searchFocused)
                            .background {
                                WorkbenchSearchNavigation(enabled: searchFocused, move: model.moveReference)
                            }
                    }.padding(.horizontal, 8).padding(.vertical, 7).background(.quaternary.opacity(0.5), in: RoundedRectangle(cornerRadius: 7))
                    HStack(spacing: 8) {
                        Picker(L10n.text("类型筛选"), selection: Binding(get: { model.type }, set: model.setType)) { ForEach(types, id: \.0) { Text($0.1).tag($0.0) } }.labelsHidden()
                            .accessibilityLabel(L10n.text("类型筛选"))
                        Picker(L10n.text("字段筛选"), selection: Binding(get: { model.field }, set: model.setField)) { ForEach(fields, id: \.0) { Text($0.1).tag($0.0) } }.labelsHidden()
                            .accessibilityLabel(L10n.text("字段筛选"))
                    }.controlSize(.small)
                }.padding(16)
                Divider()
            }
            if model.references.isEmpty && !model.loading {
                VStack(spacing: 16) {
                    if model.libraries.isEmpty, let logo = AppImages.logo {
                        Image(nsImage: logo)
                            .resizable()
                            .scaledToFit()
                            .frame(width: 96, height: 96)
                            .accessibilityLabel("BibCiTeX")
                    }
                    Text(L10n.text("暂无可显示的文献")).foregroundStyle(.secondary)
                }.frame(maxWidth: .infinity, maxHeight: .infinity)
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
                            .foregroundStyle(Color(nsColor: .labelColor))
                            .listRowBackground(model.selectedReference == reference.citeKey
                                ? Color(nsColor: .unemphasizedSelectedContentBackgroundColor) : Color.clear)
                            .contextMenu {
                                Button(L10n.text("复制引用键")) { model.copy(reference.citeKey) }
                                Button(L10n.text("复制 BibTeX")) { model.copy(reference.source) }
                            }
                        }
                    }.listStyle(.inset)
                    .onChange(of: model.selectedReference) { key in
                        if let key { proxy.scrollTo(key) }
                    }
                }
            }
        }
    }
    private var inspector: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(L10n.text("文献详情")).font(.headline).padding(16)
            Divider()
            if let reference = model.reference {
                ReferenceInspector(reference: reference, context: model.referenceSelectionContext, model: model)
            } else {
                Text(L10n.text("选择一条文献查看字段和操作")).foregroundStyle(.secondary).padding().frame(maxWidth: .infinity, maxHeight: .infinity)
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
    @AppStorage("language") private var language = "system"
    let reference: Reference
    @ObservedObject var model: WorkbenchModel

    var body: some View {
        let copied = model.copied[.list] == reference.citeKey
        Button {
            model.copy(reference.citeKey)
        } label: {
            HStack(spacing: 4) {
                NativeIcon(copied ? "check" : "copy", size: 10)
                Text(copied ? L10n.text("已复制") : reference.citeKey)
                    .font(.system(.caption, design: copied ? .default : .monospaced))
                    .lineLimit(1)
            }
            .foregroundStyle(copied ? Color.green : Color.accentColor)
            .padding(.horizontal, 7).padding(.vertical, 3)
            .background(.primary.opacity(0.06), in: Capsule())
        }
        .buttonStyle(.plain)
        .help(L10n.text("复制引用键"))
        .accessibilityLabel(L10n.text("复制引用键 {0}", reference.citeKey))
    }
}

struct ReferenceInspector: View {
    @AppStorage("language") private var language = "system"
    let reference: Reference
    let context: WorkbenchModel.ReferenceSelectionContext
    @ObservedObject var model: WorkbenchModel
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                MathChunkText(chunks: reference.title).font(.title3)
                HStack {
                    copyButton(reference.citeKey, icon: "copy", label: L10n.text("复制引用键"))
                    copyButton(reference.source, icon: "clipboard", label: L10n.text("复制 BibTeX"))
                    if !reference.file.isEmpty { Button { model.openFile(reference, context: context) } label: { NativeIcon("folderOpen") }.help(L10n.text("打开文件")).accessibilityLabel(L10n.text("打开文件")) }
                    if !reference.url.isEmpty { Button { model.openURL(reference.url) } label: { NativeIcon("externalLink") }.help(L10n.text("打开 URL")).accessibilityLabel(L10n.text("打开 URL")) }
                    if !reference.doi.isEmpty { Button { model.openURL(reference.doi.hasPrefix("http") ? reference.doi : "https://doi.org/" + reference.doi) } label: { NativeIcon("link") }.help(L10n.text("打开 DOI")).accessibilityLabel(L10n.text("打开 DOI")) }
                }.buttonStyle(.bordered)
                ForEach(metadata, id: \.0) { item in
                    if !item.1.isEmpty {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(item.0).font(.caption).foregroundStyle(.secondary)
                            Text(item.1).textSelection(.enabled).frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
                rich(L10n.text("摘要"), reference.abstractChunks)
                rich(L10n.text("书名"), reference.bookTitle)
                rich(L10n.text("期号"), reference.issue)
                rich(L10n.text("备注"), reference.note)
                DisclosureGroup("BibTeX") {
                    Text(reference.source).font(.system(.caption, design: .monospaced)).textSelection(.enabled).frame(maxWidth: .infinity, alignment: .leading).padding(.top, 8)
                }
            }.padding(16)
        }
    }
    /// Both copy actions answer on the button itself: the icon becomes a check for
    /// a moment, so a copy that succeeded says so without an alert to dismiss.
    private func copyButton(_ text: String, icon: String, label: String) -> some View {
        let copied = model.copied[.detail] == text
        return Button {
            model.copy(text, surface: .detail)
        } label: {
            if copied {
                HStack(spacing: 4) {
                    NativeIcon("check", size: 14)
                    Text(L10n.text("已复制")).font(.caption)
                }
            } else {
                NativeIcon(icon)
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
        [(L10n.text("引用键"), reference.citeKey), (L10n.text("类型"), reference.displayType), (L10n.text("作者"), reference.author.joined(separator: ", ")),
         (L10n.text("年份"), reference.year.map(String.init) ?? ""), (L10n.text("月份"), reference.month), (L10n.text("期刊"), reference.journal), (L10n.text("期刊全称"), reference.fullJournal),
         (L10n.text("卷号"), reference.volume.map(String.init) ?? ""), (L10n.text("编号"), reference.number), (L10n.text("页码"), reference.pagesText), (L10n.text("总页数"), reference.bookPages),
         (L10n.text("出版社"), reference.publisher.joined(separator: ", ")), (L10n.text("版本"), reference.edition.map(String.init) ?? ""), (L10n.text("丛书"), reference.series),
         (L10n.text("metadata.editor"), reference.editor.map { $0.1.isEmpty ? $0.0 : "\($0.0) (\($0.1))" }.joined(separator: ", ")),
         (L10n.text("学校"), reference.school), (L10n.text("地址"), reference.address), (L10n.text("组织"), reference.organization.joined(separator: ", ")), (L10n.text("机构"), reference.institution),
         ("DOI", reference.doi), ("ISBN", reference.isbn), (L10n.text("MR 分类"), reference.mrclass), ("URL", reference.url), (L10n.text("文件"), reference.file),
         ("Eprint", reference.eprint), ("Archive Prefix", reference.archivePrefix), (L10n.text("arXiv 分类"), reference.arxivPrimaryClass), (L10n.text("发表方式"), reference.howPublished)]
    }
}

private struct LibrarySidebarRow: View {
    @AppStorage("language") private var language = "system"
    @ObservedObject var model: WorkbenchModel
    let library: Bibliography
    let edit: () -> Void
    @State private var hovered = false
    @FocusState private var menuFocused: Bool
    @FocusState private var renameFocused: Bool
    @State private var renaming = false
    @State private var draftName = ""
    @State private var renameErrorDetails: LocalizedMessage?
    private var renameError: String? { renameErrorDetails?.text }
    @State private var savingName = false

    var body: some View {
        HStack(spacing: 9) {
            NativeIcon("library")
            VStack(alignment: .leading, spacing: 4) {
                if renaming {
                    TextField(L10n.text("文献库名称"), text: $draftName)
                        .textFieldStyle(.roundedBorder).focused($renameFocused)
                        .disabled(savingName).onSubmit(saveName)
                        .onExitCommand { if !savingName { renaming = false; renameFocused = false } }
                        .onChange(of: renameFocused) { focused in if !focused { saveName() } }
                        .task { renameFocused = true }
                    if let renameError { Text(renameError).font(.caption).foregroundStyle(.red) }
                } else {
                    Text(library.name).lineLimit(1)
                }
                if let description = library.descriptionText, !description.isEmpty {
                    Text(description).font(.caption).foregroundStyle(.secondary).lineLimit(1)
                }
            }
            Spacer(minLength: 0)
            if library.pinned { NativeIcon("pin", size: 13).foregroundStyle(.secondary).accessibilityLabel(L10n.text("已置顶")) }
            Menu { actions } label: { NativeIcon("more", size: 16).frame(width: 24, height: 24) }
                .menuStyle(.borderlessButton).menuIndicator(.hidden).fixedSize()
                .focused($menuFocused)
                .opacity(!renaming && (hovered || menuFocused) ? 1 : 0)
                .disabled(renaming)
                .help(L10n.text("文献库操作")).accessibilityLabel(L10n.text("{0}的操作", library.name))
        }
        .padding(.vertical, 5).contentShape(Rectangle())
        .onHover { hovered = $0 }
        .contextMenu { actions }
    }

    private func saveName() {
        guard renaming, !savingName else { return }
        let name = draftName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !name.isEmpty else { renameErrorDetails = LocalizedMessage(key: "文献库名称不能为空"); renameFocused = true; return }
        guard name != library.name else { renaming = false; return }
        savingName = true
        Task {
            do {
                let wasSelected = model.selectedLibrary == library.name
                try await RustCore.shared.updateLibrary(name: library.name, newName: name, path: nil, description: nil)
                renaming = false
                await model.reload()
                if wasSelected { model.selectLibrary(name) }
            } catch {
                renameErrorDetails = LocalizedMessage(error: error)
                renameFocused = true
            }
            savingName = false
        }
    }

    @ViewBuilder private var actions: some View {
        Button(action: edit) { Label { Text(L10n.text("编辑")) } icon: { if let image = NativeImages.image(named: "settings") { image } } }
        Button { draftName = library.name; renameErrorDetails = nil; renaming = true } label: { Label { Text(L10n.text("重命名")) } icon: { if let image = NativeImages.image(named: "rename") { image } } }
        Button {
            Task {
                do {
                    try await RustCore.shared.setLibraryPinned(name: library.name, pinned: !library.pinned)
                    await model.reload()
                } catch { model.reportError(error) }
            }
        } label: { Label { Text(library.pinned ? L10n.text("取消置顶") : L10n.text("置顶")) } icon: { if let image = NativeImages.image(named: "pin") { image } } }
        Divider()
        Button(L10n.text("打开文件")) { NSWorkspace.shared.open(URL(fileURLWithPath: library.path)) }
        Button(role: .destructive) { model.remove(library) } label: { Label { Text(L10n.text("移除")) } icon: { if let image = NativeImages.image(named: "x") { image } } }
    }
}

struct AddLibrarySheet: View {
    @AppStorage("language") private var language = "system"
    @ObservedObject var model: WorkbenchModel
    @Environment(\.dismiss) private var dismiss
    @State private var name = ""
    @State private var path = ""
    @State private var description = ""
    @State private var errorDetails: LocalizedMessage?
    private var error: String? { errorDetails?.text }
    @State private var saving = false
    private let library: Bibliography?

    init(model: WorkbenchModel, library: Bibliography? = nil) {
        self.model = model
        self.library = library
        _name = State(initialValue: library?.name ?? "")
        _path = State(initialValue: library?.path ?? "")
        _description = State(initialValue: library?.descriptionText ?? "")
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(alignment: .top, spacing: 14) {
                NativeIcon("folderAdd", size: 24)
                    .foregroundStyle(Color.accentColor)
                    .frame(width: 48, height: 48)
                    .background(Color.accentColor.opacity(0.1), in: RoundedRectangle(cornerRadius: 12))
                VStack(alignment: .leading, spacing: 6) {
                    Text(library == nil ? L10n.text("新增文献库") : L10n.text("编辑文献库")).font(.title2.weight(.semibold))
                    if library == nil { Text(L10n.text("添加一个 .bib 文件到你的工作空间")).font(.callout).foregroundStyle(.secondary) }
                }.padding(.top, 3)
            }.padding(.bottom, 24)
            VStack(alignment: .leading, spacing: 20) {
                VStack(alignment: .leading, spacing: 8) {
                    Text(L10n.text("文献库名称")).font(.callout.weight(.medium))
                    TextField(L10n.text("文献库名称"), text: $name, prompt: Text(L10n.text("为文献库起一个名字")))
                        .labelsHidden().textFieldStyle(.roundedBorder).controlSize(.large)
                }
                Group {
                    VStack(alignment: .leading, spacing: 8) {
                        Text(L10n.text("文件路径")).font(.callout.weight(.medium))
                        HStack(spacing: 12) {
                            NativeIcon("fileText", size: 24).foregroundStyle(.secondary)
                            Text(path.isEmpty ? L10n.text("尚未选择文件") : path)
                                .font(.callout).foregroundStyle(path.isEmpty ? .secondary : .primary)
                                .lineLimit(2).truncationMode(.middle).frame(maxWidth: .infinity, alignment: .leading)
                                .help(path)
                            Button(L10n.text("选择文件"), action: selectFile).controlSize(.large)
                        }
                        .padding(16).frame(maxWidth: .infinity)
                        .background(.quaternary.opacity(0.35), in: RoundedRectangle(cornerRadius: 10))
                        .overlay(RoundedRectangle(cornerRadius: 10).strokeBorder(.separator.opacity(0.5)))
                    }
                    VStack(alignment: .leading, spacing: 8) {
                        Text(L10n.text("描述")).font(.callout.weight(.medium))
                        TextField(L10n.text("描述"), text: $description, prompt: Text(L10n.text("简单描述一下这个文献库...")), axis: .vertical)
                            .labelsHidden().lineLimit(3...5).textFieldStyle(.roundedBorder).controlSize(.large)
                    }
                }
                if let error { Text(error).foregroundStyle(.red).font(.callout).textSelection(.enabled) }
            }.disabled(saving)
            Divider().padding(.top, 24).padding(.bottom, 16)
            HStack(spacing: 12) {
                if saving { ProgressView().controlSize(.small) }
                Spacer()
                Button(L10n.text("取消")) { dismiss() }.keyboardShortcut(.cancelAction).disabled(saving)
                Button(L10n.text("保存"), action: save).keyboardShortcut(.defaultAction).buttonStyle(.borderedProminent)
                    .disabled(name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || path.isEmpty || saving)
            }.controlSize(.large)
        }.padding(28).frame(width: 520).interactiveDismissDisabled(saving)
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
        guard !model.libraries.contains(where: { $0.name == trimmed && $0.name != library?.name }) else { errorDetails = LocalizedMessage(key: "该名称已存在，请换一个"); return }
        saving = true
        Task {
            do {
                let wasSelected = library == nil || model.selectedLibrary == library?.name
                if let library {
                    try await RustCore.shared.updateLibrary(name: library.name, newName: trimmed, path: path == library.path ? nil : path, description: description)
                } else {
                    try await RustCore.shared.addLibrary(name: trimmed, path: path, description: description)
                }
                await model.reload()
                if wasSelected { model.selectLibrary(trimmed) }
                dismiss()
            } catch { self.errorDetails = LocalizedMessage(error: error); saving = false }
        }
    }
}
