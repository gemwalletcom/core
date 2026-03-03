// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Preferences
import Primitives

public extension WalletPreferences {
    static func mock(walletId: WalletId = .multicoin(address: "0x\(UUID().uuidString)")) -> WalletPreferences {
        WalletPreferences(walletId: walletId)
    }
}
