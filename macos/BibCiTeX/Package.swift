// swift-tools-version: 5.10
import PackageDescription
let package = Package(
    name: "BibCiTeX",
    platforms: [.macOS(.v13)],
    products: [.executable(name: "BibCiTeX", targets: ["BibCiTeX"])],
    dependencies: [
        .package(url: "https://github.com/swhitty/SwiftDraw", exact: "0.27.0"),
        .package(url: "https://github.com/colinc86/LaTeXSwiftUI", exact: "1.5.0"),
        .package(url: "https://github.com/sparkle-project/Sparkle", exact: "2.10.0"),
    ],
    targets: [
        .systemLibrary(name: "BibCiTeXCoreFFI", path: "Generated/BibCiTeXCoreFFI"),
        .target(name: "BibCiTeXCore", dependencies: ["BibCiTeXCoreFFI"], path: "Generated/BibCiTeXCore"),
        .executableTarget(
            name: "BibCiTeX",
            dependencies: ["BibCiTeXCore", "LaTeXSwiftUI", "SwiftDraw", .product(name: "Sparkle", package: "Sparkle")],
            path: "Sources/BibCiTeX",
            resources: [.process("Resources")],
            linkerSettings: [.linkedFramework("Carbon"), .linkedFramework("AppKit"), .linkedFramework("ApplicationServices"), .linkedFramework("CoreGraphics"), .linkedFramework("Foundation"), .linkedLibrary("iconv")]
        ),
    ]
)
