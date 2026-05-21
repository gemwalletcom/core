// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "GemTest",
    platforms: [
        .iOS(.v17), .macOS(.v15)
    ],
    products: [
        .library(
            name: "GemTest",
            targets: ["GemTest"]
        )
    ],
    dependencies: [
        .package(path: "../Packages/Gemstone")
    ],
    targets: [
        .target(
            name: "GemTest",
            dependencies: [
                .product(name: "Gemstone", package: "Gemstone")
            ],
            path: "GemTest",
            exclude: [
                "Assets.xcassets",
                "GemTestApp.swift",
                "Preview Content"
            ],
            swiftSettings: [
                .swiftLanguageMode(.v5)
            ]
        ),
        .testTarget(
            name: "GemTestTests",
            dependencies: ["GemTest"],
            path: "GemTestTests",
            swiftSettings: [
                .swiftLanguageMode(.v5)
            ]
        )
    ]
)
