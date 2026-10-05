import AppKit
import SwiftUI
import Combine

@MainActor
private final class HelperPanel: NSPanel {
    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { false }
}

/// Keyboard contract for the global-hotkey search-and-paste panel.
@MainActor
enum HelperKeyHandling {
    /// Returns nil to consume the event, or the event to let AppKit handle it.
    static func handle(_ event: NSEvent, model: HelperViewModel, dismiss: @escaping () -> Void) -> NSEvent? {
        // Let the field editor/input method finish composition before handling shortcuts.
        if let editor = event.window?.firstResponder as? NSTextView, editor.hasMarkedText() {
            return event
        }
        if !event.modifierFlags.intersection([.command, .control, .option, .shift]).isEmpty {
            return event
        }

        switch event.keyCode {
        case 53: // Esc
            dismiss()
            return nil
        case 125: // Down
            model.moveSelection(delta: 1)
            return nil
        case 126: // Up
            model.moveSelection(delta: -1)
            return nil
        case 36: // Return
            if model.isSelectingBibliography {
                model.chooseCurrentBibliography()
            } else {
                model.activateSelection(onSuccess: dismiss)
            }
            return nil
        case 48: // Tab
            if event.charactersIgnoringModifiers?.lowercased() == "\t" {
                model.startSelectMode()
                return nil
            }
            return event
        default:
            return event
        }
    }
}

@MainActor
final class HelperPanelController: NSObject {
    static let shared = HelperPanelController()

    /// State belongs only to the global-hotkey helper.
    let service = HelperService()
    lazy var model = HelperViewModel(service: service)

    private var panel: NSPanel?
    private var cancellable: AnyCancellable?
    private var keyMonitor: Any?
    private var hostingController: NSHostingController<HelperView>?
    private var isVisible = false

    override init() {
        super.init()
    }

    private var opening = false

    func showPanel() {
        if isVisible || opening { hidePanel(); return }
        // Dismiss the independent tray browser before showing the helper.
        TrayPanelController.shared.dismiss()
        // Do not replace the target snapshot while a prior OS paste is queued/running.
        guard !model.isPasteOperationPending else { return }
        let session = model.beginSession()
        opening = true
        Task { @MainActor in
            do {
                try await RustCore.pasteboard.capturePasteTarget()
                guard model.sessionGeneration == session else { return }
                opening = false
                presentPanel()
            } catch {
                guard model.sessionGeneration == session else { return }
                opening = false
                NSAlert(error: error).runModal()
            }
        }
    }

    private func presentPanel() {
        ensurePanel()
        // The hotkey panel is the cross-app one: activating a row hands its cite
        // key to the app that was frontmost when the panel opened.
        // This surface owns paste-failure restore while it is the visible one.
        model.onPasteFailure = { [weak self] in
            guard let self else { return }
            self.isVisible = true
            self.positionPanel(accordingTo: self.model.preferredHeight)
            self.panel?.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
            self.installKeyMonitor()
        }
        model.loadState()
        model.recalcHeight()

        guard let panel else { return }
        isVisible = true
        positionPanel(accordingTo: model.preferredHeight)
        panel.makeKeyAndOrderFront(nil)
        panel.orderFrontRegardless()
        NSApp.activate(ignoringOtherApps: true)
        model.focusRequest += 1
        installKeyMonitor()
    }

    func hidePanel() {
        model.invalidateSession()
        opening = false
        isVisible = false
        panel?.orderOut(nil)
        removeKeyMonitor()
    }

    func isPanelVisible() -> Bool {
        return isVisible
    }

    func setTheme(_ rawMode: Int32) {
        model.setTheme(rawMode)
        panel?.appearance = NSAppearance(named: model.theme.isDark ? .darkAqua : .aqua)
    }

