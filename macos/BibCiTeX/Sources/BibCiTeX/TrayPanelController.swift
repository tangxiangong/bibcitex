import AppKit
import SwiftUI
import Combine

@MainActor
private final class TrayPanel: NSPanel {
    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { false }
}

/// The status-item window: the workbench collapsed to menu-bar scale — one
/// search field over the reference list — hanging under the menu-bar button.
///
/// This is a second *presentation* of the shared helper, not a second
/// implementation: the same `HelperViewModel`, `HelperServing`, Rust paste path
/// and `HelperView` that Cmd+Shift+K shows. Search, DTO and FFI code stay where
/// they already live; this file owns the window and nothing else.
///
/// Left-clicking the status item toggles the panel; right-clicking or
/// Control-clicking shows the menu. The panel is presented only after the Rust
/// core snapshots the paste target, so the app that receives the paste is never
/// the one that was just clicked.
@MainActor
final class TrayPanelController: NSObject {
    static let shared = TrayPanelController()

    private var model: HelperViewModel { HelperPanelController.shared.model }

    /// The status-item button the panel hangs from, installed by `AppDelegate`.
    private weak var anchor: NSStatusBarButton?

    private var panel: NSPanel?
    private var hostingController: NSHostingController<HelperView>?
    private var keyMonitor: Any?
    private var outsideMonitor: Any?
    private var heightCancellable: AnyCancellable?
    private var isVisible = false
    private var opening = false

    /// A cite key plus its venue on one line, and no wider: the tray window has
    /// to read as a panel attached to the menu bar, not as a second main window.
    private static let width: CGFloat = 460
    /// Rounds the surrounding material without clipping a row corner.
    private static let cornerRadius: CGFloat = 12

    /// Anchors the tray window to a status-item button.
    func attach(to button: NSStatusBarButton) {
        anchor = button
    }

    func toggle() {
        if isVisible || opening { dismiss() } else { show() }
    }

    /// True while the tray window owns the shared model.
    func isPanelVisible() -> Bool { isVisible }

    func show() {
        // One shared model drives one visible surface at a time.
        HelperPanelController.shared.hidePanel()
        // Do not replace the target snapshot while a prior OS paste is queued or running.
        guard !model.isPasteOperationPending else { return }
        let session = model.beginSession()
        opening = true
        Task { @MainActor in
            do {
                try await RustCore.pasteboard.capturePasteTarget()
                guard model.sessionGeneration == session else { return }
                opening = false
                present()
            } catch {
                guard model.sessionGeneration == session else { return }
                opening = false
                NSAlert(error: error).runModal()
            }
        }
    }

    func setTheme(_ rawMode: Int32) {
        model.setTheme(rawMode)
        panel?.appearance = NSAppearance(named: model.theme.isDark ? .darkAqua : .aqua)
    }

    func dismiss() {
        guard isVisible || opening else { return }
        model.invalidateSession()
        opening = false
        isVisible = false
        panel?.orderOut(nil)
        removeKeyMonitor()
        removeOutsideMonitor()
    }

    private func present() {
        ensurePanel()
        // This surface collects cite keys: activating a row copies and leaves the
        // window open, so several records can be taken in one visit.
        model.mode = .copy
        // This surface owns paste-failure restore while it is the visible one.
        model.onPasteFailure = { [weak self] in
            guard let self else { return }
            self.isVisible = true
            self.position(accordingTo: self.model.preferredHeight)
            self.panel?.makeKeyAndOrderFront(nil)
            self.installKeyMonitor()
        }
        model.loadState()
        model.recalcHeight()

        guard let panel else { return }
        isVisible = true
        position(accordingTo: model.preferredHeight)
        // A nonactivating panel becomes key without activating the app, so the
        // previously focused app keeps the pending paste target.
        panel.makeKeyAndOrderFront(nil)
        panel.orderFrontRegardless()
        model.focusRequest += 1
        installKeyMonitor()
        installOutsideMonitor()
    }

