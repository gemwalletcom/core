// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import BigInt

extension Balance {
    public static let zero: Balance = Balance(available: BigInt.zero)

    public var total: BigInt {
        available + frozen + locked + staked + pending + rewards + earn
    }
}
