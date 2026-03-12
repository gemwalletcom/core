// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.SwapperRoute
import typealias Gemstone.AssetId

extension SwapperRoute {
    static func mock() -> SwapperRoute {
        SwapperRoute(
            input: "ethereum_0x0000000000000000000000000000000000000000",
            output: "ethereum_0xdac17f958d2ee523a2206206994597c13d831ec7",
            routeData: "0x"
        )
    }
} 
