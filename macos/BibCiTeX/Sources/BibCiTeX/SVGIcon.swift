import AppKit
import SwiftUI
import SwiftDraw

/// Prefer system symbols available on this OS, retaining bundled SVG fallbacks.
struct NativeIcon: View {
    let name: String
    let size: CGFloat

    init(_ name: String, size: CGFloat = 18) {
        self.name = name
        self.size = size
    }

    var body: some View {
        if let symbol = NativeImages.symbols[name] {
            Image(systemName: symbol)
                .resizable()
                .scaledToFit()
                .frame(width: size, height: size)
                .accessibilityHidden(true)
        } else if let image = SVGImages.images[name] {
            Image(nsImage: image)
                .renderingMode(.template)
                .resizable()
                .scaledToFit()
                .frame(width: size, height: size)
                .accessibilityHidden(true)
        }
    }
}

enum NativeImages {
    // Resolve each symbol independently: newer symbols may be absent on older macOS.
    static let symbols: [String: String] = {
        let candidates = [
            "more": "ellipsis",
            "pin": "pin",
            "rename": "pencil",
            "search": "magnifyingglass",
            "user": "person",
            "alert": "exclamationmark.triangle",
            "library": "books.vertical",
            "tag": "tag",
            "chevronDown": "chevron.down",
            "x": "xmark",
            "fileText": "doc.text",
            "clipboard": "doc.on.clipboard",
            "link": "link",
            "panelRightClose": "sidebar.right",
            "settings": "gearshape",
            "download": "arrow.down.to.line",
            "externalLink": "arrow.up.right.square",
            "book": "book",
            "check": "checkmark",
            "panelLeftOpen": "sidebar.left",
            "chevronRight": "chevron.right",
            "info": "info.circle",
            "add": "plus",
            "copy": "doc.on.doc",
            "refresh": "arrow.clockwise",
            "folderOpen": "folder",
            "trash": "trash",
            "sun": "sun.max",
            "folderAdd": "folder.badge.plus",
            "panelRightOpen": "sidebar.right",
            "panelLeftClose": "sidebar.left",
            "calendar": "calendar",
            "chevronLeft": "chevron.left",
            "moon": "moon",
        ]
        return candidates.filter {
            NSImage(systemSymbolName: $0.value, accessibilityDescription: nil) != nil
        }
    }()

    static func image(named name: String) -> Image? {
        if let symbol = symbols[name] {
            return Image(systemName: symbol)
        }
        return SVGImages.images[name].map { Image(nsImage: $0).renderingMode(.template) }
    }
}

enum SVGImages {
    static let images: [String: NSImage] = {
        var result: [String: NSImage] = [:]
        for name in ["more", "pin", "rename", "search", "user", "alert", "library", "tag", "chevronDown", "x", "fileText", "clipboard", "link", "panelRightClose", "settings", "download", "externalLink", "book", "check", "panelLeftOpen", "chevronRight", "info", "add", "copy", "refresh", "folderOpen", "trash", "sun", "folderAdd", "panelRightOpen", "panelLeftClose", "calendar", "chevronLeft", "moon"] {
            result[name] = NSImage(svgNamed: name + ".svg", in: .appResources)
        }
        return result
    }()
}
