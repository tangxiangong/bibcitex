import AppKit

extension Bundle {
    static var appResources: Bundle {
        #if SWIFT_PACKAGE
        return .module
        #else
        return .main
        #endif
    }
}

enum AppImages {
    static let logo: NSImage? = Bundle.appResources.url(forResource: "favicon", withExtension: "png")
        .flatMap { NSImage(contentsOf: $0) }
}
