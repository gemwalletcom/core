// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Perpetuals",
    platforms: [
        .iOS(.v17),
        .macOS(.v15)
    ],
    products: [
        .library(
            name: "Perpetuals",
            targets: ["Perpetuals"]),
        .library(
            name: "PerpetualsTestKit",
            targets: ["PerpetualsTestKit"]),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "Preferences", path: "../../Packages/Preferences"),
        .package(name: "InfoSheet", path: "../InfoSheet"),
        .package(name: "ChainServices", path: "../../Packages/ChainServices"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Recents", path: "../Recents"),
    ],
    targets: [
        .target(
            name: "Perpetuals",
            dependencies: [
                "Primitives",
                "PrimitivesComponents",
                "GemstonePrimitives",
                "Components",
                "Style",
                "Localization",
                .product(name: "PerpetualService", package: "FeatureServices"),
                .product(name: "TransactionsService", package: "FeatureServices"),
                .product(name: "ActivityService", package: "FeatureServices"),
                "Store",
                "Formatters",
                "Preferences",
                "InfoSheet",
                .product(name: "ExplorerService", package: "ChainServices"),
                "Recents"
            ],
            path: "Sources"
        ),
        .target(
            name: "PerpetualsTestKit",
            dependencies: [
                "Perpetuals",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "PerpetualService", package: "FeatureServices"),
                .product(name: "PerpetualServiceTestKit", package: "FeatureServices"),
                .product(name: "TransactionsService", package: "FeatureServices"),
                .product(name: "TransactionsServiceTestKit", package: "FeatureServices"),
                .product(name: "ActivityService", package: "FeatureServices"),
                .product(name: "ActivityServiceTestKit", package: "FeatureServices"),
            ],
            path: "TestKit"
        ),
        .testTarget(
            name: "PerpetualsTests",
            dependencies: [
                "Perpetuals",
                "PerpetualsTestKit",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "Tests"
        ),
    ],
    swiftLanguageModes: [.v6]
)
