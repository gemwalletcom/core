// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives
import BigInt

extension GemBalance {
    public func map() throws -> Balance {
        Balance(
            available: try BigInt.from(string: available),
            frozen: try BigInt.from(string: frozen),
            locked: try BigInt.from(string: locked),
            staked: try BigInt.from(string: staked),
            pending: try BigInt.from(string: pending),
            pendingUnconfirmed: try BigInt.from(string: pendingUnconfirmed),
            rewards: try BigInt.from(string: rewards),
            reserved: try BigInt.from(string: reserved),
            withdrawable: try BigInt.from(string: withdrawable),
            earn: try BigInt.from(string: earn),
            metadata: metadata?.map()
        )
    }
}
