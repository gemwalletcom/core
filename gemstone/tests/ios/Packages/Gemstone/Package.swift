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
            targets: ["Gemstone"]
        )
    ],
    dependencies: [
    ],
    targets: [
        .target(
            name: "Gemstone",
            dependencies: ["GemstoneFFI"],
            swiftSettings: [
                .swiftLanguageMode(.v5)
            ]
        ),
        .target(
            name: "GemstoneFFI",
            path: "Sources/GemstoneFFI",
            publicHeadersPath: "include"
        )
    ]
)
