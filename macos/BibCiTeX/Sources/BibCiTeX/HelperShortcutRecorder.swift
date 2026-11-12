import AppKit
import SwiftUI

struct HelperShortcutRecorder: NSViewRepresentable {
    let value: String
    let delegate: AppDelegate
    let onChange: (String) -> Void
    let onInvalid: () -> Void

    func makeNSView(context: Context) -> ShortcutRecorderButton {
        let button = ShortcutRecorderButton(frame: .zero)
        button.bezelStyle = .rounded
        button.setButtonType(.momentaryPushIn)
        button.font = .systemFont(ofSize: NSFont.systemFontSize)
        button.target = button
        button.action = #selector(ShortcutRecorderButton.beginRecording)
        return button
    }

    func updateNSView(_ button: ShortcutRecorderButton, context: Context) {
        button.value = value
        button.shortcutDelegate = delegate
        button.onChange = onChange
        button.onInvalid = onInvalid
        button.refreshTitle()
    }

    static func dismantleNSView(_ button: ShortcutRecorderButton, coordinator: ()) {
        button.cancelRecording()
    }
}

@MainActor
final class ShortcutRecorderButton: NSButton {
    var value = HelperShortcut.defaultValue
    weak var shortcutDelegate: AppDelegate?
    var onChange: ((String) -> Void)?
    var onInvalid: (() -> Void)?
    private var recording = false
    private var keyMonitor: Any?
    private var activationObserver: NSObjectProtocol?

    override var acceptsFirstResponder: Bool { true }

    func refreshTitle() {
        title = recording ? L10n.text("请按快捷键…") : HelperShortcut(value)?.displayValue ?? value
        setAccessibilityLabel(L10n.text("助手快捷键"))
        setAccessibilityHelp(L10n.text(recording ? "按 Esc 取消录制" : "录制快捷键"))
    }

    @objc func beginRecording() {
        if recording { cancelRecording(); return }
        guard let window, window.makeFirstResponder(self) else { return }
        recording = true
        shortcutDelegate?.recordCurrentShortcut = { [weak self] in
            guard let self else { return }
            self.finishRecording(self.value)
        }
        keyMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
            guard let self, self.recording, event.window === self.window else { return event }
            if event.keyCode == 53 { self.cancelRecording(); return nil }
            if event.keyCode == 48 { self.cancelRecording(); return event }
            guard !event.isARepeat else { return nil }
            if let shortcut = HelperShortcut(keyCode: event.keyCode, modifierFlags: event.modifierFlags) {
                self.finishRecording(shortcut.value)
            } else {
                self.onInvalid?()
                NSSound.beep()
            }
            return nil
        }
        activationObserver = NotificationCenter.default.addObserver(forName: NSWindow.didResignKeyNotification, object: window, queue: .main) { [weak self] _ in
            Task { @MainActor in self?.cancelRecording() }
        }
        refreshTitle()
    }

    func cancelRecording() {
        guard recording else { return }
        recording = false
        if let keyMonitor { NSEvent.removeMonitor(keyMonitor) }
        keyMonitor = nil
        if let activationObserver { NotificationCenter.default.removeObserver(activationObserver) }
        activationObserver = nil
        shortcutDelegate?.recordCurrentShortcut = nil
        refreshTitle()
    }

    private func finishRecording(_ value: String) {
        cancelRecording()
        onChange?(value)
    }

    override func resignFirstResponder() -> Bool {
        cancelRecording()
        return super.resignFirstResponder()
    }

    override func viewWillMove(toWindow newWindow: NSWindow?) {
        if newWindow == nil { cancelRecording() }
        super.viewWillMove(toWindow: newWindow)
    }
}
