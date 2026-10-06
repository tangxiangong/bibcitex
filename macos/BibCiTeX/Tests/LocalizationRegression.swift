import Foundation
import AppKit

@main
struct LocalizationRegression {
    @MainActor
    static func main() throws {
        precondition(L10n.resolvedLanguage("system", preferred: ["zh-Hans-CN"]) == "zh-Hans")
        precondition(L10n.resolvedLanguage("system", preferred: ["zh_CN"]) == "zh-Hans")
        precondition(L10n.resolvedLanguage("system", preferred: ["en-GB"]) == "en")
        precondition(L10n.resolvedLanguage("system", preferred: ["zh-Hant-TW"]) == "en")
        precondition(L10n.resolvedLanguage("invalid", preferred: ["fr-FR"]) == "en")
        precondition(L10n.resolvedLanguage("en", preferred: ["zh-CN"]) == "en")
        precondition(L10n.resolvedLanguage("zh-Hans", preferred: ["en-US"]) == "zh-Hans")
        precondition(L10n.translate("复制引用键", language: "en") == "Copy citation key")
        precondition(L10n.translate("复制引用键", language: "zh-Hans") == "复制引用键")
        precondition(L10n.translate("1 条文献", language: "en") == "1 reference")
        precondition(L10n.translate("{0} 条文献", language: "en", arguments: ["42"]) == "42 references")
        precondition(L10n.translate("{0}的操作", language: "en", arguments: ["文献 {0} β"]) == "Actions for 文献 {0} β")
        precondition(L10n.translate("metadata.editor", language: "en") == "Editor")
        precondition(L10n.translate("metadata.editor", language: "zh-Hans") == "编辑")
        precondition(L10n.translate("unknown key", language: "en") == "unknown key")
        precondition(L10n.translate("/tmp/文献.bib", language: "en") == "/tmp/文献.bib")
        precondition(L10n.translate("搜索", language: "unsupported") == "Search")
        struct KeyedError: Error, LocalizedMessageProviding {
            var localizedMessage: LocalizedMessage { LocalizedMessage(key: "引用键为空") }
        }
        let retainedError = LocalizedMessage(key: "粘贴失败：{0}", arguments: [LocalizedMessage(error: KeyedError())])
        precondition(retainedError.resolve(language: "en") == "Failed to paste: Citation key is empty")
        precondition(retainedError.resolve(language: "zh-Hans") == "粘贴失败：引用键为空")
        let diagnostic = LocalizedMessage(key: "无法打开文件：{0}", arguments: [LocalizedMessage(literal: "/tmp/搜索 {0}.bib")])
        precondition(diagnostic.resolve(language: "en") == "Unable to open file: /tmp/搜索 {0}.bib")
        testSystemMenus()
        let en = try catalog("en"), zh = try catalog("zh-Hans")
        precondition(Set(en.keys) == Set(zh.keys) && en.count > 100)
        for key in en.keys {
            precondition(!en[key]!.isEmpty && !zh[key]!.isEmpty)
            if (key.hasPrefix("menu.") || key.hasPrefix("undoAction.")) && !en[key]!.contains("{0}") {
                precondition(L10n.nativeMenuTitle(en[key]!, language: "zh-Hans") == zh[key]!, "English menu coverage: \(key)")
                precondition(L10n.nativeMenuTitle(zh[key]!, language: "en") == en[key]!, "Chinese menu coverage: \(key)")
            }
            let pattern = try NSRegularExpression(pattern: #"\{[0-9]+\}"#)
            func placeholders(_ text: String) -> [String] {
                pattern.matches(in: text, range: NSRange(text.startIndex..., in: text))
                    .map { String(text[Range($0.range, in: text)!]) }.sorted()
            }
            precondition(placeholders(en[key]!) == placeholders(zh[key]!))
        }
        print("PASS Swift localization: bundled catalogs, language resolution, fallback, placeholders and singular/plural counts")
    }
    @MainActor
    private static func testSystemMenus() {
        let main = NSMenu()
        let application = NSMenuItem(title: "BibCiTeX", action: nil, keyEquivalent: "")
        application.submenu = NSMenu(title: "BibCiTeX")
        main.addItem(application)
        let quit = application.submenu!.addItem(withTitle: "退出 BibCiTeX", action: NSSelectorFromString("terminate:"), keyEquivalent: "q")
        let close = NSMenuItem(title: "关闭窗口", action: NSSelectorFromString("performClose:"), keyEquivalent: "w")
        close.isEnabled = false
        let titles = ["文件", "编辑", "显示", "窗口", "帮助"]
        for title in titles {
            let item = NSMenuItem(title: title, action: nil, keyEquivalent: "")
            item.submenu = NSMenu(title: title)
            main.addItem(item)
        }
        main.items[1].submenu!.addItem(close)
        let proxy = NSSelectorFromString("swiftUICommand:")
        func add(_ title: String, to index: Int, key: String = "") -> NSMenuItem {
            main.items[index].submenu!.addItem(withTitle: title, action: proxy, keyEquivalent: key)
        }
        let undo = add("撤销键入", to: 2, key: "z")
        undo.isEnabled = false
        let redo = add("重做粘贴", to: 2)
        let plainUndo = add("撤销", to: 2)
        let copy = add("拷贝", to: 2, key: "c")
        let fullscreen = add("进入全屏幕", to: 3)
        let toolbar = add("隐藏工具栏", to: 3)
        let customize = add("自定义工具栏...", to: 3)
        let move = add("移动与调整大小", to: 4)
        move.submenu = NSMenu(title: move.title)
        let position = move.submenu!.addItem(withTitle: "左上", action: proxy, keyEquivalent: "")
        let speech = add("语音", to: 2)
        speech.submenu = NSMenu(title: speech.title)
        let speaking = speech.submenu!.addItem(withTitle: "开始朗读", action: proxy, keyEquivalent: "")
        let services = application.submenu!.addItem(withTitle: "服务", action: nil, keyEquivalent: "")
        services.submenu = NSMenu(title: "服务")
        let serviceName = services.submenu!.addItem(withTitle: "拷贝", action: proxy, keyEquivalent: "")
        let serviceSettings = services.submenu!.addItem(withTitle: "服务设置…", action: proxy, keyEquivalent: "")
        let dynamicDisplay = add("移到“Display 文献 β”", to: 4)
        let windowTitle = main.items[4].submenu!.addItem(withTitle: "文件 — Research", action: NSSelectorFromString("makeKeyAndOrderFront:"), keyEquivalent: "")
        let custom = NSMenuItem(title: "文献", action: nil, keyEquivalent: "")
        custom.submenu = NSMenu(title: "文献"); main.addItem(custom)
        MainMenuLocalizer.localize(main, language: "en")
        precondition(Array(main.items[1...5]).map(\.title) == ["File", "Edit", "View", "Window", "Help"])
        precondition(Array(main.items[1...5]).map { $0.submenu!.title } == ["File", "Edit", "View", "Window", "Help"])
        precondition(close.title == "Close Window" && close.action == NSSelectorFromString("performClose:"))
        precondition(close.keyEquivalent == "w" && !close.isEnabled)
        precondition(quit.title == "Quit BibCiTeX" && quit.keyEquivalent == "q")
        precondition(windowTitle.title == "文件 — Research" && custom.title == "文献")
        precondition(undo.title == "Undo Typing" && redo.title == "Redo Paste" && plainUndo.title == "Undo")
        precondition(copy.title == "Copy" && copy.keyEquivalent == "c" && copy.action == proxy)
        precondition(!undo.isEnabled && undo.keyEquivalent == "z")
        precondition(fullscreen.title == "Enter Full Screen" && toolbar.title == "Hide Toolbar")
        precondition(customize.title == "Customize Toolbar…")
        precondition(move.title == "Move & Resize" && move.submenu!.title == "Move & Resize" && position.title == "Top Left")
        precondition(speech.title == "Speech" && speaking.title == "Start Speaking")
        precondition(services.title == "Services" && serviceSettings.title == "Services Settings…" && serviceName.title == "拷贝")
        precondition(dynamicDisplay.title == "Move to Display 文献 β")
        fullscreen.title = "退出全屏幕"; toolbar.title = "显示工具栏"
        MainMenuLocalizer.localize(main, language: "en")
        precondition(fullscreen.title == "Exit Full Screen" && toolbar.title == "Show Toolbar")
        MainMenuLocalizer.localize(main, language: "zh-Hans")
        precondition(Array(main.items[1...5]).map(\.title) == titles)
        precondition(close.title == "关闭窗口" && quit.title == "退出 BibCiTeX")
        precondition(undo.title == "撤销键入" && redo.title == "重做粘贴" && plainUndo.title == "撤销")
        precondition(fullscreen.title == "退出全屏幕" && toolbar.title == "显示工具栏")
        precondition(move.title == "移动与调整大小" && position.title == "左上")
        precondition(dynamicDisplay.title == "移到“Display 文献 β”")
        precondition(serviceName.title == "拷贝")
        // Simulate a late SwiftUI menu regeneration; refresh must use the current preference.
        let localizer = MainMenuLocalizer(menu: { main })
        localizer.start()
        main.items[2].title = L10n.language == "en" ? "编辑" : "Edit"
        precondition(main.items[2].title == L10n.translate("menu.edit", language: L10n.language))
        let late = main.items[3].submenu!.addItem(withTitle: L10n.language == "en" ? "进入全屏幕" : "Enter Full Screen", action: proxy, keyEquivalent: "")
        precondition(late.title == (L10n.language == "en" ? "Enter Full Screen" : "进入全屏幕"))
        localizer.stop()
        print("PASS recursive submenus, SwiftUI-wrapped commands, Undo/Redo, Full Screen/Toolbar states and late menu insertion")
        print("PASS native menu headings, language round trip, regenerated menus, actions, shortcuts and user window titles")
    }
    private static func catalog(_ language: String) throws -> [String: String] {
        let url = Bundle.main.url(forResource: language, withExtension: "json")!
        return try JSONDecoder().decode([String: String].self, from: Data(contentsOf: url))
    }
}
