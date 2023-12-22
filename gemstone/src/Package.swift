// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "Gemstone",
    platforms: [
        .iOS(.v13),
    ],
    products: [
        .library(
            name: "Gemstone",
            targets: ["Gemstone", "GemstoneFFI"]
        ),
    ],
    dependencies: [
    ],
    targets: [
        .target(
            name: "Gemstone",
            dependencies: ["GemstoneFFI"]
        ),
        .binaryTarget(name: "GemstoneFFI", path: "Sources/GemstoneFFI.xcframework"),
    ]
)
