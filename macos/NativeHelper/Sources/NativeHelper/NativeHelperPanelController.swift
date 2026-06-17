import AppKit
import SwiftUI
import Combine

@MainActor
final class NativeHelperPanelController: NSObject {
    static let shared = NativeHelperPanelController()

    private let service = NativeHelperService()
    private lazy var model = HelperViewModel(service: service)

    private var panel: NSPanel?
    private var cancellable: AnyCancellable?
    private var keyMonitor: Any?
    private var hostingController: NSHostingController<HelperView>?
    private var isVisible = false

    override init() {
        super.init()
    }

    func showPanel() {
        ensurePanel()
        model.loadState()
        model.recalcHeight()

        guard let panel else { return }
        isVisible = true
        positionPanel(accordingTo: model.preferredHeight)
        panel.makeKeyAndOrderFront(nil)
        panel.orderFrontRegardless()
        NSApp.activate(ignoringOtherApps: true)
        installKeyMonitor()
    }

    func hidePanel() {
        isVisible = false
        panel?.orderOut(nil)
        removeKeyMonitor()
    }

    func isPanelVisible() -> Bool {
        return isVisible
    }

    func setTheme(_ rawMode: Int32) {
        model.setTheme(rawMode)
        positionPanel(accordingTo: model.preferredHeight)
    }

    private func ensurePanel() {
        if panel != nil {
            return
        }

        let frame = NSRect(x: 0, y: 0, width: 760, height: model.preferredHeight)
        let style: NSWindow.StyleMask = [
            .nonactivatingPanel,
            .fullSizeContentView,
            .titled,
        ]

        let panel = NSPanel(
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

        let content = HelperView(model: model, onHidePanel: hidePanel)
        let host = NSHostingController(rootView: content)
        host.view.translatesAutoresizingMaskIntoConstraints = false

        panel.contentView = host.view
        panel.contentViewController = host
        panel.backgroundColor = .clear

        self.panel = panel
        self.hostingController = host

        cancellable = model.$preferredHeight.sink { [weak self] height in
            self?.positionPanel(accordingTo: height)
        }
    }

    private func installKeyMonitor() {
        if keyMonitor != nil { return }

        keyMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
            guard let self else { return event }
            if self.panel?.isVisible != true {
                return event
            }

            guard let chars = event.charactersIgnoringModifiers?.lowercased() else {
                return event
            }

            switch event.keyCode {
            case 53: // Esc
                self.hidePanel()
                return nil
            case 125: // Down
                self.model.moveSelection(delta: 1)
                return nil
            case 126: // Up
                self.model.moveSelection(delta: -1)
                return nil
            case 36: // Return
                if self.model.isSelectingBibliography {
                    self.model.chooseCurrentBibliography()
                } else {
                    self.model.activateSelection(onSuccess: self.hidePanel)
                }
                return nil
            case 48: // Tab
                if chars == "\t" {
                    self.model.startSelectMode()
                    return nil
                }
                return event
            default:
                return event
            }
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

        let visibleFrame = NSScreen.main?.visibleFrame ?? NSRect(x: 0, y: 0, width: 960, height: 760)
        let width: CGFloat = max(640, min(960, 760))
        let clampedHeight = min(max(height, 160), visibleFrame.height - 120)

        let x = visibleFrame.minX + (visibleFrame.width - width) / 2
        let y = visibleFrame.maxY - clampedHeight - visibleFrame.height / 6

        var frame = panel.frame
        frame.size = CGSize(width: width, height: clampedHeight)
        frame.origin = CGPoint(x: x, y: y)
        panel.setFrame(frame, display: true, animate: false)
    }
}

extension NativeHelperPanelController: NSWindowDelegate {
    func windowDidResignKey(_ notification: Notification) {
        hidePanel()
    }
}
