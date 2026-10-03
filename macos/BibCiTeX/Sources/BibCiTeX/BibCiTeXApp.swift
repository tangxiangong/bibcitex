import SwiftUI
import AppKit
import Carbon
import BibCiTeXCore

@main
struct BibCiTeXApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var delegate
    @StateObject private var model = WorkbenchModel()
    @StateObject private var updates = Updater()
    @Environment(\.openWindow) private var openWindow
    @AppStorage("appearance") private var appearance = "system"
    var body: some Scene {
        Window("BibCiTeX", id: "main") {
            WorkbenchView(model: model)
                .onAppear {
                    applyAppearance()
                    delegate.reopenMain = { openWindow(id: "main") }
                    delegate.updates = updates
                    delegate.configureMainWindow()
                }
                .onChange(of: appearance) { _ in applyAppearance() }
        }
        .defaultSize(width: 1200, height: 800)
        .commands {
            CommandGroup(after: .appInfo) {
                Button("检查更新") { updates.check() }.disabled(!updates.canCheck)
            }
            CommandGroup(replacing: .newItem) {
                Button("新增文献库") { delegate.showMainWindow(); model.adding = true }.keyboardShortcut("o")
            }
            CommandMenu("文献") {
                Button("快捷助手") { HelperPanelController.shared.showPanel() }.keyboardShortcut("k", modifiers: [.command, .shift])
                Button("复制引用键") { if let reference = model.reference { model.copy(reference.citeKey) } }.keyboardShortcut("c", modifiers: [.command, .shift]).disabled(model.reference == nil)
                Button("刷新") { Task { await model.reload() } }.keyboardShortcut("r")
            }
            CommandGroup(after: .toolbar) {
                Picker("主题", selection: $appearance) {
                    Text("跟随系统").tag("system")
                    Text("浅色").tag("light")
                    Text("深色").tag("dark")
                }
            }
        }
    }
    private func applyAppearance() {
        NSApp.appearance = appearance == "system" ? nil : NSAppearance(named: appearance == "dark" ? .darkAqua : .aqua)
    }
}

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate, NSWindowDelegate, NSMenuItemValidation {
    var reopenMain: (() -> Void)?
    var updates: Updater?
    private var statusItem: NSStatusItem?
    private var hotKey: EventHotKeyRef?
    private var eventHandler: EventHandlerRef?
    func applicationWillFinishLaunching(_ notification: Notification) { BibCiTeXCore.initialize() }
    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.regular)
        configureMainWindow()
        let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.squareLength)
        if let image = SVGImages.images["library"]?.copy() as? NSImage {
            image.isTemplate = true; image.size = NSSize(width: 18, height: 18); item.button?.image = image
        }
        item.button?.toolTip = "BibCiTeX"
        let menu = NSMenu()
        let main = menu.addItem(withTitle: "显示窗口", action: #selector(showMainWindow), keyEquivalent: "")
        main.target = self
        let helper = menu.addItem(withTitle: "快捷助手", action: #selector(showHelper), keyEquivalent: "")
        helper.target = self
        let update = menu.addItem(withTitle: "检查更新", action: #selector(checkUpdates), keyEquivalent: "")
        update.target = self
        menu.addItem(.separator())
        menu.addItem(withTitle: "退出 BibCiTeX", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        item.menu = menu
        statusItem = item
        registerShortcut()
    }
    func validateMenuItem(_ menuItem: NSMenuItem) -> Bool {
        menuItem.action != #selector(checkUpdates) || updates?.canCheck == true
    }
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { false }
    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        if !flag { showMainWindow() }; return true
    }
    func applicationWillTerminate(_ notification: Notification) {
        if let hotKey { UnregisterEventHotKey(hotKey) }
        if let eventHandler { RemoveEventHandler(eventHandler) }
    }
    func windowShouldClose(_ sender: NSWindow) -> Bool { sender.orderOut(nil); return false }
    @objc func showMainWindow() {
        if let window = NSApp.windows.first(where: { !($0 is NSPanel) && $0.title == "BibCiTeX" }) {
            window.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
        } else {
            reopenMain?()
            NSApp.activate(ignoringOtherApps: true)
        }
    }
    func configureMainWindow() {
        DispatchQueue.main.async {
            for window in NSApp.windows where !(window is NSPanel) && window.title == "BibCiTeX" {
                window.delegate = self
                window.isReleasedWhenClosed = false
                window.isOpaque = false
                window.backgroundColor = .clear
                window.titlebarAppearsTransparent = true
            }
        }
    }
    @objc private func checkUpdates() { updates?.check() }
    @objc private func showHelper() { HelperPanelController.shared.showPanel() }
    private func registerShortcut() {
        var spec = EventTypeSpec(eventClass: OSType(kEventClassKeyboard), eventKind: UInt32(kEventHotKeyPressed))
        let status = InstallEventHandler(GetApplicationEventTarget(), { _, event, _ in
            guard let event else { return OSStatus(eventNotHandledErr) }
            var identifier = EventHotKeyID()
            GetEventParameter(event, EventParamName(kEventParamDirectObject), EventParamType(typeEventHotKeyID), nil, MemoryLayout<EventHotKeyID>.size, nil, &identifier)
            guard identifier.signature == 0x42494243 && identifier.id == 1 else { return OSStatus(eventNotHandledErr) }
            Task { @MainActor in HelperPanelController.shared.showPanel() }
            return noErr
        }, 1, &spec, nil, &eventHandler)
        guard status == noErr else { return }
        let identifier = EventHotKeyID(signature: 0x42494243, id: 1)
        let result = RegisterEventHotKey(UInt32(kVK_ANSI_K), UInt32(cmdKey | shiftKey), identifier, GetApplicationEventTarget(), 0, &hotKey)
        if result != noErr { NSLog("BibCiTeX: Cmd+Shift+K registration failed (%d)", result) }
    }
}
