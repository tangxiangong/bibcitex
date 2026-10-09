import SwiftUI

/// Tray surface metrics; colours come from `PanelPalette`, shared with the helper.
enum TrayMetrics {
    static let panelRadius: CGFloat = 20
    static let detailRadius: CGFloat = 16
    static let fieldRadius: CGFloat = 8
}

/// A purpose-built tray browser: library controls above a list and a compact reading pane.
struct TrayWorkbenchView: View {
    @AppStorage("language") private var language = "system"
    @ObservedObject var model: TrayWorkbenchModel
    let showMain: () -> Void
    let dismiss: () -> Void
    @FocusState private var searchFocused: Bool
    @FocusState private var detailToggleFocused: Bool
    @FocusState private var detailCloseFocused: Bool
    private var types: [(String, String)] { [("all", L10n.text("全部类型")), ("Article", L10n.text("期刊论文")), ("Book", L10n.text("图书")), ("Thesis", L10n.text("学位论文")), ("TechReport", L10n.text("技术报告")), ("Misc", L10n.text("其他")), ("Booklet", L10n.text("小册子")), ("InBook", L10n.text("书籍章节")), ("InCollection", L10n.text("文集章节")), ("InProceedings", L10n.text("会议论文"))] }
    private var fields: [(String, String)] { [("all", L10n.text("全部字段")), ("author", L10n.text("作者")), ("title", L10n.text("标题")), ("journal", L10n.text("期刊")), ("year", L10n.text("年份"))] }

