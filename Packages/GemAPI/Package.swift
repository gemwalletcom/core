// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "GemAPI",
    platforms: [.iOS(.v17), .macOS(.v15)],
    products: [
        .library(
            name: "GemAPI",
            targets: ["GemAPI"]
        ),
        .library(
            name: "GemAPITestKit",
            targets: ["GemAPITestKit"]
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "SwiftHTTPClient", path: "../SwiftHTTPClient"),
    ],
    targets: [
        .target(
            name: "GemAPI",
            dependencies: [
                "Primitives",
                "SwiftHTTPClient",
            ],
            path: "Sources"
        ),
        .target(
            name: "GemAPITestKit",
            dependencies: [
                "GemAPI",
                "SwiftHTTPClient",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "TestKit"
        ),
        .testTarget(
            name: "GemAPITests",
            dependencies: [
                "GemAPI",
                "Primitives",
                "SwiftHTTPClient",
            ]
        ),
    ]
)
