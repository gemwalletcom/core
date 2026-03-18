// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension TransactionPerpetualMetadata {
    static func mock(
        pnl: Double = 0,
        price: Double = 0,
        direction: PerpetualDirection = .long,
        isLiquidation: Bool? = nil,
        provider: PerpetualProvider? = .hypercore
    ) -> TransactionPerpetualMetadata {
        TransactionPerpetualMetadata(
            pnl: pnl,
            price: price,
            direction: direction,
            isLiquidation: isLiquidation,
            provider: provider
        )
    }
}
