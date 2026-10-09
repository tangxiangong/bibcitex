import SwiftUI
import AppKit
import LaTeXSwiftUI

/// The independent global-hotkey search-and-paste surface.
struct HelperView: View {
    @AppStorage("language") private var language = "system"
    @ObservedObject var model: HelperViewModel
    var onHidePanel: () -> Void = {}
    @FocusState private var searchFocused: Bool

    var body: some View {
        VStack(spacing: 0) {
            header
            if let error = model.errorMessage {
                errorBar(error)
            }
            if showsList {
                PanelSeparator()
                content
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .background(PanelPalette.scrim)
        .background(PanelMaterial(radius: HelperMetrics.panelRadius, fallback: .hudWindow,
            fallbackState: .followsWindowActiveState))
        .clipShape(RoundedRectangle(cornerRadius: HelperMetrics.panelRadius, style: .continuous))
        .ignoresSafeArea()
        .scrollIndicators(.never)
        .onAppear { searchFocused = true }
        .onChange(of: model.focusRequest) { _ in searchFocused = true }
    }

    /// The idle surface is the search field alone; anything more is the oversized
    /// idle panel the compact helper exists to avoid.
    private var showsList: Bool {
        model.isLoading || model.isSelectingBibliography || !model.query.isEmpty
    }

    private var placeholder: String {
        model.isSelectingBibliography ? L10n.text("搜索或选择文献库") : L10n.text("搜索文献、作者、标题")
    }

    private var header: some View {
        HStack(spacing: 14) {
            NativeIcon("search", size: 18)
                .fontWeight(.medium)
                .foregroundStyle(.secondary)
            TextField(placeholder,
                text: Binding(get: { model.query }, set: { model.requestQuery($0) }))
                .font(.system(size: 20))
                .textFieldStyle(.plain)
                .focused($searchFocused)
                .accessibilityLabel(L10n.text("搜索"))
            if model.isSearching || model.isLoading {
                ProgressView().controlSize(.small).accessibilityLabel(L10n.text("正在搜索"))
            }
            libraryButton
        }
        // Aligns the search glyph with the row glyphs: list padding plus row padding.
        .padding(.horizontal, 18)
        .frame(height: HelperMetrics.header)
    }

    @ViewBuilder private var libraryButton: some View {
        if !model.isSelectingBibliography, let bib = model.currentBibliography {
            Button {
                model.startSelectMode()
                searchFocused = true
            } label: {
                HStack(spacing: 5) {
                    NativeIcon("library", size: 14)
                    Text(bib.name).lineLimit(1)
                    NativeIcon("chevronDown", size: 9)
                }
                .font(.system(size: 11, weight: .medium))
                .padding(.horizontal, 10).padding(.vertical, 6)
                .background(PanelPalette.controlSurface, in: Capsule())
            }
            .buttonStyle(.plain)
            .help(L10n.text("切换文献库 (Tab)"))
            .frame(maxWidth: 160)
            .accessibilityLabel(L10n.text("切换文献库"))
        }
    }

    /// Floating controls, no bar: Return (cross-app paste) and Tab as clickable
    /// hints, grouped in one capsule at the trailing edge.
    private var bottomBar: some View {
        HStack(spacing: 0) {
            Spacer(minLength: 0)
            HStack(spacing: 2) {
                HelperBarButton(action: { model.activateSelection(onSuccess: onHidePanel) }) {
                    barHint(L10n.text("粘贴引用键"), key: "↵")
                }
                .disabled(model.selectedReferenceIndex == nil)
                HelperBarButton(action: {
                    model.startSelectMode()
                    searchFocused = true
                }) {
                    barHint(L10n.text("切换文献库"), key: "⇥")
                }
            }
            .padding(4)
            .panelGlassCapsule()
        }
        .padding(.horizontal, 12)
        .frame(height: HelperMetrics.bottomBar)
    }

    private func barHint(_ title: String, key: String) -> some View {
        HStack(spacing: 6) {
            Text(title)
                .font(.system(size: 12, weight: .medium))
                .foregroundStyle(.secondary)
            HelperKeyCap(text: key)
        }
    }

    private func errorBar(_ error: String) -> some View {
        HStack(spacing: 7) {
            NativeIcon("alert", size: 14)
            Text(error).lineLimit(2)
            Spacer(minLength: 0)
            // A failed paste is the one case where the key must still be reachable:
            // the paste surface shows its keys as labels, so nothing else can hand
            // it over.
            if let key = model.failedPasteKey {
                Button(L10n.text("复制引用键")) { model.copyKey(key) }
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
                emptyState(L10n.text("未找到文献库，请先到主窗口添加文献库"), symbol: "library")
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
                        }.frame(maxWidth: .infinity).padding(8)
                    }
                    .scrollIndicators(.never)
                    .onChange(of: model.selectedBibliographyIndex) { index in
                        if let index { proxy.scrollTo(index) }
                    }
                }
            }
        } else if model.isSearching {
            ProgressView().frame(maxWidth: .infinity, minHeight: HelperMetrics.emptyRow)
        } else if model.searchResults.isEmpty {
            emptyState(L10n.text("未找到匹配记录"), symbol: "search")
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
                    }
                    .frame(maxWidth: .infinity).padding(8)
                }
                .scrollIndicators(.never)
                .mask(bottomDissolve)
                // An inset rather than an overlay: rows scroll beneath the controls,
                // while keyboard scrolling still lands the selection clear of them.
                .safeAreaInset(edge: .bottom, spacing: 0) {
                    if model.showsBottomBar { bottomBar }
                }
                .onChange(of: model.selectedReferenceIndex) { index in
                    if let index { proxy.scrollTo(index) }
                }
            }
        }
    }

    /// Ghosts rows passing beneath the floating controls. The ramp stays inside the
    /// bottom inset, so a list resting against its end is never faded.
    private var bottomDissolve: some View {
        GeometryReader { geo in
            let band = model.showsBottomBar ? min(HelperMetrics.bottomBar / max(geo.size.height, 1), 1) : 0
            LinearGradient(
                stops: [
                    .init(color: .black, location: 0),
                    .init(color: .black, location: 1 - band),
                    .init(color: .black.opacity(0.25), location: 1 - band / 2),
                    .init(color: .black.opacity(0), location: 1),
                ],
                startPoint: .top, endPoint: .bottom)
        }
        // Spans the scroll view's full frame, including the controls' inset.
        .ignoresSafeArea()
    }

    /// Same fields as the workbench sidebar: name first, path as the secondary line.
    private func libraryRow(_ bib: Bibliography, selected: Bool) -> some View {
        HStack(spacing: 10) {
            NativeIcon("library", size: 15).foregroundStyle(.secondary)
            VStack(alignment: .leading, spacing: 3) {
                Text(bib.name).font(.system(size: 13, weight: .semibold)).lineLimit(1)
                    .foregroundStyle(bib.available ? .primary : .secondary)
                Text(bib.path).font(.system(size: 11)).foregroundStyle(.secondary)
                    .lineLimit(1).truncationMode(.middle)
            }
            Spacer(minLength: 8)
            if !bib.available {
                Text(L10n.text("不可用")).font(.system(size: 11)).foregroundStyle(.secondary)
            } else if model.currentBibliography?.id == bib.id {
                NativeIcon("check", size: 13).foregroundStyle(Color.accentColor)
            }
        }
        .padding(.horizontal, 10).padding(.vertical, 8)
        .frame(maxWidth: .infinity)
        .frame(height: HelperMetrics.libraryRow)
        .contentShape(Rectangle())
        .background(selection(selected))
    }

    /// Same fields as a workbench list row — title, authors, type/year/venue — with
    /// the cite key lifted to the top right, where it is one click to copy and
    /// never competes with the title for the same line.
    private func referenceRow(_ reference: Reference, selected: Bool) -> some View {
        HStack(alignment: .top, spacing: 10) {
            NativeIcon("fileText", size: 15).foregroundStyle(.secondary).padding(.top, 2)
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
            citeKeyLabel(reference)
        }
        .padding(.horizontal, 10).padding(.vertical, 8)
        .frame(maxWidth: .infinity, alignment: .leading)
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

    private func selection(_ selected: Bool) -> some View {
        RoundedRectangle(cornerRadius: HelperMetrics.rowRadius, style: .continuous)
            .fill(selected ? PanelPalette.selection : .clear)
    }

    private func emptyState(_ text: String, symbol: String) -> some View {
        VStack(spacing: 8) {
            NativeIcon(symbol, size: 24)
            Text(text).font(.system(size: 12))
        }.foregroundStyle(.secondary).frame(maxWidth: .infinity, minHeight: HelperMetrics.emptyRow)
    }
}

