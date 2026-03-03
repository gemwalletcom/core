// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Stake",
    platforms: [
        .iOS(.v17),
        .macOS(.v15)
    ],
    products: [
        .library(
            name: "Stake",
            targets: ["Stake"]),
        .library(
            name: "StakeTestKit",
            targets: ["StakeTestKit"]),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "ChainServices", path: "../../Packages/ChainServices"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "Preferences", path: "../../Packages/Preferences"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "InfoSheet", path: "../InfoSheet"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "Style", path: "../../Packages/Style"),
    ],
    targets: [
        .target(
            name: "Stake",
            dependencies: [
                "Primitives",
                "Components",
                "GemstonePrimitives",
                "Localization",
                .product(name: "StakeService", package: "ChainServices"),
                .product(name: "ExplorerService", package: "ChainServices"),
                .product(name: "EarnService", package: "FeatureServices"),
                "Preferences",
                "Store",
                "InfoSheet",
                "PrimitivesComponents",
                "Formatters",
                "Style",
            ],
            path: "Sources"
        ),
        .target(
            name: "StakeTestKit",
            dependencies: [
                "Stake",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "TestKit"
        ),
        .testTarget(
            name: "StakeTests",
            dependencies: [
                "StakeTestKit",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "StakeServiceTestKit", package: "ChainServices"),
                "Stake"
            ],
            path: "Tests"
        ),
    ]
)
