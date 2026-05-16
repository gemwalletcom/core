// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "GemTest",
    platforms: [
        .iOS(.v17)
    ],
    products: [
        .library(
            name: "GemTest",
            targets: ["GemTest"]
        )
    ],
    dependencies: [
        .package(path: "../Packages/Gemstone"),
        .package(url: "https://github.com/attaswift/BigInt.git", from: "5.1.0")
    ],
    targets: [
        .target(
            name: "GemTest",
            dependencies: [
                .product(name: "Gemstone", package: "Gemstone"),
                .product(name: "BigInt", package: "BigInt")
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
