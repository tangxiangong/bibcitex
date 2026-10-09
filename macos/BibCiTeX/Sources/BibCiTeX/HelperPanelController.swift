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
        case 36, 76: // Return / keypad Enter
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
    private var failureAlert: NSAlert?

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
        model.onPasteStart = { [weak self] in
            // Release the panel's keyboard focus without invalidating the paste session.
            self?.isVisible = false
            self?.panel?.orderOut(nil)
            self?.removeKeyMonitor()
        }
        // The hotkey panel is the cross-app one: activating a row hands its cite
        // key to the app that was frontmost when the panel opened.
        // This surface owns paste-failure restore while it is the visible one.
        model.onPasteFailure = { [weak self] in
            self?.presentPasteFailure()
        }
        model.loadState()
        model.recalcHeight()

        guard let panel else { return }
        isVisible = true
        positionPanel(accordingTo: model.preferredHeight)
        showWithoutActivatingApplication(panel)
        model.focusRequest += 1
        installKeyMonitor()
    }

    private func presentPasteFailure() {
        guard let panel, failureAlert == nil, let message = model.errorMessage else { return }
        let alert = NSAlert()
        alert.alertStyle = .warning
        alert.messageText = message
        alert.addButton(withTitle: L10n.text("确定"))
        // Set this before any key-window transition: sheet attachment may resign
        // the panel before AppKit exposes attachedSheet.
        failureAlert = alert
        isVisible = true
        positionPanel(accordingTo: model.preferredHeight)
        showWithoutActivatingApplication(panel)
        installKeyMonitor()
        let session = model.sessionGeneration
        alert.beginSheetModal(for: panel) { [weak self] _ in
            guard let self, self.model.sessionGeneration == session else { return }
            self.failureAlert = nil
            guard self.isVisible else { return }
            if let panel = self.panel { self.showWithoutActivatingApplication(panel) }
            self.model.focusRequest += 1
        }
    }

    private func showWithoutActivatingApplication(_ panel: NSPanel) {
        let alpha = panel.isVisible ? panel.alphaValue : 0
        // Order just this nonactivating panel, then give it keyboard focus.
        // Do not unhide/activate the app, which would restore its other windows.
        panel.orderFrontRegardless()
        panel.makeKey()
        PanelSurface.refreshShadow(panel)
        PanelSurface.fadeIn(panel, from: alpha)
    }

    func hidePanel() {
        model.invalidateSession()
        opening = false
        isVisible = false
        if let panel, let sheet = panel.attachedSheet {
            panel.endSheet(sheet, returnCode: .cancel)
        }
        failureAlert = nil
        removeKeyMonitor()
        guard let panel, panel.isVisible else { return }
        // A re-show during the fade owns the panel again.
        PanelSurface.fadeOut(panel) { [weak self] in self?.isVisible == false }
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
        // Borderless: the rounded content alone shapes the panel and its shadow.
        let style: NSWindow.StyleMask = [
            .borderless,
            .nonactivatingPanel,
            .fullSizeContentView,
        ]

        let panel = HelperPanel(
            contentRect: frame,
            styleMask: style,
            backing: .buffered,
            defer: true,
        )

        panel.isFloatingPanel = true
        panel.hidesOnDeactivate = false
        // App hiding (Cmd+H) is independent of losing activation. The helper
        // must remain displayable without unhiding the main application.
        panel.canHide = false
        panel.level = .floating
        panel.collectionBehavior = [.transient, .moveToActiveSpace, .fullScreenAuxiliary]
        panel.isOpaque = false
        panel.hasShadow = true
        panel.animationBehavior = .none
        panel.isReleasedWhenClosed = false
        panel.delegate = self
        panel.appearance = nil

        let content = HelperView(model: model, onHidePanel: hidePanel)
        let host = NSHostingController(rootView: content)
        if #available(macOS 13.0, *) {
            // The model owns the panel height, including shrinking back to search-only.
            host.sizingOptions = []
        }
        panel.minSize = NSSize(width: 0, height: HelperMetrics.header)
        // The hosting view is the whole content; HelperView draws and clips the material.
        host.view.wantsLayer = true
        panel.contentView = host.view
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
            if event.window !== self.panel || self.panel?.isKeyWindow != true || self.failureAlert != nil || self.panel?.attachedSheet != nil {
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
            PanelSurface.refreshShadow(panel)
        }
    }
}

extension HelperPanelController: NSWindowDelegate {
    func windowDidResignKey(_ notification: Notification) {
        guard isVisible, !model.isPasting, failureAlert == nil, panel?.attachedSheet == nil else { return }
        hidePanel()
    }
}
