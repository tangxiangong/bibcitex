import SwiftUI
import AppKit
import LaTeXSwiftUI

struct HelperView: View {
    @ObservedObject var model: HelperViewModel
    var onHidePanel: () -> Void = {}
    @FocusState private var searchFocused: Bool
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 14) {
                SVGIcon("search", size: 23)
                    .font(.system(size: 22, weight: .regular))
                    .foregroundStyle(.secondary)
                TextField(model.isSelectingBibliography ? "搜索或选择文献库" : "搜索文献、作者、标题",
                    text: Binding(get: { model.query }, set: { model.requestQuery($0) }))
                    .font(.system(size: 22))
                    .textFieldStyle(.plain)
                    .focused($searchFocused)
                    .accessibilityLabel("搜索")
                if model.isSearching || model.isLoading {
                    ProgressView().controlSize(.small)
                }
                if !model.isSelectingBibliography, let bib = model.currentBibliography {
                    Button {
                        model.startSelectMode()
                        searchFocused = true
                    } label: {
                        HStack(spacing: 5) {
                            SVGIcon("library", size: 20)
                            Text(bib.name).lineLimit(1)
                            SVGIcon("chevronDown", size: 10).font(.system(size: 9, weight: .semibold))
                        }
                        .font(.system(size: 11, weight: .medium))
                        .padding(.horizontal, 10).padding(.vertical, 6)
                        .background(.primary.opacity(0.06), in: Capsule())
                    }
                    .buttonStyle(.plain)
                    .help("切换文献库 (Tab)")
                    .frame(maxWidth: 160)
                }
            }
            .padding(.horizontal, 22)
            .frame(height: 56)

            if let error = model.errorMessage {
                HStack(spacing: 8) {
                    SVGIcon("alert", size: 16)
                    Text(error).lineLimit(2)
                    Spacer(minLength: 0)
                    if model.failedPasteKey != nil {
                        Button("复制", action: model.copyFailedKeyAgain)
                    }
                }
                .font(.system(size: 12))
                .foregroundStyle(.red)
                .padding(12)
            }

            if model.isLoading || model.isSelectingBibliography || !model.query.isEmpty {
                Divider().opacity(0.5)
                content
            }

        }
        .ignoresSafeArea()
        .onAppear { searchFocused = true }
        .onChange(of: model.focusRequest) { _ in searchFocused = true }
    }

    @ViewBuilder private var content: some View {
        if model.isLoading {
            ProgressView().frame(maxWidth: .infinity, minHeight: 112)
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
                                    HStack(spacing: 14) {
                                        SVGIcon("library", size: 20)
                                            .font(.system(size: 23)).foregroundStyle(.secondary)
                                            .frame(width: 32)
                                        VStack(alignment: .leading, spacing: 5) {
                                            Text(bib.name).font(.system(size: 14, weight: .semibold)).lineLimit(1)
                                            Text(bib.path).font(.system(size: 11)).foregroundStyle(.secondary).lineLimit(1).truncationMode(.middle)
                                        }
                                        Spacer(minLength: 8)
                                        if let description = bib.descriptionText, !description.isEmpty {
                                            Text(description).font(.system(size: 11)).foregroundStyle(.secondary).lineLimit(1)
                                        }
                                        if model.currentBibliography?.id == bib.id {
                                            SVGIcon("check", size: 14).foregroundStyle(Color.accentColor)
                                        }
                                    }
                                    .padding(.horizontal, 14).frame(height: 64)
                                    .contentShape(Rectangle())
                                    .background(selection(model.selectedBibliographyIndex == index))
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
            ProgressView().frame(maxWidth: .infinity, minHeight: 112)
        } else if model.searchResults.isEmpty {
            emptyState("未找到匹配记录", symbol: "search")
        } else {
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: 2) {
                        ForEach(Array(model.searchResults.enumerated()), id: \.element.citeKey) { index, reference in
                            Button {
                                model.copyAndPaste(reference, at: index, onSuccess: onHidePanel)
                            } label: {
                                HStack(alignment: .center, spacing: 14) {
                                    SVGIcon("fileText", size: 24)
                                        .font(.system(size: 23)).foregroundStyle(.secondary).frame(width: 32)
                                    VStack(alignment: .leading, spacing: 5) {
                                        MathChunkText(chunks: reference.title)
                                            .font(.system(size: 14, weight: .medium)).lineLimit(2)
                                        HStack(spacing: 8) {
                                            Text(reference.displayType)
                                            Text(reference.citeKey).foregroundStyle(Color.accentColor)
                                            Text(reference.author.prefix(2).joined(separator: ", "))
                                        }.font(.system(size: 11)).foregroundStyle(.secondary).lineLimit(1)
                                        Text(reference.venueText).font(.system(size: 11)).foregroundStyle(.secondary).lineLimit(1)
                                    }
                                    Spacer(minLength: 0)
                                    if let year = reference.year {
                                        Text(String(year)).font(.system(size: 11)).foregroundStyle(.secondary)
                                    }
                                }
                                .padding(.horizontal, 14).frame(height: 92)
                                .contentShape(Rectangle())
                                .background(selection(model.selectedReferenceIndex == index))
                            }
                            .buttonStyle(.plain)
                            .id(index)
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

    private func selection(_ selected: Bool) -> some View {
        RoundedRectangle(cornerRadius: 9)
            .fill(selected ? Color.accentColor.opacity(colorScheme == .dark ? 0.25 : 0.13) : .clear)
    }

    private func emptyState(_ text: String, symbol: String) -> some View {
        VStack(spacing: 10) {
            SVGIcon(symbol, size: 28).font(.system(size: 26, weight: .light))
            Text(text).font(.system(size: 12))
        }.foregroundStyle(.secondary).frame(maxWidth: .infinity, minHeight: 112)
    }
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
