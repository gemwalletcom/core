// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Style
import SwiftUI

public extension SwapProvider {
    var image: Image {
        switch self {
        case .uniswapV3, .uniswapV4: Images.SwapProviders.uniswap
        case .jupiter: Images.SwapProviders.jupiter
        case .pancakeswapV3: Images.SwapProviders.pancakeswap
        case .thorchain: Images.SwapProviders.thorchain
        case .across: Images.SwapProviders.across
        case .oku: Images.SwapProviders.oku
        case .wagmi: Images.SwapProviders.wagmi
        case .cetusAggregator: Images.SwapProviders.cetus
        case .stonfiV2: Images.SwapProviders.stonfi
        case .mayan: Images.SwapProviders.mayan
        case .chainflip: Images.SwapProviders.chainflip
        case .relay: Images.SwapProviders.relay
        case .aerodrome: Images.SwapProviders.aerodrome
        case .hyperliquid: Images.SwapProviders.hyperliquid
        case .nearIntents: Images.SwapProviders.nearIntents
        case .orca: Images.SwapProviders.orca
        case .panora: Images.SwapProviders.panora
        case .okx: Images.SwapProviders.okx
        case .squid: Images.SwapProviders.squid
        }
    }
}
