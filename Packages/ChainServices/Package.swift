// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "ChainServices",
    platforms: [
        .iOS(.v17),
        .macOS(.v15)
    ],
    products: [
        .library(name: "NameService", targets: ["NameService"]),
        .library(name: "NameServiceTestKit", targets: ["NameServiceTestKit"]),
        .library(name: "StakeService", targets: ["StakeService"]),
        .library(name: "StakeServiceTestKit", targets: ["StakeServiceTestKit"]),
        .library(name: "NodeService", targets: ["NodeService"]),
        .library(name: "NodeServiceTestKit", targets: ["NodeServiceTestKit"]),
        .library(name: "WalletConnectorService", targets: ["WalletConnectorService"]),
        .library(name: "WalletConnectorServiceTestKit", targets: ["WalletConnectorServiceTestKit"]),
        .library(name: "ScanService", targets: ["ScanService"]),
        .library(name: "ScanServiceTestKit", targets: ["ScanServiceTestKit"]),
        .library(name: "ExplorerService", targets: ["ExplorerService"]),
        .library(name: "ChainService", targets: ["ChainService"]),
        .library(name: "ChainServiceTestKit", targets: ["ChainServiceTestKit"]),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "GemAPI", path: "../GemAPI"),
        .package(name: "Store", path: "../Store"),
        .package(name: "Blockchain", path: "../Blockchain"),
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
        .package(name: "Preferences", path: "../Preferences"),
        .package(name: "NativeProviderService", path: "../NativeProviderService"),
        .package(name: "reown-swift", path: "../../Submodules/reown-swift"),
    ],
    targets: [
        .target(
            name: "NameService",
            dependencies: [
                "Primitives",
                "GemAPI"
            ],
            path: "NameService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "NameServiceTestKit",
            dependencies: [
                "NameService",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "NameService/TestKit"
        ),
        .target(
            name: "StakeService",
            dependencies: [
                "Primitives",
                "Store",
                "GemAPI",
                "ChainService",
            ],
            path: "StakeService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "StakeServiceTestKit",
            dependencies: [
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "GemAPITestKit", package: "GemAPI"),
                "ChainServiceTestKit",
                "Primitives",
                "StakeService"
            ],
            path: "StakeService/TestKit"
        ),
        .target(
            name: "NodeService",
            dependencies: [
                "Primitives",
                "Store",
                "ChainService",
            ],
            path: "NodeService",
            exclude: ["TestKit", "Tests"]
        ),
        .target(
            name: "NodeServiceTestKit",
            dependencies: [
                "NodeService",
                .product(name: "StoreTestKit", package: "Store"),
            ],
            path: "NodeService/TestKit"
        ),
        .testTarget(
            name: "NodeServiceTests",
            dependencies: [
                "NodeService",
                "NodeServiceTestKit",
                "Primitives",
                .product(name: "StoreTestKit", package: "Store"),
            ],
            path: "NodeService/Tests"
        ),
        .target(
            name: "WalletConnectorService",
            dependencies: [
                "Primitives",
                "Gemstone",
                "GemstonePrimitives",
                "NativeProviderService",
                .product(name: "WalletConnect", package: "reown-swift"),
                .product(name: "ReownWalletKit", package: "reown-swift"),
                .product(name: "WalletConnectNetworking", package: "reown-swift"),
            ],
            path: "WalletConnectorService",
            exclude: ["TestKit", "Tests"]
        ),
        .target(
            name: "WalletConnectorServiceTestKit",
            dependencies: [
                "WalletConnectorService",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "WalletConnectorService/TestKit"
        ),
        .target(
            name: "ScanService",
            dependencies: [
                "Primitives",
                "Blockchain",
            ],
            path: "ScanService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "ScanServiceTestKit",
            dependencies: [
                "ScanService",
                "Primitives",
                "Blockchain",
                "NativeProviderService",
            ],
            path: "ScanService/TestKit"
        ),
        .target(
            name: "ExplorerService",
            dependencies: [
                "Primitives",
                "GemstonePrimitives",
                "Preferences"
            ],
            path: "ExplorerService",
            exclude: ["Tests", "TestKit"]
        ),
        .target(
            name: "ChainService",
            dependencies: [
                "Primitives",
                "Blockchain",
            ],
            path: "ChainService",
            exclude: ["TestKit"]
        ),
        .target(
            name: "ChainServiceTestKit",
            dependencies: [
                "ChainService",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                "Blockchain",
                .product(name: "BlockchainTestKit", package: "Blockchain"),
            ],
            path: "ChainService/TestKit"
        ),
        .testTarget(
            name: "WalletConnectorServiceTests",
            dependencies: [
                "WalletConnectorService",
            ],
            path: "WalletConnectorService/Tests"
        ),
    ]
)
