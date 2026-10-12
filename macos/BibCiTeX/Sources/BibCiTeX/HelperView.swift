import SwiftUI
import AppKit
import LaTeXSwiftUI

/// The independent global-hotkey search-and-paste surface.
struct HelperView: View {
    @AppStorage("language") private var language = "system"
    @ObservedObject var model: HelperViewModel
    var onHidePanel: () -> Void = {}
    @FocusState private var searchFocused: Bool

    @State private var taggingReference: TagAssignmentTarget?
    var body: some View {
        VStack(spacing: 0) {
            header
            if let error = model.errorMessage {
                errorBar(error)
            }
            QueryDiagnostics(analysis: model.queryAnalysis, compact: true)
            if showsList && model.candidateKind == nil {
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
        .popover(item: $taggingReference) { reference in TagManagerView(citeKey: reference.citeKey, libraryUID: reference.libraryUID, recordUID: reference.recordUID) }
        .sheet(isPresented: $model.showTagManager) { TagManagerView(path: "", citeKey: "") }
        .onReceive(NotificationCenter.default.publisher(for: .searchTagsChanged)) { _ in model.tagsChanged() }
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
            ScrollViewReader { proxy in
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 7) {
                        ForEach(model.capsules) { capsule in
                            HStack(spacing: 5) {
                                Button { model.editCapsule(capsule.id) } label: { Text(capsule.text).lineLimit(1) }
                                    .buttonStyle(.plain).help(L10n.text("搜索条件"))
                                Button { model.removeCapsule(capsule.id) } label: { NativeIcon("x", size: 9) }
                                    .buttonStyle(.plain).accessibilityLabel(L10n.text("移除"))
                            }.font(.system(size: 12, weight: .medium))
                                .padding(.horizontal, 9).padding(.vertical, 6)
                                .background(PanelPalette.controlSurface, in: Capsule())
                                .contextMenu { Button(L10n.text("搜索条件")) { model.expandCapsulesToText() } }
                        }
                        SearchInput(text: model.inputText, placeholder: model.capsules.isEmpty && model.candidateKind == nil ? placeholder : "", fontSize: 20,
                            focusRequest: model.focusRequest, caretAtEndOnFocus: true, isEditable: model.candidateKind == nil,
                            changed: { model.editInput($0, start: $1, end: $2, replacement: $3, full: $4) }, compositionChanged: model.suspendForComposition, triggerInserted: { symbol, before, after in
                                guard !model.isSelectingBibliography, !model.editingCondition else { return false }
                                model.beginCandidates(symbol, nativeBefore: before, nativeAfter: after); return true
                            }, deleteBackwardAtStart: model.deletePreviousCapsule)
                            .frame(width: model.candidateKind != nil ? (model.inputText.isEmpty ? 0 : min(CGFloat(model.inputText.count) * 11 + 12, 430)) : max(model.capsules.isEmpty ? 190 : 90, min(CGFloat(model.inputText.count) * 11 + 24, 430)), height: 26)
                            .clipped().opacity(model.candidateKind != nil && model.inputText.isEmpty ? 0 : 1)
                            .id("helper-tail")
                        if let symbol = model.candidateKind {
                            Text(symbol).font(.system(size: 16, weight: .medium)).foregroundStyle(.secondary)
                            SearchInput(text: model.editingOptionalFieldValue ? model.candidateValue : model.candidateFilter,
                                placeholder: L10n.text(model.editingOptionalFieldValue ? "字段值（可选）" : "筛选候选"), fontSize: 15,
                                focusRequest: model.candidateFocusRequest, caretAtEndOnFocus: true,
                                changed: { text, _, _, _, _ in
                                    if model.editingOptionalFieldValue { model.candidateValue = text }
                                    else { model.candidateFilter = text; model.candidateSelection = 0 }
                                }, move: { delta in
                                    if !model.editingOptionalFieldValue { model.moveCandidate(delta) }
                                })
                                .frame(width: 140, height: 21)
                                .padding(.horizontal, 8).padding(.vertical, 5)
                                .background(PanelPalette.controlSurface, in: RoundedRectangle(cornerRadius: 7))
                                .overlay(RoundedRectangle(cornerRadius: 7).stroke(Color.accentColor, lineWidth: 1))
                                .background(HelperCandidateAnchor(model: model))
                                .id("helper-condition")
                        }
                    }
                }.frame(maxWidth: .infinity, alignment: .leading)
                    .onChange(of: model.focusRequest) { _ in proxy.scrollTo("helper-tail", anchor: .trailing) }
                    .onChange(of: model.candidateFocusRequest) { _ in if model.candidateKind != nil { proxy.scrollTo("helper-condition", anchor: .trailing) } }
            }
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
                .disabled(model.selectedReferenceIndex == nil || model.pasteSelection?.isEmpty == true)
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
                            ForEach(Array(model.filteredBibliographies.enumerated()), id: \.element.name) { index, bib in
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
                            HStack(spacing: 6) {
                                referenceRow(reference, selected: model.isSelectedForPaste(reference, at: index))
                            }
                                .contentShape(Rectangle())
                                .onTapGesture {
                                    let modifiers = NSEvent.modifierFlags
                                    if modifiers.contains(.command) || modifiers.contains(.shift) {
                                        model.selectForPaste(at: index, extending: modifiers.contains(.shift), toggling: modifiers.contains(.command))
                                    } else { model.activate(reference, at: index, onSuccess: onHidePanel) }
                                }
                                .id(index)
                                .accessibilityElement(children: .contain)
                                .accessibilityAddTraits(.isButton)
                                .accessibilityHint(Text(L10n.text("粘贴引用键")))
                                .accessibilityAction {
                                    model.activate(reference, at: index, onSuccess: onHidePanel)
                                }
                                .accessibilityAddTraits(model.isSelectedForPaste(reference, at: index) ? .isSelected : [])
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
                MathChunkText(chunks: reference.title, highlightRanges: SearchHighlight.ranges(model.matchRanges[reference.citeKey] ?? [], field: "title"))
                    .help(model.matchEvidence[reference.citeKey] ?? "")                    .font(.system(size: 13, weight: .medium)).lineLimit(2)
                if !reference.author.isEmpty {
                    Text(SearchHighlight.authors(reference.author, matches: model.matchRanges[reference.citeKey] ?? []))
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
            VStack(alignment: .trailing, spacing: 6) {
                citeKeyLabel(reference)
                TagChips(tags: reference.tags, edit: { taggingReference = TagAssignmentTarget(libraryUID: reference.libraryUID, recordUID: reference.id, citeKey: reference.citeKey) })
            }
        }
        .padding(.horizontal, 10).padding(.vertical, 8)
        .frame(maxWidth: .infinity, alignment: .leading)
        .frame(height: HelperMetrics.referenceRow, alignment: .top)
        .background(selection(selected))
        .contextMenu {
            Button(L10n.text("标签")) { taggingReference = TagAssignmentTarget(libraryUID: reference.libraryUID, recordUID: reference.id, citeKey: reference.citeKey) }
        }
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
    var highlightRanges: [Range<Int>] = []
    var body: some View {
        if chunks.isEmpty {
            Text(L10n.text("暂无标题")).foregroundStyle(.secondary)
        } else if !chunks.contains(where: { $0.kind == .math }) {
            Text(SearchHighlight.attributed(chunks.map(\.text).joined(), ranges: highlightRanges))
        } else {
            let projection = SearchHighlight.latexProjection(chunks, ranges: highlightRanges)
            LaTeX(projection.text)
                .parsingMode(.onlyEquations)
                .ignoreStringFormatting()
                .script(.custom(1.1))
                .decorateComponents { original, range, isEquation, rendered in
                    let local = projection.ranges.compactMap { match -> Range<Int>? in
                        let lower = max(range.lowerBound, match.lowerBound), upper = min(range.upperBound, match.upperBound)
                        return lower < upper ? (lower - range.lowerBound)..<(upper - range.lowerBound) : nil
                    }
                    guard !local.isEmpty else { return rendered }
                    // Keep the renderer's complete image and baseline; decorate
                    // the atomic math run without splitting or rewriting TeX.
                    if isEquation { return rendered.underline(true, color: .accentColor) }
                    return Text(SearchHighlight.attributed(original, ranges: local))
                }

        }
    }
}

/// The candidate list is hosted in a real child window, never in the result stack.
/// Showing it preserves the main editor's focus until Tab or a click selects its filter.
private struct HelperCandidateAnchor: NSViewRepresentable {
    @ObservedObject var model: HelperViewModel
    func makeCoordinator() -> Coordinator { Coordinator(model) }
    func makeNSView(context: Context) -> NSView { NSView() }
    func updateNSView(_ view: NSView, context: Context) {
        let shouldShow = model.candidateKind != nil && model.candidateField == nil && !model.showTagManager
        DispatchQueue.main.async { [weak view, weak coordinator = context.coordinator] in
            guard let view else { return }; coordinator?.update(anchor: view, visible: shouldShow)
        }
    }
    static func dismantleNSView(_ view: NSView, coordinator: Coordinator) { coordinator.close() }
    @MainActor final class Coordinator: NSObject, NSWindowDelegate {
        let model: HelperViewModel
        var panel: CandidatePanel?
        init(_ model: HelperViewModel) { self.model = model }
        func update(anchor: NSView, visible: Bool) {
            guard visible, model.candidateKind != nil, model.candidateField == nil, !model.showTagManager, let parent = anchor.window, parent.isVisible else { close(); return }
            let child: CandidatePanel
            if let panel { child = panel }
            else {
                child = CandidatePanel(contentRect: NSRect(x: 0, y: 0, width: model.candidatePopupSize.width, height: model.candidatePopupSize.height), styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: false)
                child.isOpaque = false; child.backgroundColor = .clear; child.hasShadow = true
                child.isFloatingPanel = true; child.hidesOnDeactivate = false; child.level = parent.level
                child.contentView = NSHostingView(rootView: candidateSnapshot())
                child.delegate = self; panel = child
            }
            let local = anchor.convert(anchor.bounds, to: nil)
            let rect = parent.convertToScreen(local)
            let screen = parent.screen?.visibleFrame ?? NSScreen.main?.visibleFrame ?? parent.frame
            let size = CGSize(width: min(model.candidatePopupSize.width, screen.width), height: min(model.candidatePopupSize.height, screen.height))
            let x = min(max(rect.minX, screen.minX), screen.maxX - size.width)
            let below = rect.minY - size.height - 8
            let y = below >= screen.minY ? below : min(rect.maxY + 8, screen.maxY - size.height)
            // Update rows, selection, and geometry from the same committed snapshot.
            // A separately observed hosting tree must not resize around stale rows.
            if let hosting = child.contentView as? NSHostingView<HelperFloatingCandidates> {
                hosting.rootView = candidateSnapshot()
                hosting.frame = NSRect(origin: .zero, size: size)
                hosting.layoutSubtreeIfNeeded()
            }
            child.setFrame(NSRect(origin: CGPoint(x: x, y: y), size: size), display: true)
            if child.parent !== parent { parent.addChildWindow(child, ordered: .above) }
            child.orderFront(nil)

        }
        private func candidateSnapshot() -> HelperFloatingCandidates {
            let rows = model.candidates
            let selected = rows.indices.contains(model.candidateSelection) ? rows[model.candidateSelection].id : nil
            return HelperFloatingCandidates(rows: rows, selectedID: selected,
                filterIdentity: (model.candidateKind ?? "") + "\u{1f}" + model.candidateFilter,
                size: model.candidatePopupSize) { [weak model] id in
                    guard let model, let index = model.candidates.firstIndex(where: { $0.id == id }) else { return }
                    model.candidateSelection = index; model.confirmCandidate()
                }
        }
        func close() {
            guard let panel else { return }
            let parent = panel.parent
            let wasKey = panel.isKeyWindow
            parent?.removeChildWindow(panel); panel.orderOut(nil)
            self.panel = nil
            if wasKey, parent?.isVisible == true { parent?.makeKey() }
        }
        func windowDidResignKey(_ notification: Notification) {
            DispatchQueue.main.async { [weak self] in
                guard let self, let panel = self.panel, let parent = panel.parent,
                      !parent.isKeyWindow, !panel.isKeyWindow else { return }
                HelperPanelController.shared.hidePanel()
            }
        }
    }
    @MainActor final class CandidatePanel: NSPanel {
        override var canBecomeKey: Bool { true }
        override var canBecomeMain: Bool { false }
    }
}

private struct HelperFloatingCandidates: View {
    let rows: [HelperViewModel.Candidate]
    let selectedID: String?
    let filterIdentity: String
    let size: CGSize
    let select: (String) -> Void
    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            if rows.isEmpty { Text(L10n.text("无候选")).foregroundStyle(.secondary) }
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach(rows, id: \.id) { candidate in
                            Button { select(candidate.id) } label: {
                                HStack { Text(candidate.name).lineLimit(1); Spacer(minLength: 0) }.font(.system(size: 13)).padding(.horizontal, 7).frame(height: 28)
                                    .background(selectedID == candidate.id ? Color.accentColor.opacity(0.18) : Color.clear, in: RoundedRectangle(cornerRadius: 6))
                            }.buttonStyle(.plain).id(candidate.id)
                                .accessibilityAddTraits(selectedID == candidate.id ? .isSelected : [])
                        }
                    }
                }.scrollIndicators(.never)
                    .onChange(of: selectedID) { if let id = $0 { proxy.scrollTo(id) } }
                    .onChange(of: rows.map(\.id)) { _ in if let first = rows.first { proxy.scrollTo(first.id, anchor: .top) } }
            }.id(filterIdentity)
        }.padding(6).frame(width: size.width, height: size.height)
            .background(PanelMaterial(radius: 12, fallback: .popover))
            .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}