    private func ensurePanel() {
        if panel != nil { return }

        let frame = NSRect(x: 0, y: 0, width: Self.width, height: model.preferredHeight)
        let panel = TrayPanel(
            contentRect: frame,
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: true,
        )
        panel.isFloatingPanel = true
        panel.level = .floating
        // Transient keeps it out of the window list and the Cmd+Tab switcher.
        panel.collectionBehavior = [.transient, .moveToActiveSpace, .fullScreenAuxiliary]
        panel.isOpaque = false
        panel.hasShadow = true
        panel.isReleasedWhenClosed = false
        panel.backgroundColor = .clear
        panel.appearance = nil
        panel.animationBehavior = .utilityWindow
        panel.minSize = NSSize(width: 0, height: HelperMetrics.header)
        panel.delegate = self

        let content = HelperView(model: model, onHidePanel: { [weak self] in self?.dismiss() })
        let host = NSHostingController(rootView: content)
        if #available(macOS 13.0, *) {
            // The model owns the panel height; SwiftUI must not add its own size.
            host.sizingOptions = []
        }
        host.view.translatesAutoresizingMaskIntoConstraints = false

        let surface: NSView
        if #available(macOS 26.0, *) {
            let glass = NSGlassEffectView()
            glass.style = .regular
            glass.cornerRadius = Self.cornerRadius
            glass.contentView = host.view
            surface = glass
        } else {
            let material = NSVisualEffectView()
            material.material = .hudWindow
            material.blendingMode = .behindWindow
            // Forced active: a nonactivating panel is never the app's active
            // window, so followsWindowActiveState would wash the panel out.
            material.state = .active
            material.wantsLayer = true
            material.layer?.cornerRadius = Self.cornerRadius
            material.layer?.masksToBounds = true
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

        self.panel = panel
        self.hostingController = host

        heightCancellable = model.$preferredHeight
            .removeDuplicates()
            .receive(on: RunLoop.main)
            .sink { [weak self] height in
                guard let self, self.isVisible, height == self.model.preferredHeight else { return }
                self.position(accordingTo: height)
            }
    }

    private func installKeyMonitor() {
        if keyMonitor != nil { return }
        keyMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
            guard let self else { return event }
            if event.window !== self.panel || self.panel?.isKeyWindow != true {
                return event
            }
            return HelperKeyHandling.handle(event, model: self.model, dismiss: self.dismiss)
        }
    }

    private func removeKeyMonitor() {
        if let keyMonitor {
            NSEvent.removeMonitor(keyMonitor)
            self.keyMonitor = nil
        }
    }

    private func installOutsideMonitor() {
        if outsideMonitor != nil { return }
        // A nonactivating panel does not resign key on a click that lands in
        // another app, so watch global clicks to dismiss on an outside click.
        outsideMonitor = NSEvent.addGlobalMonitorForEvents(
            matching: [.leftMouseDown, .rightMouseDown, .otherMouseDown]
        ) { [weak self] _ in
            Task { @MainActor in
                guard let self, self.isVisible, let panel = self.panel else { return }
                if panel.frame.contains(NSEvent.mouseLocation) { return }
                self.dismiss()
            }
        }
    }

    private func removeOutsideMonitor() {
        if let outsideMonitor {
            NSEvent.removeMonitor(outsideMonitor)
            self.outsideMonitor = nil
        }
    }

    /// Hang the panel just below the status-item button, clamped to its screen.
    private func position(accordingTo height: CGFloat) {
        guard let panel else { return }

        let visibleFrame = (anchor?.window?.screen ?? NSScreen.main)?.visibleFrame
            ?? NSRect(x: 0, y: 0, width: 1440, height: 760)
        let width = min(Self.width, visibleFrame.width - 32)
        let clampedHeight = min(max(height, HelperMetrics.header), visibleFrame.height - 96)

        // Centre on the status-item button so the panel points at the icon the
        // user clicked, then keep it fully on screen.
        var x = visibleFrame.maxX - width
        if let button = anchor, let buttonWindow = button.window {
            let buttonFrame = buttonWindow.convertToScreen(button.convert(button.bounds, to: nil))
            x = buttonFrame.midX - width / 2
        }
        x = min(max(x, visibleFrame.minX + 8), visibleFrame.maxX - width - 8)

        // The menu bar owns the top of the visible frame, so the panel hangs
        // from it with the same gap the system's own menu-bar extras use.
        let y = visibleFrame.maxY - clampedHeight - 6

        var frame = panel.frame
        frame.size = CGSize(width: width, height: clampedHeight)
        frame.origin = CGPoint(x: x, y: y)
        if panel.frame != frame {
            panel.setFrame(frame, display: true, animate: false)
        }
    }
}

extension TrayPanelController: NSWindowDelegate {
    func windowDidResignKey(_ notification: Notification) {
        guard isVisible, !model.isPasting else { return }
        dismiss()
    }
}
