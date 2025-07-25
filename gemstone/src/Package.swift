// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Gemstone",
    platforms: [
        .iOS(.v17), .macOS(.v15)
    ],
    products: [
        .library(
            name: "Gemstone",
            targets: ["Gemstone", "GemstoneFFI"]
        )
    ],
    dependencies: [
    ],
    targets: [
        .target(
            name: "Gemstone",
            dependencies: ["GemstoneFFI"],
            swiftSettings: [
                .swiftLanguageMode(.v5) // TODO: - remove when GemstoneFFI will support swift6 fully
            ]
        ),
        .binaryTarget(name: "GemstoneFFI", path: "Sources/GemstoneFFI.xcframework")
    ]
)
