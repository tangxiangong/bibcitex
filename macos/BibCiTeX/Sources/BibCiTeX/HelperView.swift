import SwiftUI
import AppKit
import LaTeXSwiftUI

/// The helper surface, hosted by both the tray window and the Cmd+Shift+K panel:
/// the workbench's centre column — active library, search field, reference rows —
/// compressed to menu-bar scale. Rows carry the same fields as `WorkbenchView`'s
/// list, and the inspector is deliberately left out: finding a record and taking
/// its cite key is the whole job here.
///
/// The two surfaces differ only in what activating a row does, which the model
/// holds as `mode`: the panel pastes into the app that was frontmost, the tray
/// window copies and stays open. Every row also carries its own copy button, so a
/// cite key is one click away on either surface regardless of that mode.
struct HelperView: View {
    @ObservedObject var model: HelperViewModel
    var onHidePanel: () -> Void = {}
    @FocusState private var searchFocused: Bool
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        VStack(spacing: 0) {
            header
            if let error = model.errorMessage {
                errorBar(error)
            }
            if showsList {
                Divider().opacity(0.5)
                content
            }
        }
        .ignoresSafeArea()
        .onAppear { searchFocused = true }
        .onChange(of: model.focusRequest) { _ in searchFocused = true }
    }

    /// The idle surface is the search field alone; anything more is the oversized
    /// idle panel the compact helper exists to avoid.
    private var showsList: Bool {
        model.isLoading || model.isSelectingBibliography || !model.query.isEmpty
    }

    private var placeholder: String {
        model.isSelectingBibliography ? "搜索或选择文献库" : "搜索文献、作者、标题"
    }

    private var header: some View {
        HStack(spacing: 14) {
            SVGIcon("search", size: 22)
                .foregroundStyle(.secondary)
            TextField(placeholder,
                text: Binding(get: { model.query }, set: { model.requestQuery($0) }))
                .font(.system(size: 22))
                .textFieldStyle(.plain)
                .focused($searchFocused)
                .accessibilityLabel("搜索")
            if model.isSearching || model.isLoading {
                ProgressView().controlSize(.small).accessibilityLabel("正在搜索")
            }
            libraryButton
        }
        .padding(.horizontal, 22)
        .frame(height: HelperMetrics.header)
    }

    @ViewBuilder private var libraryButton: some View {
        if !model.isSelectingBibliography, let bib = model.currentBibliography {
            Button {
                model.startSelectMode()
                searchFocused = true
            } label: {
                HStack(spacing: 5) {
                    SVGIcon("library", size: 14)
                    Text(bib.name).lineLimit(1)
                    SVGIcon("chevronDown", size: 9)
                }
                .font(.system(size: 11, weight: .medium))
                .padding(.horizontal, 10).padding(.vertical, 6)
                .background(.primary.opacity(0.06), in: Capsule())
            }
            .buttonStyle(.plain)
            .help("切换文献库 (Tab)")
            .frame(maxWidth: 160)
            .accessibilityLabel("切换文献库")
        }
    }

    private func errorBar(_ error: String) -> some View {
        HStack(spacing: 7) {
            SVGIcon("alert", size: 14)
            Text(error).lineLimit(2)
            Spacer(minLength: 0)
            // A failed paste is the one case where the key must still be reachable:
            // the paste surface shows its keys as labels, so nothing else can hand
            // it over. The tray window's rows are already clickable, so it skips this.
            if let key = model.failedPasteKey {
                Button("复制引用键") { model.copyKey(key) }
                    .buttonStyle(.plain)
                    .foregroundStyle(Color.accentColor)
            }
        }
        .font(.system(size: 11))
        .foregroundStyle(.red)
        .padding(.horizontal, 12)
        .padding(.vertical, 7)
    }

    @ViewBuilder private var content: some View {
        if model.isLoading {
            ProgressView().frame(maxWidth: .infinity, minHeight: HelperMetrics.emptyRow)
        } else if model.isSelectingBibliography {
            if model.filteredBibliographies.isEmpty {
                emptyState("未找到文献库，请先到主窗口添加文献库", symbol: "library")
            } else {
                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(spacing: 2) {
                            ForEach(Array(model.filteredBibliographies.enumerated()), id: \.element.id) { index, bib in
                                Button {
                                    model.selectBibliography(bib, at: index)
                                    searchFocused = true
                                } label: {
                                    libraryRow(bib, selected: model.selectedBibliographyIndex == index)
                                }
                                .buttonStyle(.plain)
                                .id(index)
                                .accessibilityAddTraits(model.selectedBibliographyIndex == index ? .isSelected : [])
                            }
                        }.padding(8)
                    }
                    .onChange(of: model.selectedBibliographyIndex) { index in
                        if let index { proxy.scrollTo(index) }
                    }
                }
            }
        } else if model.isSearching {
            ProgressView().frame(maxWidth: .infinity, minHeight: HelperMetrics.emptyRow)
        } else if model.searchResults.isEmpty {
            emptyState("未找到匹配记录", symbol: "search")
        } else {
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: 2) {
                        ForEach(Array(model.searchResults.enumerated()), id: \.element.citeKey) { index, reference in
                            // The row is a tap target rather than a Button, so the
                            // copy button inside it can take its own clicks.
                            referenceRow(reference, selected: model.selectedReferenceIndex == index)
                                .contentShape(Rectangle())
                                .onTapGesture {
                                    model.activate(reference, at: index, onSuccess: onHidePanel)
                                }
                                .id(index)
                                .accessibilityElement(children: .contain)
                                .accessibilityAddTraits(model.selectedReferenceIndex == index ? .isSelected : [])
                        }
                    }.padding(8)
                }
                .onChange(of: model.selectedReferenceIndex) { index in
                    if let index { proxy.scrollTo(index) }
                }
            }
        }
    }

    /// Same fields as the workbench sidebar: name first, path as the secondary line.
    private func libraryRow(_ bib: Bibliography, selected: Bool) -> some View {
        HStack(spacing: 10) {
            SVGIcon("library", size: 15).foregroundStyle(.secondary)
            VStack(alignment: .leading, spacing: 3) {
                Text(bib.name).font(.system(size: 13, weight: .semibold)).lineLimit(1)
                Text(bib.path).font(.system(size: 11)).foregroundStyle(.secondary)
                    .lineLimit(1).truncationMode(.middle)
            }
            Spacer(minLength: 8)
            if model.currentBibliography?.id == bib.id {
                SVGIcon("check", size: 13).foregroundStyle(Color.accentColor)
            }
        }
        .padding(.horizontal, 10).padding(.vertical, 8)
        .frame(height: HelperMetrics.libraryRow)
        .contentShape(Rectangle())
        .background(selection(selected))
    }

    /// Same fields as a workbench list row — title, authors, type/year/venue — with
    /// the cite key lifted to the top right, where it is one click to copy and
    /// never competes with the title for the same line.
    private func referenceRow(_ reference: Reference, selected: Bool) -> some View {
        HStack(alignment: .top, spacing: 10) {
            SVGIcon("fileText", size: 15).foregroundStyle(.secondary).padding(.top, 2)
            VStack(alignment: .leading, spacing: 3) {
                MathChunkText(chunks: reference.title)
                    .font(.system(size: 13, weight: .medium)).lineLimit(2)
                if !reference.author.isEmpty {
                    Text(reference.author.joined(separator: ", "))
                        .font(.system(size: 11)).foregroundStyle(.secondary).lineLimit(1)
                }
                HStack(spacing: 5) {
                    Text(reference.displayType)
                    if let year = reference.year {
                        Text("·"); Text(String(year))
                    }
                    if !reference.venueText.isEmpty {
                        Text("·"); Text(reference.venueText).lineLimit(1)
                    }
                }
                .font(.system(size: 11)).foregroundStyle(.secondary)
                .lineLimit(1)
            }
            Spacer(minLength: 6)
            // Only the tray window's cite key is a control: that surface exists to
            // collect keys, so clicking one copies it. The hotkey panel pastes the
            // whole row into another app, where a second click target would only
            // compete with the paste the row already performs.
            if model.mode == .copy {
                citeKeyButton(reference)
            } else {
                citeKeyLabel(reference)
            }
        }
        .padding(.horizontal, 10).padding(.vertical, 8)
        .frame(height: HelperMetrics.referenceRow, alignment: .top)
        .background(selection(selected))
    }

    /// The same key in the same place, shown but not clickable: on the paste
    /// surface it is information about the row, not an action of its own.
    private func citeKeyLabel(_ reference: Reference) -> some View {
        Text(reference.citeKey)
            .font(.system(size: 11, design: .monospaced))
            .foregroundStyle(Color.accentColor)
            .lineLimit(1)
    }

    /// The row's own copy action, pinned to the top right. It confirms in place —
    /// a check and 已复制 for a moment — so the copy is answered without an alert,
    /// and it never invokes the paste path the row body may run.
    private func citeKeyButton(_ reference: Reference) -> some View {
        let copied = model.copiedKey == reference.citeKey
        return Button {
            model.copy(reference)
        } label: {
            HStack(spacing: 4) {
                SVGIcon(copied ? "check" : "copy", size: 10)
                Text(copied ? "已复制" : reference.citeKey)
                    .font(.system(size: 11, design: copied ? .default : .monospaced))
                    .lineLimit(1)
            }
            .foregroundStyle(copied ? Color.green : Color.accentColor)
            .padding(.horizontal, 7).padding(.vertical, 4)
            .background(.primary.opacity(0.06), in: Capsule())
        }
        .buttonStyle(.plain)
        .help("复制引用键")
        .accessibilityLabel("复制引用键 \(reference.citeKey)")
        .frame(maxWidth: 150)
    }

    private func selection(_ selected: Bool) -> some View {
        RoundedRectangle(cornerRadius: 7)
            .fill(selected ? Color.accentColor.opacity(colorScheme == .dark ? 0.25 : 0.13) : .clear)
    }

    private func emptyState(_ text: String, symbol: String) -> some View {
        VStack(spacing: 8) {
            SVGIcon(symbol, size: 24)
            Text(text).font(.system(size: 12))
        }.foregroundStyle(.secondary).frame(maxWidth: .infinity, minHeight: HelperMetrics.emptyRow)
    }
}

/// One source of truth for the helper surface metrics: the view draws these
/// heights and `HelperViewModel` sizes the panel from them.
enum HelperMetrics {
    static let header: CGFloat = 56
    /// Title (two lines), authors and the metadata line, plus the row padding.
    static let referenceRow: CGFloat = 80
    static let libraryRow: CGFloat = 56
    static let listPadding: CGFloat = 16
    static let emptyRow: CGFloat = 112
    static let errorBar: CGFloat = 40
}

struct MathChunkText: View {
    let chunks: [TextChunk]
    var body: some View {
        if chunks.isEmpty {
            Text("暂无标题").foregroundStyle(.secondary)
        } else if !chunks.contains(where: { $0.kind == .math }) {
            Text(verbatim: chunks.map(\.text).joined())
        } else {
            LaTeX(chunks.map { $0.kind == .math ? "$\($0.text)$" : $0.text }.joined())
                .parsingMode(.onlyEquations)
                .ignoreStringFormatting()
        }
    }
}
