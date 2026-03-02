// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "FiatConnect",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "FiatConnect",
            targets: ["FiatConnect"]
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "GemAPI", path: "../../Packages/GemAPI"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
    ],
    targets: [
        .target(
            name: "FiatConnect",
            dependencies: [
                "Primitives",
                "Formatters",
                "GemAPI",
                "Style",
                "Components",
                "Localization",
                "Store",
                "PrimitivesComponents",
            ],
            path: "Sources"
        ),
        .testTarget(
            name: "FiatConnectTests",
            dependencies: [
                "FiatConnect",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "Tests"
        ),
    ]
)
