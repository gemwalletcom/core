// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Assets",
    platforms: [
        .iOS(.v17),
        .macOS(.v15)
    ],
    products: [
        .library(
            name: "Assets",
            targets: ["Assets"]),
        .library(
            name: "AssetsTestKit",
            targets: ["AssetsTestKit"]),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "Preferences", path: "../../Packages/Preferences"),
        .package(name: "Blockchain", path: "../../Packages/Blockchain"),
        .package(name: "InfoSheet", path: "../InfoSheet"),
        .package(name: "QRScanner", path: "../QRScanner"),
        .package(name: "Recents", path: "../Recents"),
        .package(name: "ChainServices", path: "../../Packages/ChainServices"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices")
    ],
    targets: [
        .target(
            name: "Assets",
            dependencies: [
                "Primitives",
                "Formatters",
                "Localization",
                "Style",
                "Components",
                "PrimitivesComponents",
                "Store",
                "Preferences",
                "Blockchain",
                "InfoSheet",
                "QRScanner",
                "Recents",
                .product(name: "PriceAlertService", package: "FeatureServices"),
                .product(name: "ExplorerService", package: "ChainServices"),
                .product(name: "AssetsService", package: "FeatureServices"),
                .product(name: "TransactionsService", package: "FeatureServices"),
                .product(name: "BalanceService", package: "FeatureServices"),
                .product(name: "PriceService", package: "FeatureServices"),
                .product(name: "BannerService", package: "FeatureServices"),
                .product(name: "ChainService", package: "ChainServices"),
                .product(name: "ActivityService", package: "FeatureServices")
            ],
            path: "Sources"
        ),
        .target(
            name: "AssetsTestKit",
            dependencies: [
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "BalanceServiceTestKit", package: "FeatureServices"),
                .product(name: "AssetsServiceTestKit", package: "FeatureServices"),
                .product(name: "PriceAlertServiceTestKit", package: "FeatureServices"),
                .product(name: "PriceServiceTestKit", package: "FeatureServices"),
                .product(name: "ActivityServiceTestKit", package: "FeatureServices"),
                "Components",
                "Assets"
            ],
            path: "TestKit"
        ),
        .testTarget(
            name: "AssetsTests",
            dependencies: [
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "AssetsServiceTestKit", package: "FeatureServices"),
                .product(name: "BalanceServiceTestKit", package: "FeatureServices"),
                .product(name: "TransactionsServiceTestKit", package: "FeatureServices"),
                .product(name: "PriceServiceTestKit", package: "FeatureServices"),
                .product(name: "PriceAlertServiceTestKit", package: "FeatureServices"),
                .product(name: "BannerServiceTestKit", package: "FeatureServices"),
                "AssetsTestKit"
            ]
        )
    ]
)