    private func ensurePanel() {
        if panel != nil {
            return
        }

        let frame = NSRect(x: 0, y: 0, width: 720, height: model.preferredHeight)
        let style: NSWindow.StyleMask = [
            .nonactivatingPanel,
            .fullSizeContentView,
            .titled,
        ]

        let panel = HelperPanel(
            contentRect: frame,
            styleMask: style,
            backing: .buffered,
            defer: true,
        )

        panel.isFloatingPanel = true
        panel.level = .floating
        panel.collectionBehavior = [.transient, .moveToActiveSpace, .fullScreenAuxiliary]
        panel.isOpaque = false
        panel.hasShadow = true
        panel.isReleasedWhenClosed = false
        panel.titlebarAppearsTransparent = true
        panel.titleVisibility = .hidden
        panel.standardWindowButton(.closeButton)?.isHidden = true
        panel.standardWindowButton(.miniaturizeButton)?.isHidden = true
        panel.standardWindowButton(.zoomButton)?.isHidden = true
        panel.delegate = self
        panel.appearance = nil

        let content = HelperView(model: model, onHidePanel: hidePanel)
        let host = NSHostingController(rootView: content)
        if #available(macOS 13.0, *) {
            // The model owns the panel height, including shrinking back to search-only.
            host.sizingOptions = []
        }
        host.view.translatesAutoresizingMaskIntoConstraints = false
        panel.minSize = NSSize(width: 0, height: HelperMetrics.header)

        let surface: NSView
        if #available(macOS 26.0, *) {
            let glass = NSGlassEffectView()
            glass.style = .regular
            glass.cornerRadius = 18
            glass.contentView = host.view
            surface = glass
        } else {
            let material = NSVisualEffectView()
            material.material = .hudWindow
            material.blendingMode = .behindWindow
            material.state = .followsWindowActiveState
            material.addSubview(host.view)
            surface = material
        }
        panel.contentView = surface
        NSLayoutConstraint.activate([
            host.view.leadingAnchor.constraint(equalTo: surface.leadingAnchor),
            host.view.trailingAnchor.constraint(equalTo: surface.trailingAnchor),
            host.view.topAnchor.constraint(equalTo: surface.topAnchor),
            host.view.bottomAnchor.constraint(equalTo: surface.bottomAnchor),
        ])
        panel.backgroundColor = .clear

        self.panel = panel
        self.hostingController = host

        cancellable = model.$preferredHeight
            .removeDuplicates()
            .receive(on: RunLoop.main)
            .sink { [weak self] height in
                guard let self, isVisible, height == model.preferredHeight else { return }
                positionPanel(accordingTo: height)
            }
    }

    private func installKeyMonitor() {
        if keyMonitor != nil { return }

        keyMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
            guard let self else { return event }
            if event.window !== self.panel || self.panel?.isKeyWindow != true {
                return event
            }
            return HelperKeyHandling.handle(event, model: self.model, dismiss: self.hidePanel)
        }
    }

    private func removeKeyMonitor() {
        if let keyMonitor {
            NSEvent.removeMonitor(keyMonitor)
            self.keyMonitor = nil
        }
    }

    private func positionPanel(accordingTo height: CGFloat) {
        guard let panel else { return }

        let screen = NSScreen.screens.first { NSMouseInRect(NSEvent.mouseLocation, $0.frame, false) } ?? NSScreen.main
        let visibleFrame = screen?.visibleFrame ?? NSRect(x: 0, y: 0, width: 960, height: 760)
        let width: CGFloat = min(720, visibleFrame.width - 40)
        let clampedHeight = min(max(height, HelperMetrics.header), visibleFrame.height - 120)

        let x = visibleFrame.minX + (visibleFrame.width - width) / 2
        let y = visibleFrame.maxY - clampedHeight - visibleFrame.height / 6

        var frame = panel.frame
        frame.size = CGSize(width: width, height: clampedHeight)
        frame.origin = CGPoint(x: x, y: y)
        if panel.frame != frame {
            panel.setFrame(frame, display: true, animate: false)
        }
    }
}

extension HelperPanelController: NSWindowDelegate {
    func windowDidResignKey(_ notification: Notification) {
        guard isVisible, !model.isPasting else { return }
        hidePanel()
    }
}
