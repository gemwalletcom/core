// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Transfer",
    platforms: [
        .iOS(.v17),
        .macOS(.v15)
    ],
    products: [
        .library(
            name: "Transfer",
            targets: ["Transfer"]),
        .library(
            name: "TransferTestKit",
            targets: ["TransferTestKit"]),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Keystore", path: "../../Packages/Keystore"),
        .package(name: "Blockchain", path: "../../Packages/Blockchain"),
        .package(name: "Signer", path: "../../Packages/Signer"),
        .package(name: "Preferences", path: "../../Packages/Preferences"),
        .package(name: "Validators", path: "../../Packages/Validators"),
        .package(name: "Store", path: "../../Packages/Store"),

        .package(name: "Stake", path: "../Stake"),
        .package(name: "WalletConnector", path: "../WalletConnector"),
        .package(name: "InfoSheet", path: "../InfoSheet"),
        .package(name: "FiatConnect", path: "../FiatConnect"),
        .package(name: "Swap", path: "../Swap"),
        .package(name: "Perpetuals", path: "../Perpetuals"),
        .package(name: "EventPresenterService", path: "../EventPresenterService"),

        .package(name: "ChainServices", path: "../../Packages/ChainServices"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "GemAPI", path: "../../Packages/GemAPI")
    ],
    targets: [
        .target(
            name: "Transfer",
            dependencies: [
                "Primitives",
                "Formatters",
                "Localization",
                "Style",
                "Components",
                "PrimitivesComponents",
                "GemstonePrimitives",
                "Keystore",
                "Blockchain",
                "Signer",
                "Preferences",
                "Validators",

                "Stake",
                "WalletConnector",
                "InfoSheet",
                "FiatConnect",
                "Swap",
                "Perpetuals",
                "EventPresenterService",

                .product(name: "ChainService", package: "ChainServices"),
                .product(name: "WalletService", package: "FeatureServices"),
                .product(name: "NodeService", package: "ChainServices"),
                .product(name: "TransactionStateService", package: "FeatureServices"),
                .product(name: "ScanService", package: "ChainServices"),
                .product(name: "BalanceService", package: "FeatureServices"),
                .product(name: "AssetsService", package: "FeatureServices"),
                .product(name: "PriceService", package: "FeatureServices"),
                .product(name: "EarnService", package: "FeatureServices"),
                .product(name: "PerpetualService", package: "FeatureServices"),
                .product(name: "ExplorerService", package: "ChainServices"),
                .product(name: "NameService", package: "ChainServices"),
                .product(name: "AddressNameService", package: "FeatureServices"),
                .product(name: "ActivityService", package: "FeatureServices")
            ],
            path: "Sources"
        ),
        .target(
            name: "TransferTestKit",
            dependencies: [
                "Transfer",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "TestKit"
        ),
        .testTarget(
            name: "TransferTests",
            dependencies: [
                "Transfer",
                "TransferTestKit",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "BlockchainTestKit", package: "Blockchain"),
                .product(name: "ScanServiceTestKit", package: "ChainServices"),
                .product(name: "SwapServiceTestKit", package: "FeatureServices"),
                .product(name: "KeystoreTestKit", package: "Keystore"),
                .product(name: "WalletServiceTestKit", package: "FeatureServices"),
                .product(name: "NameServiceTestKit", package: "ChainServices"),
                .product(name: "NodeServiceTestKit", package: "ChainServices"),
                .product(name: "PriceServiceTestKit", package: "FeatureServices"),
                .product(name: "AssetsServiceTestKit", package: "FeatureServices"),
                .product(name: "BalanceServiceTestKit", package: "FeatureServices"),
                .product(name: "TransactionStateServiceTestKit", package: "FeatureServices"),
                .product(name: "AddressNameServiceTestKit", package: "FeatureServices"),
                .product(name: "ActivityServiceTestKit", package: "FeatureServices"),
                .product(name: "GemAPITestKit", package: "GemAPI"),
                .product(name: "ChainServiceTestKit", package: "ChainServices"),
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "StakeServiceTestKit", package: "ChainServices"),
                .product(name: "NFTServiceTestKit", package: "FeatureServices"),
                .product(name: "SignerTestKit", package: "Signer"),
                .product(name: "EarnServiceTestKit", package: "FeatureServices"),
                .product(name: "EventPresenterServiceTestKit", package: "EventPresenterService"),
            ],
            path: "Tests"
        ),
    ]
)
