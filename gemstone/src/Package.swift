// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "Gemstone",
    platforms: [
        .iOS(.v17),
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
