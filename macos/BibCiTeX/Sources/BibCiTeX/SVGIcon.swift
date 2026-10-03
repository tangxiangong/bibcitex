import AppKit
import SwiftUI
import SwiftDraw

/// Icons are standalone SVG resources, loaded from the application resource bundle.
struct SVGIcon: View {
    let name: String
    let size: CGFloat

    init(_ name: String, size: CGFloat = 18) {
        self.name = name
        self.size = size
    }

    var body: some View {
        if let image = SVGImages.images[name] {
            Image(nsImage: image)
                .renderingMode(.template)
                .resizable()
                .scaledToFit()
                .frame(width: size, height: size)
                .accessibilityHidden(true)
        }
    }
}

enum SVGImages {
    static let images: [String: NSImage] = {
        var result: [String: NSImage] = [:]
        for name in ["search", "user", "alert", "library", "tag", "chevronDown", "x", "fileText", "clipboard", "link", "panelRightClose", "settings", "download", "externalLink", "book", "check", "panelLeftOpen", "chevronRight", "info", "add", "copy", "refresh", "folderOpen", "trash", "sun", "folderAdd", "panelRightOpen", "panelLeftClose", "calendar", "chevronLeft", "moon"] {
            result[name] = NSImage(svgNamed: name + ".svg", in: .appResources)
        }
        return result
    }()
}