    var body: some View {
        VStack(spacing: 0) {
            VStack(spacing: 12) {
                HStack(spacing: 10) {
                    Text("BibCiTeX").font(.headline)
                    Spacer()
                    toolbarButtons
                }
                HStack(spacing: 8) {
                    NativeIcon("search", size: 15).foregroundStyle(.secondary)
                    TextField(L10n.text("搜索文献"), text: $model.query)
                        .textFieldStyle(.plain).focused($searchFocused)
                        .padding(.trailing, 174)
                }
                .overlay(alignment: .trailing) {
                    HStack(spacing: 10) {
                        PanelSeparator(vertical: true).frame(height: 16)
                        Menu {
                            Picker(L10n.text("文献库"), selection: $model.libraryName) {
                                ForEach(model.libraries) { library in
                                    Text(library.name).tag(Optional(library.name))
                                }
                            }
                        } label: {
                            HStack(spacing: 6) {
                                NativeIcon("library", size: 13).foregroundStyle(.secondary)
                                Text(model.libraryName ?? L10n.text("暂无文献库"))
                                    .font(.system(size: 12, weight: .medium))
                                    .lineLimit(1).truncationMode(.middle)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                NativeIcon("chevronDown", size: 10).foregroundStyle(.secondary)
                            }.frame(width: 150, height: 20).contentShape(Rectangle())
                        }
                        .menuStyle(.borderlessButton).menuIndicator(.hidden)
                        .disabled(model.libraries.isEmpty)
                        .help(model.libraryName ?? L10n.text("文献库")).accessibilityLabel(L10n.text("文献库"))
                    }
                }
                .padding(.horizontal, 10).padding(.vertical, 8)
                .background(PanelPalette.controlSurface, in: RoundedRectangle(cornerRadius: TrayMetrics.fieldRadius, style: .continuous))
                .overlay {
                    RoundedRectangle(cornerRadius: TrayMetrics.fieldRadius, style: .continuous)
                        .strokeBorder(searchFocused ? Color.accentColor.opacity(0.6) : PanelPalette.border, lineWidth: 1)
                        .allowsHitTesting(false)
                }
                HStack(spacing: 8) {
                    Picker(L10n.text("类型筛选"), selection: $model.type) {
                        ForEach(types, id: \.0) { Text($0.1).tag($0.0) }
                    }.labelsHidden().accessibilityLabel(L10n.text("类型筛选"))
                    Picker(L10n.text("字段筛选"), selection: $model.field) {
                        ForEach(fields, id: \.0) { Text($0.1).tag($0.0) }
                    }.labelsHidden().accessibilityLabel(L10n.text("字段筛选"))
                    Spacer()
                    if model.loading { ProgressView().controlSize(.small) }
                    Text(L10n.references(model.references.count)).font(.caption).foregroundStyle(.secondary)
                }.controlSize(.small)
            }.padding(16)
            PanelSeparator()
            results.frame(maxWidth: .infinity)
                .overlay(alignment: .trailing) {
                    if model.detailVisible {
                        VStack(spacing: 0) {
                            HStack {
                                Text(L10n.text("文献详情")).font(.caption).foregroundStyle(.secondary)
                                Spacer()
                                iconButton("x", L10n.text("关闭")) {
                                    model.detailVisible = false
                                }.focused($detailCloseFocused)
                            }.padding(.horizontal, 14).padding(.vertical, 8)
                            PanelSeparator()
                            detail
                        }
                        .frame(width: 282)
                        .frame(maxHeight: .infinity)
                        .background(Color(nsColor: .windowBackgroundColor), in: RoundedRectangle(cornerRadius: TrayMetrics.detailRadius, style: .continuous))
                        .clipShape(RoundedRectangle(cornerRadius: TrayMetrics.detailRadius, style: .continuous))
                        .overlay {
                            RoundedRectangle(cornerRadius: TrayMetrics.detailRadius, style: .continuous)
                                .strokeBorder(PanelPalette.border, lineWidth: 1)
                                .allowsHitTesting(false)
                        }
                        .shadow(color: .black.opacity(0.18), radius: 12, x: -3, y: 4)
                        .padding(12)
                    }
                }
            if let error = model.error {
                PanelSeparator()
                HStack {
                    NativeIcon("alert", size: 14)
                    Text(error).font(.caption).lineLimit(3)
                    Spacer()
                    iconButton("x", L10n.text("关闭")) { model.error = nil }
                }.foregroundStyle(.red).padding(10)
            }
        }
        .background(PanelPalette.scrim)
        .background(PanelMaterial(radius: TrayMetrics.panelRadius, fallback: .popover))
        .clipShape(RoundedRectangle(cornerRadius: TrayMetrics.panelRadius, style: .continuous))
        // Like the helper, the tray never shows scrollers.
        .scrollIndicators(.never)
        .background(AutoHidingScrollbars(alwaysHidden: true))
        .background(NeutralListSelection())
        .onAppear { searchFocused = true }
        .onChange(of: model.detailVisible) { visible in
            if visible { detailCloseFocused = true } else { detailToggleFocused = true }
        }
    }
    @ViewBuilder private var toolbarButtons: some View {
        toolbarButtonGroup
            .padding(4)
            .panelGlassCapsule()
    }
    private var toolbarButtonGroup: some View {
        HStack(spacing: 8) {
            iconButton(model.detailVisible ? "panelRightClose" : "panelRightOpen", L10n.text("文献详情")) {
                model.detailVisible.toggle()
            }
            .focused($detailToggleFocused)
            .accessibilityAddTraits(model.detailVisible ? .isSelected : [])
            iconButton("refresh", L10n.text("刷新")) { Task { await model.reload() } }
            iconButton("externalLink", L10n.text("显示窗口"), action: showMain)
            iconButton("x", L10n.text("关闭"), action: dismiss)
        }
    }
    private var results: some View {
        ScrollViewReader { proxy in
            List(selection: $model.selection) {
                ForEach(model.references) { reference in
                    VStack(alignment: .leading, spacing: 5) {
                        HStack(alignment: .top, spacing: 12) {
                            MathChunkText(chunks: reference.title).lineLimit(2)
                                .frame(maxWidth: .infinity, alignment: .leading)
                            citeKeyButton(reference.citeKey)
                                .frame(maxWidth: 180, alignment: .trailing)
                        }
                        if !reference.author.isEmpty {
                            Text(reference.author.joined(separator: ", ")).font(.caption).foregroundStyle(.secondary).lineLimit(1)
                        }
                        Text([reference.displayType, reference.year.map(String.init) ?? "", reference.venueText]
                            .filter { !$0.isEmpty }.joined(separator: " · "))
                            .font(.caption2).foregroundStyle(.secondary).lineLimit(1)
                    }.padding(.vertical, 7).tag(reference.citeKey).id(reference.citeKey)
                    .foregroundStyle(Color(nsColor: .labelColor))
                    .listRowBackground(RowSelectionBackground(selected: model.selection == reference.citeKey))
                }
            }.listStyle(.plain).scrollContentBackground(.hidden)
                .overlay {
                    if model.references.isEmpty && !model.loading {
                        Text(L10n.text("暂无可显示的文献")).font(.callout).foregroundStyle(.secondary)
                    }
                }
                .onChange(of: model.selection) { key in if let key { proxy.scrollTo(key) } }
        }
    }
    private func citeKeyButton(_ key: String) -> some View {
        let copied = model.copied[.list] == key
        return Button { model.copy(key) } label: {
            HStack(spacing: 4) {
                NativeIcon(copied ? "check" : "copy", size: 10)
                Text(copied ? L10n.text("已复制") : key)
                    .font(.system(.caption, design: copied ? .default : .monospaced))
                    .lineLimit(1)
            }
            .foregroundStyle(copied ? Color.green : Color.accentColor)
            .padding(.horizontal, 7).padding(.vertical, 3)
            .background(PanelPalette.controlSurface, in: Capsule())
        }
        .buttonStyle(.plain)
        .help(L10n.text("复制引用键"))
        .accessibilityLabel(L10n.text("复制引用键 {0}", key))
    }
    @ViewBuilder private var detail: some View {
        if let reference = model.reference {
            VStack(alignment: .leading, spacing: 0) {
                ScrollView {
                    VStack(alignment: .leading, spacing: 12) {
                        MathChunkText(chunks: reference.title).font(.headline)
                        metadata(L10n.text("引用键"), reference.citeKey)
                        metadata(L10n.text("作者"), reference.author.joined(separator: ", "))
                        metadata(L10n.text("类型"), reference.displayType)
                        metadata(L10n.text("年份"), reference.year.map(String.init) ?? "")
                        metadata(L10n.text("期刊"), reference.venueText)
                        metadata("DOI", reference.doi)
                        metadata("URL", reference.url)
                        if !reference.abstractChunks.isEmpty {
                            Text(L10n.text("摘要")).font(.caption).foregroundStyle(.secondary)
                            MathChunkText(chunks: reference.abstractChunks).textSelection(.enabled)
                        }
                        DisclosureGroup("BibTeX") {
                            Text(reference.source).font(.system(.caption, design: .monospaced))
                                .textSelection(.enabled).frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }.frame(maxWidth: .infinity, alignment: .leading).padding(14)
                }
                PanelSeparator()
                HStack(spacing: 8) {
                    copyButton(reference.citeKey, L10n.text("复制引用键"))
                    copyButton(reference.source, L10n.text("复制 BibTeX"))
                }.controlSize(.small).padding(12)
            }
        } else {
            Text(L10n.text("选择一条文献查看字段和操作")).font(.callout).foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, maxHeight: .infinity).padding(16)
        }
    }
    @ViewBuilder private func metadata(_ label: String, _ value: String) -> some View {
        if !value.isEmpty {
            VStack(alignment: .leading, spacing: 3) {
                Text(label).font(.caption).foregroundStyle(.secondary)
                Text(value).font(.callout).textSelection(.enabled)
            }
        }
    }
    private func copyButton(_ value: String, _ label: String) -> some View {
        Button { model.copy(value, surface: .detail) } label: {
            HStack(spacing: 4) {
                NativeIcon(model.copied[.detail] == value ? "check" : "copy", size: 12)
                Text(model.copied[.detail] == value ? L10n.text("已复制") : label)
            }
        }.accessibilityLabel(label)
    }
    private func iconButton(_ icon: String, _ label: String, action: @escaping () -> Void) -> some View {
        Button(action: action) { NativeIcon(icon, size: 16).frame(width: 24, height: 24) }
            .buttonStyle(.plain).help(label).accessibilityLabel(label)
    }
}
