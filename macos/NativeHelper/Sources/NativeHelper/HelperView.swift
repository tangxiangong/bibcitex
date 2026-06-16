import SwiftUI
import AppKit

#if canImport(LaTeXSwiftUI)
import LaTeXSwiftUI
#endif

struct HelperView: View {
    @ObservedObject var model: HelperViewModel

    var onHidePanel: () -> Void = {}

    var body: some View {
        ZStack {
            background

            VStack(spacing: 0) {
                inputRow

                if let error = model.errorMessage {
                    errorBar(error)
                        .padding(.horizontal, 12)
                        .padding(.top, 8)
                }

                Divider()
                    .padding(.top, 8)

                content
            }
            .padding(.bottom, 8)
        }
        .preferredColorScheme(model.theme.isDark ? .dark : .light)
        .onAppear {
            model.loadState()
            model.recalcHeight()
        }
    }

    private var background: some View {
        ZStack {
            BlurView(material: model.theme.isDark ? .systemUltraThinMaterial : .systemThinMaterial)
                .ignoresSafeArea()
            Color.clear
        }
    }

    private var inputRow: some View {
        HStack(spacing: 10) {
            Image(systemName: "magnifyingglass")
                .foregroundStyle(.secondary)

            TextField(
                model.isSelectingBibliography ? "搜索或选择文献库" : "搜索文献、作者、标题",
                text: Binding(
                    get: { model.query },
                    set: { model.updateQuery($0) },
                ),
            )
            .textFieldStyle(.plain)

            if !model.isSelectingBibliography, let bib = model.currentBibliography {
                Button(action: model.startSelectMode) {
                    Text(bib.name)
                        .lineLimit(1)
                        .truncationMode(.tail)
                        .padding(.horizontal, 12)
                        .padding(.vertical, 6)
                        .background(.ultraThinMaterial, in: Capsule())
                }
                .buttonStyle(.plain)
            }
        }
        .padding(.horizontal, 12)
        .padding(.top, 10)
        .padding(.bottom, 10)
    }

    private var content: some View {
        Group {
            if model.isSelectingBibliography {
                bibliographyList
            } else {
                referenceList
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .padding(.horizontal, 8)
    }

    private var bibliographyList: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                ForEach(Array(model.filteredBibliographies.enumerated()), id: \.element.id) { index, bibliography in
                    bibliographyRow(bibliography, index: index)
                    Divider()
                }

                if model.filteredBibliographies.isEmpty {
                    Text("未找到文献库，请先到主窗口添加文献库")
                        .font(.callout)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, minHeight: 56)
                }
            }
        }
        .scrollIndicators(.hidden)
    }

    @ViewBuilder
    private func bibliographyRow(_ bibliography: NativeHelperBibliography, index: Int) -> some View {
        HStack(spacing: 10) {
            VStack(alignment: .leading, spacing: 4) {
                Text(bibliography.name)
                    .font(.system(size: 16, weight: .semibold))
                    .lineLimit(1)

                Text(bibliography.path)
                    .lineLimit(1)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            if let descriptionText = bibliography.descriptionText, !descriptionText.isEmpty {
                Text(descriptionText)
                    .lineLimit(1)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(14)
        .contentShape(Rectangle())
        .frame(minHeight: 64)
        .background(
            model.selectedBibliographyIndex == index
                ? Color.accentColor.opacity(0.2)
                : Color.clear,
        )
        .onTapGesture {
            model.selectedBibliographyIndex = index
            model.selectBibliography(bibliography)
        }
    }

    private var referenceList: some View {
        Group {
            if model.query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                Text("请输入关键字搜索文献")
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, minHeight: 120)
            } else if model.searchResults.isEmpty {
                Text("未找到匹配记录")
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, minHeight: 120)
            } else {
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach(Array(model.searchResults.enumerated()), id: \.element.id) { index, reference in
                            referenceRow(reference, index: index)
                            Divider()
                        }
                    }
                }
                .scrollIndicators(.hidden)
            }
        }
    }

    @ViewBuilder
    private func referenceRow(_ reference: NativeHelperReference, index: Int) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .top, spacing: 6) {
                Text(reference.displayType)
                    .font(.caption)
                    .fontWeight(.semibold)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 2)
                    .background(.mint.opacity(0.15), in: Capsule())

                Text(reference.citeKey)
                    .font(.caption)
                    .foregroundStyle(.secondary)

                Spacer()
            }

            MathChunkText(chunks: reference.title)
                .font(.system(size: 16, weight: .semibold))

            if !reference.venueText.isEmpty {
                Text(reference.venueText)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }

            if let year = reference.year {
                Text("\(year)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(12)
        .contentShape(Rectangle())
        .frame(minHeight: 92)
        .background(
            model.selectedReferenceIndex == index
                ? Color.accentColor.opacity(0.2)
                : Color.clear,
        )
        .onTapGesture {
            model.selectedReferenceIndex = index
            model.copyAndPaste(reference, onSuccess: onHidePanel)
        }
    }

    private func errorBar(_ text: String) -> some View {
        HStack {
            Image(systemName: "exclamationmark.triangle")
            Text(text)
                .lineLimit(1)
                .truncationMode(.tail)
            Spacer()
            if let failed = model.failedPasteKey {
                Button("复制") {
                    model.copyFailedKeyAgain()
                }
                .buttonStyle(.plain)
                .font(.caption)
            }
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        .font(.caption)
        .background(.red.opacity(0.12), in: RoundedRectangle(cornerRadius: 8))
    }
}

struct BlurView: NSViewRepresentable {
    let material: NSWindow.Material

    func makeNSView(context: Context) -> NSVisualEffectView {
        let view = NSVisualEffectView()
        view.blendingMode = .behindWindow
        view.state = .active
        view.material = material
        return view
    }

    func updateNSView(_ nsView: NSVisualEffectView, context: Context) {}
}

struct MathChunkText: View {
    let chunks: [NativeHelperChunk]

    var body: some View {
        Group {
            if chunks.isEmpty {
                Text("暂无标题")
                    .foregroundStyle(.secondary)
            } else {
                HStack(alignment: .firstTextBaseline, spacing: 0) {
                    ForEach(chunks) { chunk in
                        switch chunk.kind {
                        case .normal:
                            Text(chunk.text)
                        case .verbatim:
                            Text(chunk.text)
                                .font(.system(.body, design: .monospaced))
                        case .math:
                            MathChunk(chunk.text)
                        }
                    }
                }
            }
        }
    }
}

struct MathChunk: View {
    let value: String

    init(_ value: String) {
        self.value = value
    }

    var body: some View {
        #if canImport(LaTeXSwiftUI)
        LaTeX(value)
            .font(.system(size: 14))
        #else
        Text(value)
            .font(.system(size: 14))
        #endif
    }
}