/// A footer control: text and keycap that light up in a capsule under the pointer.
private struct HelperBarButton<Label: View>: View {
    let action: () -> Void
    @ViewBuilder let label: Label
    @State private var hovered = false
    @Environment(\.isEnabled) private var isEnabled

    var body: some View {
        Button(action: action) {
            label
                .padding(.horizontal, 10)
                .frame(height: HelperMetrics.barButton)
                .background(hovered && isEnabled ? PanelPalette.rowHover : .clear, in: Capsule())
                .contentShape(Capsule())
        }
        .buttonStyle(.plain)
        .opacity(isEnabled ? 1 : 0.5)
        .onHover { hovered = $0 }
        .animation(.easeOut(duration: 0.12), value: hovered)
    }
}

/// An outlined key, sized to sit beside a footer label.
private struct HelperKeyCap: View {
    let text: String

    var body: some View {
        Text(text)
            .font(.system(size: 11, weight: .medium))
            .foregroundStyle(.secondary)
            .frame(minWidth: HelperMetrics.keyCap, minHeight: HelperMetrics.keyCap)
            .overlay(
                RoundedRectangle(cornerRadius: 6, style: .continuous)
                    .strokeBorder(PanelPalette.border, lineWidth: 1)
            )
    }
}

struct MathChunkText: View {
    @AppStorage("language") private var language = "system"
    let chunks: [TextChunk]
    var body: some View {
        if chunks.isEmpty {
            Text(L10n.text("暂无标题")).foregroundStyle(.secondary)
        } else if !chunks.contains(where: { $0.kind == .math }) {
            Text(verbatim: chunks.map(\.text).joined())
        } else {
            LaTeX(chunks.map { $0.kind == .math ? "$\($0.text)$" : $0.text }.joined())
                .parsingMode(.onlyEquations)
                .ignoreStringFormatting()
                .script(.custom(1.1))

        }
    }
}
