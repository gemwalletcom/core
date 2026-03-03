// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension WalletId {
    static func mock(
        address: String = "0x0000000000000000000000000000000000000000"
    ) -> WalletId {
        .multicoin(address: address)
    }
}
