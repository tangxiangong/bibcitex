import AppKit
import SwiftUI

@MainActor
private final class TrayPanel: NSPanel {
    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { false }
}

/// Owns the independent tray workbench, anchored to the status item.
@MainActor
final class TrayPanelController: NSObject, NSWindowDelegate {
    static let shared = TrayPanelController()
    private let model = TrayWorkbenchModel(service: RustWorkbenchService())
    private weak var anchor: NSStatusBarButton?
    private var panel: NSPanel?
    private var host: NSHostingController<TrayWorkbenchView>?
    private var keyMonitor: Any?
    private var outsideMonitor: Any?
    private var isVisible = false
    private var presentation = 0
    var showMain: (() -> Void)?

    func attach(to button: NSStatusBarButton) { anchor = button }
    func isPanelVisible() -> Bool { isVisible }
    func toggle() { if isVisible { dismiss() } else { show() } }
    func show() {
        HelperPanelController.shared.hidePanel()
        ensurePanel()
        position()
        isVisible = true
        presentation += 1
        let current = presentation
        panel?.makeKeyAndOrderFront(nil)
        panel?.orderFrontRegardless()
        installMonitors()
        Task {
            guard isVisible, presentation == current else { return }
            await model.reload()
        }
    }
    func dismiss() {
        guard isVisible else { return }
        isVisible = false
        presentation += 1
        model.suspend()
        panel?.orderOut(nil)
        if let keyMonitor { NSEvent.removeMonitor(keyMonitor) }
        if let outsideMonitor { NSEvent.removeMonitor(outsideMonitor) }
        keyMonitor = nil; outsideMonitor = nil
    }
    private func ensurePanel() {
        guard panel == nil else { return }
        let panel = TrayPanel(contentRect: NSRect(x: 0, y: 0, width: 680, height: 580),
            styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: true)
        panel.isFloatingPanel = true
        panel.level = .floating
        panel.collectionBehavior = [.transient, .moveToActiveSpace, .fullScreenAuxiliary]
        panel.isOpaque = false; panel.hasShadow = true
        panel.isReleasedWhenClosed = false; panel.backgroundColor = .clear
        panel.animationBehavior = .utilityWindow; panel.delegate = self
        let host = NSHostingController(rootView: TrayWorkbenchView(model: model,
            showMain: { [weak self] in self?.dismiss(); self?.showMain?() },
            dismiss: { [weak self] in self?.dismiss() }))
        host.sizingOptions = []
        host.view.translatesAutoresizingMaskIntoConstraints = false
        let surface: NSView
        if #available(macOS 26.0, *) {
            let glass = NSGlassEffectView()
            glass.style = .regular; glass.cornerRadius = 12; glass.contentView = host.view
            surface = glass
        } else {
            let material = NSVisualEffectView()
            material.material = .popover; material.blendingMode = .behindWindow; material.state = .active
            material.wantsLayer = true; material.layer?.cornerRadius = 12; material.layer?.masksToBounds = true
            material.addSubview(host.view); surface = material
        }
        panel.contentView = surface
        NSLayoutConstraint.activate([
            host.view.leadingAnchor.constraint(equalTo: surface.leadingAnchor),
            host.view.trailingAnchor.constraint(equalTo: surface.trailingAnchor),
            host.view.topAnchor.constraint(equalTo: surface.topAnchor),
            host.view.bottomAnchor.constraint(equalTo: surface.bottomAnchor),
        ])
        self.panel = panel; self.host = host
    }
    private func installMonitors() {
        guard keyMonitor == nil else { return }
        keyMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
            guard let self, event.window === panel, panel?.isKeyWindow == true else { return event }
            if let editor = panel?.firstResponder as? NSTextView, editor.hasMarkedText() { return event }
            guard event.modifierFlags.intersection([.command, .option, .control, .shift]).isEmpty else { return event }
            if event.keyCode == 53 { dismiss(); return nil }
            // Only the search field redirects arrows; lists and pickers keep native navigation.
            if let editor = panel?.firstResponder as? NSTextView, editor.isFieldEditor {
                if event.keyCode == 125 { model.move(1); return nil }
                if event.keyCode == 126 { model.move(-1); return nil }
            }
            return event
        }
        outsideMonitor = NSEvent.addGlobalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown, .otherMouseDown]) { [weak self] _ in
            Task { @MainActor in
                guard let self, self.isVisible, let panel = self.panel, !panel.frame.contains(NSEvent.mouseLocation) else { return }
                self.dismiss()
            }
        }
    }
    private func position() {
        guard let panel else { return }
        let screen = (anchor?.window?.screen ?? NSScreen.main)?.visibleFrame
            ?? NSRect(x: 0, y: 0, width: 1440, height: 900)
        let width = min(680, screen.width - 16), height = min(580, screen.height - 16)
        var x = screen.maxX - width - 8
        if let button = anchor, let window = button.window {
            x = window.convertToScreen(button.convert(button.bounds, to: nil)).midX - width / 2
        }
        x = min(max(x, screen.minX + 8), screen.maxX - width - 8)
        panel.setFrame(NSRect(x: x, y: screen.maxY - height - 6, width: width, height: height), display: true)
    }
    func windowDidResignKey(_ notification: Notification) { dismiss() }
}
