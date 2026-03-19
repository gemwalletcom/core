// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "FeatureServices",
    platforms: [
        .iOS(.v17),
        .macOS(.v15)
    ],
    products: [
        .library(name: "PerpetualService", targets: ["PerpetualService"]),
        .library(name: "PerpetualServiceTestKit", targets: ["PerpetualServiceTestKit"]),
        .library(name: "BalanceService", targets: ["BalanceService"]),
        .library(name: "BalanceServiceTestKit", targets: ["BalanceServiceTestKit"]),
        .library(name: "BannerService", targets: ["BannerService"]),
        .library(name: "BannerServiceTestKit", targets: ["BannerServiceTestKit"]),
        .library(name: "NFTService", targets: ["NFTService"]),
        .library(name: "NFTServiceTestKit", targets: ["NFTServiceTestKit"]),
        .library(name: "AvatarService", targets: ["AvatarService"]),
        .library(name: "PriceService", targets: ["PriceService"]),
        .library(name: "PriceServiceTestKit", targets: ["PriceServiceTestKit"]),
        .library(name: "StreamService", targets: ["StreamService"]),
        .library(name: "StreamServiceTestKit", targets: ["StreamServiceTestKit"]),
        .library(name: "PriceAlertService", targets: ["PriceAlertService"]),
        .library(name: "PriceAlertServiceTestKit", targets: ["PriceAlertServiceTestKit"]),
        .library(name: "TransactionStateService", targets: ["TransactionStateService"]),
        .library(name: "TransactionStateServiceTestKit", targets: ["TransactionStateServiceTestKit"]),
        .library(name: "TransactionsService", targets: ["TransactionsService"]),
        .library(name: "TransactionsServiceTestKit", targets: ["TransactionsServiceTestKit"]),
        .library(name: "DiscoverAssetsService", targets: ["DiscoverAssetsService"]),
        .library(name: "DiscoverAssetsServiceTestKit", targets: ["DiscoverAssetsServiceTestKit"]),
        .library(name: "SwapService", targets: ["SwapService"]),
        .library(name: "SwapServiceTestKit", targets: ["SwapServiceTestKit"]),
        .library(name: "AssetsService", targets: ["AssetsService"]),
        .library(name: "AssetsServiceTestKit", targets: ["AssetsServiceTestKit"]),
        .library(name: "WalletSessionService", targets: ["WalletSessionService"]),
        .library(name: "WalletSessionServiceTestKit", targets: ["WalletSessionServiceTestKit"]),
        .library(name: "WalletService", targets: ["WalletService"]),
        .library(name: "WalletServiceTestKit", targets: ["WalletServiceTestKit"]),
        .library(name: "AppService", targets: ["AppService"]),
        .library(name: "AppServiceTestKit", targets: ["AppServiceTestKit"]),
        .library(name: "DeviceService", targets: ["DeviceService"]),
        .library(name: "DeviceServiceTestKit", targets: ["DeviceServiceTestKit"]),
        .library(name: "NotificationService", targets: ["NotificationService"]),
        .library(name: "NotificationServiceTestKit", targets: ["NotificationServiceTestKit"]),
        .library(name: "AddressNameService", targets: ["AddressNameService"]),
        .library(name: "AddressNameServiceTestKit", targets: ["AddressNameServiceTestKit"]),
        .library(name: "ActivityService", targets: ["ActivityService"]),
        .library(name: "ActivityServiceTestKit", targets: ["ActivityServiceTestKit"]),
        .library(name: "RewardsService", targets: ["RewardsService"]),
        .library(name: "RewardsServiceTestKit", targets: ["RewardsServiceTestKit"]),
        .library(name: "AuthService", targets: ["AuthService"]),
        .library(name: "AuthServiceTestKit", targets: ["AuthServiceTestKit"]),
        .library(name: "ConnectionsService", targets: ["ConnectionsService"]),
        .library(name: "ConnectionsServiceTestKit", targets: ["ConnectionsServiceTestKit"]),
        .library(name: "ContactService", targets: ["ContactService"]),
        .library(name: "EarnService", targets: ["EarnService"]),
        .library(name: "EarnServiceTestKit", targets: ["EarnServiceTestKit"]),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "Store", path: "../Store"),
        .package(name: "Blockchain", path: "../Blockchain"),
        .package(name: "ChainServices", path: "../ChainServices"),
        .package(name: "GemAPI", path: "../GemAPI"),
        .package(name: "Preferences", path: "../Preferences"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "Signer", path: "../Signer"),
        .package(name: "Keystore", path: "../Keystore"),
        .package(name: "Formatters", path: "../Formatters"),
        .package(name: "SwiftHTTPClient", path: "../SwiftHTTPClient"),
        .package(name: "NativeProviderService", path: "../NativeProviderService"),
    ],
    targets: [
        .target(
            name: "PerpetualService",
            dependencies: [
                "Primitives",
                "Store",
                "Blockchain",
                "Formatters",
                "Preferences",
                .product(name: "ChainService", package: "ChainServices"),
                .product(name: "WebSocketClient", package: "SwiftHTTPClient"),
            ],
            path: "PerpetualService",
            exclude: ["Tests", "TestKit"]
        ),
        .target(
            name: "PerpetualServiceTestKit",
            dependencies: [
                "PerpetualService",
                "Primitives",
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
            ],
            path: "PerpetualService/TestKit"
        ),
        .target(
            name: "BalanceService",
            dependencies: [
                "Primitives",
                "Store",
                .product(name: "ChainService", package: "ChainServices"),
                "AssetsService",
                "PriceService",
                "Formatters"
            ],
            path: "BalanceService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "BalanceServiceTestKit",
            dependencies: [
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "ChainServiceTestKit", package: "ChainServices"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                "AssetsServiceTestKit",
                "PriceServiceTestKit",
                "BalanceService",
                "Primitives"
            ],
            path: "BalanceService/TestKit"
        ),
        .target(
            name: "BannerService",
            dependencies: [
                "Primitives",
                "Store",
                "NotificationService",
                "Preferences"
            ],
            path: "BannerService",
            exclude: ["TestKit", "Tests"]
        ),
        .target(
            name: "BannerServiceTestKit",
            dependencies: [
                .product(name: "StoreTestKit", package: "Store"),
                "NotificationServiceTestKit",
                "BannerService"
            ],
            path: "BannerService/TestKit"
        ),
        .target(
            name: "NFTService",
            dependencies: [
                "Primitives",
                "Store",
                "GemAPI",
            ],
            path: "NFTService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "NFTServiceTestKit",
            dependencies: [
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "GemAPITestKit", package: "GemAPI"),
                "NFTService",
            ],
            path: "NFTService/TestKit"
        ),
        .target(
            name: "AvatarService",
            dependencies: [
                "Primitives",
                "Store"
            ],
            path: "AvatarService",
            exclude: ["Tests", "TestKit"]
        ),
        .target(
            name: "PriceService",
            dependencies: [
                "Primitives",
                "Store",
                "GemAPI",
            ],
            path: "PriceService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "PriceServiceTestKit",
            dependencies: [
                "Primitives",
                "PriceService",
                .product(name: "StoreTestKit", package: "Store"),
            ],
            path: "PriceService/TestKit"
        ),
        .target(
            name: "StreamService",
            dependencies: [
                "Primitives",
                "Store",
                "PriceService",
                "PriceAlertService",
                "BalanceService",
                "TransactionsService",
                "NFTService",
                "PerpetualService",
                "DeviceService",
                "GemAPI",
                "Preferences",
                .product(name: "WebSocketClient", package: "SwiftHTTPClient")
            ],
            path: "StreamService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "StreamServiceTestKit",
            dependencies: [
                "StreamService",
                .product(name: "StoreTestKit", package: "Store"),
                "PriceServiceTestKit",
                "PriceAlertServiceTestKit",
                "BalanceServiceTestKit",
                "TransactionsServiceTestKit",
                "NFTServiceTestKit",
                "PerpetualServiceTestKit",
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "WebSocketClientTestKit", package: "SwiftHTTPClient"),
            ],
            path: "StreamService/TestKit"
        ),
        .target(
            name: "PriceAlertService",
            dependencies: [
                "Primitives",
                "Store",
                "NotificationService",
                "DeviceService",
                "GemAPI",
                "PriceService",
                "Preferences"
            ],
            path: "PriceAlertService",
            exclude: ["TestKit", "Tests"]
        ),
        .target(
            name: "PriceAlertServiceTestKit",
            dependencies: [
                "PriceAlertService",
                .product(name: "StoreTestKit", package: "Store"),
                "DeviceServiceTestKit",
                .product(name: "GemAPITestKit", package: "GemAPI"),
                "PriceServiceTestKit",
                .product(name: "PreferencesTestKit", package: "Preferences")
            ],
            path: "PriceAlertService/TestKit"
        ),
        .target(
            name: "TransactionStateService",
            dependencies: [
                "Primitives",
                "Store",
                "Blockchain",
                "Gemstone",
                .product(name: "ChainService", package: "ChainServices"),
                .product(name: "StakeService", package: "ChainServices"),
                "BalanceService",
                "EarnService",
                "NFTService",
                "GemstonePrimitives"
            ],
            path: "TransactionStateService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "TransactionStateServiceTestKit",
            dependencies: [
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "StakeServiceTestKit", package: "ChainServices"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                "NFTServiceTestKit",
                .product(name: "ChainServiceTestKit", package: "ChainServices"),
                "BalanceServiceTestKit",
                "SwapServiceTestKit",
                "EarnService",
                "Blockchain",
                "NativeProviderService",
                "TransactionStateService"
            ],
            path: "TransactionStateService/TestKit"
        ),
        .target(
            name: "TransactionsService",
            dependencies: [
                "Primitives",
                "GemAPI",
                "Store",
                "Preferences",
                "AssetsService",
            ],
            path: "TransactionsService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "TransactionsServiceTestKit",
            dependencies: [
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "GemAPITestKit", package: "GemAPI"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                "AssetsServiceTestKit",
                "TransactionsService",
            ],
            path: "TransactionsService/TestKit"
        ),
        .target(
            name: "DiscoverAssetsService",
            dependencies: [
                "Primitives",
                "BalanceService",
                "AssetsService",
                "Preferences",
                "GemAPI",
            ],
            path: "DiscoverAssetsService",
            exclude: ["Tests", "TestKit"]
        ),
        .target(
            name: "DiscoverAssetsServiceTestKit",
            dependencies: [
                "DiscoverAssetsService",
            ],
            path: "DiscoverAssetsService/TestKit"
        ),
        .target(
            name: "SwapService",
            dependencies: [
                "Gemstone",
                "GemstonePrimitives",
                "Primitives",
                .product(name: "ChainService", package: "ChainServices"),
                "Signer",
                "Keystore",
                "NativeProviderService",
            ],
            path: "SwapService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "SwapServiceTestKit",
            dependencies: [
                .product(name: "ChainServiceTestKit", package: "ChainServices"),
                "SwapService",
                "Gemstone"
            ],
            path: "SwapService/TestKit"
        ),
        .target(
            name: "AssetsService",
            dependencies: [
                "Primitives",
                "Store",
                "GemAPI",
                .product(name: "ChainService", package: "ChainServices"),
                "Preferences",
                "GemstonePrimitives"
            ],
            path: "AssetsService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "AssetsServiceTestKit",
            dependencies: [
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "GemAPITestKit", package: "GemAPI"),
                .product(name: "ChainServiceTestKit", package: "ChainServices"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                "AssetsService",
                "Primitives",
                "GemstonePrimitives"
            ],
            path: "AssetsService/TestKit"
        ),
        .target(
            name: "WalletSessionService",
            dependencies: [
                "Primitives",
                "Store",
                "Preferences"
            ],
            path: "WalletSessionService",
            exclude: ["TestKit", "Tests"]
        ),
        .target(
            name: "WalletSessionServiceTestKit",
            dependencies: [
                "WalletSessionService",
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "StoreTestKit", package: "Store")
            ],
            path: "WalletSessionService/TestKit"
        ),
        .target(
            name: "WalletService",
            dependencies: [
                "Primitives",
                "Keystore",
                "Store",
                "Preferences",
                "AvatarService",
                "WalletSessionService",
                "BalanceService"
            ],
            path: "WalletService",
            exclude: ["TestKit", "Tests"]
        ),
        .target(
            name: "WalletServiceTestKit",
            dependencies: [
                .product(name: "KeystoreTestKit", package: "Keystore"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "StoreTestKit", package: "Store"),
                "BalanceServiceTestKit",
                "WalletService"
            ],
            path: "WalletService/TestKit"
        ),
        .target(
            name: "AppService",
            dependencies: [
                "Primitives",
                "Store",
                "GemAPI",
                .product(name: "NodeService", package: "ChainServices"),
                .product(name: "ChainService", package: "ChainServices"),
                "GemstonePrimitives",
                "Preferences",
                "BannerService",
                "DeviceService",
                "AssetsService",
                "WalletService",
                "NotificationService",
                "PriceService",
                "StreamService",
                "PerpetualService",
                "ConnectionsService",
            ],
            path: "AppService",
            exclude: ["Tests", "TestKit"]
        ),
        .target(
            name: "AppServiceTestKit",
            dependencies: [
                "AppService",
                "Primitives",
                .product(name: "GemAPITestKit", package: "GemAPI"),
                "BannerServiceTestKit",
                .product(name: "NodeServiceTestKit", package: "ChainServices"),
                "DeviceServiceTestKit",
                "AssetsServiceTestKit",
                .product(name: "PreferencesTestKit", package: "Preferences"),
                "PriceServiceTestKit",
                "StreamServiceTestKit",
                "PerpetualServiceTestKit",
                "ConnectionsServiceTestKit",
            ],
            path: "AppService/TestKit"
        ),
        .target(
            name: "DeviceService",
            dependencies: [
                "Primitives",
                "Store",
                "Preferences",
                "GemAPI",
            ],
            path: "DeviceService",
            exclude: ["TestKit", "Tests"]
        ),
        .target(
            name: "DeviceServiceTestKit",
            dependencies: [
                "DeviceService",
                .product(name: "GemAPITestKit", package: "GemAPI"),
                .product(name: "StoreTestKit", package: "Store")
            ],
            path: "DeviceService/TestKit"
        ),
        .target(
            name: "NotificationService",
            dependencies: [
                "Primitives",
                "Store",
                "GemAPI",
                "DeviceService",
                "Preferences",
                "WalletService"
            ],
            path: "NotificationService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "NotificationServiceTestKit",
            dependencies: [
                "NotificationService",
                "DeviceServiceTestKit",
                "WalletServiceTestKit",
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "GemAPITestKit", package: "GemAPI"),
                .product(name: "PreferencesTestKit", package: "Preferences")
            ],
            path: "NotificationService/TestKit"
        ),
        .target(
            name: "AddressNameService",
            dependencies: [
                "GemAPI",
                "Primitives",
                "Store"
            ],
            path: "AddressNameService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "AddressNameServiceTestKit",
            dependencies: [
                "AddressNameService",
                .product(name: "GemAPITestKit", package: "GemAPI"),
                .product(name: "StoreTestKit", package: "Store")
            ],
            path: "AddressNameService/TestKit"
        ),
        .target(
            name: "ActivityService",
            dependencies: [
                "Primitives",
                "Store"
            ],
            path: "ActivityService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "ActivityServiceTestKit",
            dependencies: [
                "ActivityService",
                .product(name: "StoreTestKit", package: "Store")
            ],
            path: "ActivityService/TestKit"
        ),
        .target(
            name: "ContactService",
            dependencies: [
                "Primitives",
                "Store"
            ],
            path: "ContactService"
        ),
        .target(
            name: "RewardsService",
            dependencies: [
                "Primitives",
                "GemAPI",
                "AuthService",
                "Preferences",
            ],
            path: "RewardsService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "RewardsServiceTestKit",
            dependencies: [
                "RewardsService",
                .product(name: "GemAPITestKit", package: "GemAPI"),
            ],
            path: "RewardsService/TestKit"
        ),
        .target(
            name: "AuthService",
            dependencies: [
                "Primitives",
                "GemAPI",
                "Keystore",
                "Gemstone",
                "GemstonePrimitives",
                "Preferences",
            ],
            path: "AuthService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "AuthServiceTestKit",
            dependencies: [
                "AuthService",
            ],
            path: "AuthService/TestKit"
        ),
        .target(
            name: "ConnectionsService",
            dependencies: [
                "Primitives",
                "Store",
                "Preferences",
                .product(name: "WalletConnectorService", package: "ChainServices"),
            ],
            path: "ConnectionsService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "ConnectionsServiceTestKit",
            dependencies: [
                "ConnectionsService",
                "Primitives",
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "WalletConnectorServiceTestKit", package: "ChainServices"),
            ],
            path: "ConnectionsService/TestKit"
        ),
        .target(
            name: "EarnService",
            dependencies: [
                "Primitives",
                "Store",
                "Blockchain",
                "BalanceService",
            ],
            path: "EarnService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "EarnServiceTestKit",
            dependencies: [
                "EarnService",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "EarnService/TestKit"
        ),
        .testTarget(
            name: "PriceAlertServiceTests",
            dependencies: [
                "PriceAlertService",
                "PriceAlertServiceTestKit",
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "GemAPITestKit", package: "GemAPI"),
                "DeviceServiceTestKit",
                "PriceServiceTestKit",
                .product(name: "PrimitivesTestKit", package: "Primitives")
            ],
            path: "PriceAlertService/Tests"
        ),
        .testTarget(
            name: "BannerServiceTests",
            dependencies: [
                "BannerService",
                "BannerServiceTestKit",
                .product(name: "StoreTestKit", package: "Store"),
                "NotificationServiceTestKit",
                "Primitives"
            ],
            path: "BannerService/Tests"
        ),
        .testTarget(
            name: "WalletSessionServiceTests",
            dependencies: [
                "WalletSessionService",
                "WalletSessionServiceTestKit",
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "PrimitivesTestKit", package: "Primitives")
            ],
            path: "WalletSessionService/Tests"
        ),
        .testTarget(
            name: "WalletServiceTests",
            dependencies: [
                "WalletService",
                "WalletServiceTestKit",
                "BalanceServiceTestKit",
                .product(name: "KeystoreTestKit", package: "Keystore"),
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "PrimitivesTestKit", package: "Primitives")
            ],
            path: "WalletService/Tests"
        ),
        .testTarget(
            name: "AppServiceTests",
            dependencies: [
                "AppService",
                "AppServiceTestKit",
                .product(name: "GemAPITestKit", package: "GemAPI"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                "PerpetualServiceTestKit",
            ],
            path: "AppService/Tests"
        ),
        .testTarget(
            name: "DeviceServiceTests",
            dependencies: [
                "DeviceService",
                "DeviceServiceTestKit",
                "Primitives",
                .product(name: "GemAPITestKit", package: "GemAPI"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "StoreTestKit", package: "Store"),
            ],
            path: "DeviceService/Tests"
        ),
    ]
)
