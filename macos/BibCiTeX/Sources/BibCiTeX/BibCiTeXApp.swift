import SwiftUI
import AppKit
import SwiftDraw
import Carbon
import BibCiTeXCore

@main
struct BibCiTeXApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var delegate
    @StateObject private var model = WorkbenchModel()
    @StateObject private var updates = Updater()
    @Environment(\.openWindow) private var openWindow
    @AppStorage("language") private var language = "system"
    @AppStorage("appearance") private var appearance = "system"
    var body: some Scene {
        Window("BibCiTeX", id: "main") {
            WorkbenchView(model: model)
                .onAppear {
                    applyAppearance(appearance)
                    delegate.reopenMain = { openWindow(id: "main") }
                    delegate.updates = updates
                    delegate.configureMainWindow()
                }
                .environment(\.locale, L10n.locale)
                .onChange(of: language) { _ in delegate.localizeMenu(); updates.languageChanged() }
                .onChange(of: appearance) { value in applyAppearance(value) }
        }
        .defaultSize(width: 1200, height: 800)
        .commands {
            CommandGroup(replacing: .appInfo) {
                Button(L10n.text("关于 BibCiTeX")) {
                    var options: [NSApplication.AboutPanelOptionKey: Any] = [:]
                    if let image = AppImages.logo {
                        options[.applicationIcon] = image
                    }
                    NSApp.orderFrontStandardAboutPanel(options: options)
                }
            }
            CommandGroup(replacing: .appSettings) {
                Button(L10n.text("设置") + "…") { openWindow(id: "settings") }.keyboardShortcut(",")
            }
            CommandGroup(after: .appInfo) {
                Button(L10n.text("检查更新")) { updates.check() }.disabled(!updates.canCheck)
            }
            CommandGroup(replacing: .newItem) {
                Button(L10n.text("新增文献库")) { delegate.showMainWindow(); model.adding = true }.keyboardShortcut("o")
            }
            CommandMenu(L10n.text("文献")) {
                Button(L10n.text("快捷助手")) { HelperPanelController.shared.showPanel() }.keyboardShortcut("k", modifiers: [.command, .shift])
                Button(L10n.text("复制引用键")) { if let reference = model.reference { model.copy(reference.citeKey) } }.keyboardShortcut("c", modifiers: [.command, .shift]).disabled(model.reference == nil)
                Button(L10n.text("刷新")) { Task { await model.reload() } }.keyboardShortcut("r")
            }
        }
        Window(L10n.text("设置"), id: "settings") {
            PreferencesView(updates: updates)
                .environment(\.locale, L10n.locale)
                .onAppear { applyAppearance(appearance) }
                .onChange(of: appearance) { value in applyAppearance(value) }
                .onChange(of: language) { _ in delegate.localizeMenu(); updates.languageChanged() }
        }
        .windowResizability(.contentSize)
    }
    private func applyAppearance(_ appearance: String) {
        NSApp.appearance = appearance == "system" ? nil : NSAppearance(named: appearance == "dark" ? .darkAqua : .aqua)
    }
}

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate, NSWindowDelegate, NSMenuItemValidation {
    var reopenMain: (() -> Void)?
    var updates: Updater?
    private let mainMenuLocalizer = MainMenuLocalizer(menu: { NSApp?.mainMenu })
    private var statusItem: NSStatusItem?
    private var statusMenu: NSMenu?
    private var hotKey: EventHotKeyRef?
    private var eventHandler: EventHandlerRef?
    func applicationWillFinishLaunching(_ notification: Notification) { BibCiTeXCore.initialize() }
    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.regular)
        configureMainWindow()
        mainMenuLocalizer.start()
        let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.squareLength)
        if let image = NSImage(svgNamed: "tray-template.svg", in: .appResources) {
            image.isTemplate = true; image.size = NSSize(width: 18, height: 18); item.button?.image = image
        }
        item.button?.toolTip = "BibCiTeX"
        let menu = NSMenu()
        let main = menu.addItem(withTitle: L10n.text("显示窗口"), action: #selector(showMainWindow), keyEquivalent: "")
        main.target = self
        let helper = menu.addItem(withTitle: L10n.text("快捷助手"), action: #selector(showHelper), keyEquivalent: "")
        helper.target = self
        let update = menu.addItem(withTitle: L10n.text("检查更新"), action: #selector(checkUpdates), keyEquivalent: "")
        update.target = self
        menu.addItem(.separator())
        menu.addItem(withTitle: L10n.text("退出 BibCiTeX"), action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        // Do not assign `item.menu`, or every click would pop the menu and the
        // button action below would never run. Keep the menu and show it manually.
        statusMenu = menu
        // Left-click opens the tray window; right/Control-click shows the menu.
        if let button = item.button {
            TrayPanelController.shared.attach(to: button)
            TrayPanelController.shared.showMain = { [weak self] in self?.showMainWindow() }
            button.target = self
            button.action = #selector(handleStatusItemClick(_:))
            button.sendAction(on: [.leftMouseUp, .rightMouseUp])
        }
        statusItem = item
        registerShortcut()
    }

    @objc private func handleStatusItemClick(_ sender: NSStatusBarButton) {
        let event = NSApp.currentEvent
        let isSecondary = event?.type == .rightMouseUp
            || event?.modifierFlags.contains(.control) == true
        if isSecondary {
            if TrayPanelController.shared.isPanelVisible() { TrayPanelController.shared.dismiss() }
            localizeMenu()
            guard let menu = statusMenu else { return }
            menu.popUp(positioning: nil, at: NSPoint(x: 0, y: sender.bounds.height), in: sender)
        } else {
            TrayPanelController.shared.toggle()
        }
    }
    func localizeMenu() {
        mainMenuLocalizer.refresh()
        let keys = ["显示窗口", "快捷助手", "检查更新", "", "退出 BibCiTeX"]
        for (item, key) in zip(statusMenu?.items ?? [], keys) where !key.isEmpty {
            item.title = L10n.text(key)
        }
    }
    func validateMenuItem(_ menuItem: NSMenuItem) -> Bool {
        menuItem.action != #selector(checkUpdates) || updates?.canCheck == true
    }
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { false }
    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        if !flag { showMainWindow() }; return true
    }
    func applicationWillTerminate(_ notification: Notification) {
        mainMenuLocalizer.stop()
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
                window.isOpaque = true
                window.backgroundColor = .windowBackgroundColor
                window.titlebarAppearsTransparent = false
                window.titleVisibility = .hidden
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

private struct PreferencesView: View {
    @ObservedObject var updates: Updater
    @AppStorage("language") private var language = "system"
    @AppStorage("appearance") private var appearance = "system"

    var body: some View {
        Form {
            Section(L10n.text("外观")) {
                Picker(L10n.text("主题"), selection: $appearance) {
                    Text(L10n.text("跟随系统")).tag("system")
                    Text(L10n.text("浅色")).tag("light")
                    Text(L10n.text("深色")).tag("dark")
                }
            }
            Section(L10n.text("语言")) {
                Picker(L10n.text("语言"), selection: $language) {
                    Text(L10n.text("跟随系统")).tag("system")
                    Text("简体中文").tag("zh-Hans")
                    Text("English").tag("en")
                }
            }
            Section(L10n.text("更新")) {
                LabeledContent(L10n.text("当前版本"), value: Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "—")
                Picker(L10n.text("更新通道"), selection: $updates.channel) {
                    Text(L10n.text("正式版")).tag("stable")
                    Text("Beta").tag("beta")
                    Text("Alpha").tag("alpha")
                }
                Toggle(L10n.text("自动下载并安装更新"), isOn: $updates.automaticDownloads)
                Button(L10n.text("检查更新")) { updates.check() }.disabled(!updates.canCheck)
                if let reason = updates.unavailableReason {
                    Text(L10n.text(reason)).font(.callout).foregroundStyle(.secondary).fixedSize(horizontal: false, vertical: true)
                }
            }
        }
        .formStyle(.grouped)
        .background(AutoHidingScrollbars())
        .frame(width: 480, height: 440)
    }
}
