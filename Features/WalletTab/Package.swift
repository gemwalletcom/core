// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "WalletTab",
    platforms: [
        .iOS(.v17),
        .macOS(.v15)
    ],
    products: [
        .library(
            name: "WalletTab",
            targets: ["WalletTab"]
        ),
        .library(
            name: "WalletTabTestKit",
            targets: ["WalletTabTestKit"]
        )
    ],
    dependencies: [
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "InfoSheet", path: "../InfoSheet"),

        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "Preferences", path: "../../Packages/Preferences"),
        .package(name: "ChainServices", path: "../../Packages/ChainServices"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "GemAPI", path: "../../Packages/GemAPI"),
        .package(name: "Perpetuals", path: "../Perpetuals"),
        .package(name: "Recents", path: "../Recents"),
    ],
    targets: [
        .target(
            name: "WalletTab",
            dependencies: [
                "Primitives",
                "Localization",
                "Style",
                "Components",
                "PrimitivesComponents",
                "InfoSheet",
                "Store",
                "Preferences",
                .product(name: "BalanceService", package: "FeatureServices"),
                .product(name: "BannerService", package: "FeatureServices"),
                .product(name: "WalletService", package: "FeatureServices"),
                .product(name: "ActivityService", package: "FeatureServices"),
                .product(name: "AssetsService", package: "FeatureServices"),
                .product(name: "PerpetualService", package: "FeatureServices"),
                .product(name: "DiscoverAssetsService", package: "FeatureServices"),
                .product(name: "PriceService", package: "FeatureServices"),
                "Perpetuals",
                "Recents"
            ],
            path: "Sources"
        ),
        .target(
            name: "WalletTabTestKit",
            dependencies: [
                .product(name: "BalanceServiceTestKit", package: "FeatureServices"),
                .product(name: "BannerServiceTestKit", package: "FeatureServices"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "WalletServiceTestKit", package: "FeatureServices"),
                .product(name: "AssetsServiceTestKit", package: "FeatureServices"),
                .product(name: "ActivityServiceTestKit", package: "FeatureServices"),
                .product(name: "PerpetualServiceTestKit", package: "FeatureServices"),
                .product(name: "DiscoverAssetsServiceTestKit", package: "FeatureServices"),
                "WalletTab"
            ],
            path: "TestKit"
        ),
        .testTarget(
            name: "WalletTabTests",
            dependencies: [
                .product(name: "BannerServiceTestKit", package: "FeatureServices"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "WalletServiceTestKit", package: "FeatureServices"),
                .product(name: "ChainServiceTestKit", package: "ChainServices"),
                .product(name: "PriceServiceTestKit", package: "FeatureServices"),
                .product(name: "AssetsServiceTestKit", package: "FeatureServices"),
                .product(name: "BalanceServiceTestKit", package: "FeatureServices"),
                .product(name: "TransactionStateServiceTestKit", package: "FeatureServices"),
                .product(name: "PerpetualServiceTestKit", package: "FeatureServices"),
                .product(name: "GemAPITestKit", package: "GemAPI"),
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "ActivityServiceTestKit", package: "FeatureServices"),
                "WalletTab",
                "WalletTabTestKit"
            ]
        ),
    ]
)
