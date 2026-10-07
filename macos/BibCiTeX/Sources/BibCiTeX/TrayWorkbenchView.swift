import SwiftUI

/// A purpose-built tray browser: library controls above a list and a compact reading pane.
struct TrayWorkbenchView: View {
    @AppStorage("language") private var language = "system"
    @ObservedObject var model: TrayWorkbenchModel
    let showMain: () -> Void
    let dismiss: () -> Void
    @FocusState private var searchFocused: Bool
    private var types: [(String, String)] { [("all", L10n.text("全部类型")), ("Article", L10n.text("期刊论文")), ("Book", L10n.text("图书")), ("Thesis", L10n.text("学位论文")), ("TechReport", L10n.text("技术报告")), ("Misc", L10n.text("其他")), ("Booklet", L10n.text("小册子")), ("InBook", L10n.text("书籍章节")), ("InCollection", L10n.text("文集章节")), ("InProceedings", L10n.text("会议论文"))] }
    private var fields: [(String, String)] { [("all", L10n.text("全部字段")), ("author", L10n.text("作者")), ("title", L10n.text("标题")), ("journal", L10n.text("期刊")), ("year", L10n.text("年份"))] }

    var body: some View {
        VStack(spacing: 0) {
            VStack(spacing: 12) {
                HStack(spacing: 10) {
                    Text("BibCiTeX").font(.headline)
                    Spacer()
                    iconButton("refresh", L10n.text("刷新")) { Task { await model.reload() } }
                    iconButton("externalLink", L10n.text("显示窗口"), action: showMain)
                    iconButton("x", L10n.text("关闭"), action: dismiss)
                }
                HStack(spacing: 8) {
                    SVGIcon("search", size: 15).foregroundStyle(.secondary)
                    TextField(L10n.text("搜索文献"), text: $model.query)
                        .textFieldStyle(.plain).focused($searchFocused)
                        .padding(.trailing, 174)
                }
                .overlay(alignment: .trailing) {
                    HStack(spacing: 10) {
                        Divider().frame(height: 16).opacity(0.65)
                        Menu {
                            Picker(L10n.text("文献库"), selection: $model.libraryName) {
                                ForEach(model.libraries) { library in
                                    Text(library.name).tag(Optional(library.name))
                                }
                            }
                        } label: {
                            HStack(spacing: 6) {
                                SVGIcon("library", size: 13).foregroundStyle(.secondary)
                                Text(model.libraryName ?? L10n.text("暂无文献库"))
                                    .font(.system(size: 12, weight: .medium))
                                    .lineLimit(1).truncationMode(.middle)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                SVGIcon("chevronDown", size: 10).foregroundStyle(.secondary)
                            }.frame(width: 150, height: 20).contentShape(Rectangle())
                        }
                        .menuStyle(.borderlessButton).menuIndicator(.hidden)
                        .disabled(model.libraries.isEmpty)
                        .help(model.libraryName ?? L10n.text("文献库")).accessibilityLabel(L10n.text("文献库"))
                    }
                }
                .padding(.horizontal, 10).padding(.vertical, 8)
                .background(.quaternary.opacity(0.35), in: RoundedRectangle(cornerRadius: 8))
                .overlay {
                    RoundedRectangle(cornerRadius: 8)
                        .strokeBorder(searchFocused ? Color.accentColor.opacity(0.6) : Color.primary.opacity(0.1), lineWidth: 1)
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
            Divider()
            HStack(spacing: 0) {
                results.frame(maxWidth: .infinity)
                Divider()
                detail.frame(width: 282)
            }
            if let error = model.error {
                Divider()
                HStack {
                    SVGIcon("alert", size: 14)
                    Text(error).font(.caption).lineLimit(3)
                    Spacer()
                    iconButton("x", L10n.text("关闭")) { model.error = nil }
                }.foregroundStyle(.red).padding(10)
            }
        }
        .onAppear { searchFocused = true }
    }
    private var results: some View {
        ScrollViewReader { proxy in
            List(selection: $model.selection) {
                ForEach(model.references) { reference in
                    VStack(alignment: .leading, spacing: 5) {
                        Text(reference.citeKey).font(.system(.caption, design: .monospaced)).foregroundStyle(Color.accentColor)
                        MathChunkText(chunks: reference.title).lineLimit(2)
                        if !reference.author.isEmpty {
                            Text(reference.author.joined(separator: ", ")).font(.caption).foregroundStyle(.secondary).lineLimit(1)
                        }
                        Text([reference.displayType, reference.year.map(String.init) ?? "", reference.venueText]
                            .filter { !$0.isEmpty }.joined(separator: " · "))
                            .font(.caption2).foregroundStyle(.secondary).lineLimit(1)
                    }.padding(.vertical, 7).tag(reference.citeKey).id(reference.citeKey)
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
    @ViewBuilder private var detail: some View {
        if let reference = model.reference {
            VStack(alignment: .leading, spacing: 0) {
                ScrollView {
                    VStack(alignment: .leading, spacing: 12) {
                        Text(L10n.text("文献详情")).font(.caption).foregroundStyle(.secondary)
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
                Divider()
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
        Button { model.copy(value) } label: {
            HStack(spacing: 4) {
                SVGIcon(model.copied == value ? "check" : "copy", size: 12)
                Text(model.copied == value ? L10n.text("已复制") : label)
            }
        }.accessibilityLabel(label)
    }
    private func iconButton(_ icon: String, _ label: String, action: @escaping () -> Void) -> some View {
        Button(action: action) { SVGIcon(icon, size: 16).frame(width: 24, height: 24) }
            .buttonStyle(.plain).help(label).accessibilityLabel(label)
    }
}
