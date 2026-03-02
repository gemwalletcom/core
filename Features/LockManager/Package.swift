// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "LockManager",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "LockManager",
            targets: ["LockManager"]),
    ],
    dependencies: [
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Keystore", path: "../../Packages/Keystore"),
    ],
    targets: [
        .target(
            name: "LockManager",
            dependencies: [
                "Style",
                "Components",
                "Localization",
                "Keystore"
            ],
            path: "Sources"
        ),
        .testTarget(
            name: "LockManagerTests",
            dependencies: ["LockManager"]
        ),
    ]
)
