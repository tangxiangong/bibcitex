import AppKit

/// SwiftUI's environment locale does not localize AppKit's standard menu bar.
/// Update the existing menus so their actions, validation and shortcuts survive.
@MainActor
final class MainMenuLocalizer: NSObject {
    private let menuProvider: () -> NSMenu?
    private var isApplying = false
    private var isObserving = false

    init(menu: @escaping () -> NSMenu?) {
        menuProvider = menu
        super.init()
    }

    func start() {
        guard !isObserving else { return }
        isObserving = true
        for name in [NSMenu.didAddItemNotification, NSMenu.didChangeItemNotification] {
            NotificationCenter.default.addObserver(self, selector: #selector(menuChanged(_:)), name: name, object: nil)
        }
        refresh()
    }

    func stop() {
        NotificationCenter.default.removeObserver(self)
        isObserving = false
    }

    func refresh() {
        guard !isApplying, let menu = menuProvider() else { return }
        isApplying = true
        defer { isApplying = false }
        Self.localize(menu, language: L10n.language)
    }

    @objc private func menuChanged(_ notification: Notification) {
        guard !isApplying, let changed = notification.object as? NSMenu,
              let main = menuProvider(), Self.contains(changed, in: main) else { return }
        // SwiftUI can regenerate system menu titles after the language changes.
        refresh()
    }

    private static func contains(_ menu: NSMenu, in root: NSMenu) -> Bool {
        root === menu || root.items.contains { item in
            item.submenu.map { contains(menu, in: $0) } ?? false
        }
    }

    static func localize(_ main: NSMenu, language: String) {
        let headings: [(key: String, titles: [String], action: String)] = [
            ("menu.file", ["File", "文件"], "performClose:"),
            ("menu.edit", ["Edit", "编辑"], "copy:"),
            ("menu.view", ["View", "显示", "视图"], "toggleFullScreen:"),
            ("menu.window", ["Window", "窗口"], "performMiniaturize:"),
            ("menu.help", ["Help", "帮助"], "showHelp:")
        ]
        for item in main.items {
            guard let submenu = item.submenu else { continue }
            let heading = headings.first { $0.titles.contains(item.title) || $0.titles.contains(submenu.title) }
                ?? headings.first { heading in submenu.items.contains { $0.action == NSSelectorFromString(heading.action) } }
            guard let heading else { continue }
            let title = L10n.translate(heading.key, language: language)
            if item.title != title { item.title = title }
            if submenu.title != title { submenu.title = title }
            localizeActions(submenu, language: language)
        }
        // The application's own menu also contains standard Hide and Quit items.
        if let appMenu = main.items.first?.submenu { localizeActions(appMenu, language: language) }
    }

    private static func localizeActions(_ menu: NSMenu, language: String) {
        let actions = [
            "performClose:": "menu.close", "cut:": "menu.cut", "copy:": "menu.copy",
            "paste:": "menu.paste", "pasteAsPlainText:": "menu.pastePlainText", "selectAll:": "menu.selectAll",
            "performMiniaturize:": "menu.minimize", "performZoom:": "menu.zoom", "arrangeInFront:": "menu.bringAllToFront",
            "hide:": "menu.hide", "hideOtherApplications:": "menu.hideOthers", "unhideAllApplications:": "menu.showAll",
            "terminate:": "退出 BibCiTeX", "showHelp:": "menu.appHelp"
        ]
        let windowCommands = Set(actions.keys).union(["undo:", "redo:", "toggleFullScreen:", "toggleToolbarShown:", "runToolbarCustomizationPalette:"])
        for item in menu.items {
            // AppKit populates the Window menu with user window names. These can
            // equal command labels, so recognize their role before matching text.
            let action = item.action.map(NSStringFromSelector)
            if item.representedObject is NSWindow
                || action == "makeKeyAndOrderFront:" || action == "_bringWindowToFront:"
                || (item.target is NSWindow && !windowCommands.contains(action ?? "")) { continue }
            let isServices = item.submenu != nil && ["Services", "服务"].contains(item.title)
            // SwiftUI can wrap standard actions in its own selector. Match the
            // localized title first, including stateful Full Screen / Toolbar
            // variants and Undo / Redo descriptions, then fall back to selectors.
            let title = L10n.nativeMenuTitle(item.title, language: language)
                ?? action.flatMap { actions[$0] }.map { L10n.translate($0, language: language) }
            if let title, item.title != title { item.title = title }
            if let submenu = item.submenu {
                if let title, submenu.title != title { submenu.title = title }
                if isServices {
                    // External service names belong to their providers. Only
                    // AppKit's own settings and empty-state labels are ours to translate.
                    for service in submenu.items where ["Services Settings…", "Services Settings...", "服务设置…", "No Services Apply", "没有服务可应用"].contains(service.title) {
                        if let title = L10n.nativeMenuTitle(service.title, language: language), service.title != title { service.title = title }
                    }
                } else {
                    localizeActions(submenu, language: language)
                }
            }
        }
    }
}
