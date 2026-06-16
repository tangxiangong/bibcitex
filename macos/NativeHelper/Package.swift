// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "NativeHelper",
    platforms: [
        .macOS(.v12),
    ],
    products: [
        .library(
            name: "NativeHelper",
            type: .static,
            targets: ["NativeHelper"],
        ),
    ],
    dependencies: [
        .package(
            url: "https://github.com/colinc86/LaTeXSwiftUI",
            from: "1.5.0",
        ),
    ],
    targets: [
        .target(
            name: "NativeHelper",
            dependencies: ["LaTeXSwiftUI"],
            path: "Sources/NativeHelper",
            linkerSettings: [
                .linkedFramework("AppKit"),
                .linkedFramework("SwiftUI"),
            ],
        ),
    ],
)
