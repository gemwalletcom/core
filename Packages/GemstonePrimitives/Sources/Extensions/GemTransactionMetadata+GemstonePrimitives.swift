// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

extension Gemstone.TransactionMetadata {
    public func mapToAnyCodableValue() -> AnyCodableValue? {
        switch self {
        case .perpetual(let perpetualMetadata):
            .encode(TransactionPerpetualMetadata(
                pnl: perpetualMetadata.pnl,
                price: perpetualMetadata.price,
                direction: perpetualMetadata.direction.map(),
                isLiquidation: perpetualMetadata.isLiquidation,
                provider: perpetualMetadata.provider?.map()
            ))
        }
    }
}
