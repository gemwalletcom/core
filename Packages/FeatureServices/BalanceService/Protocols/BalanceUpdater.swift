// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol BalanceUpdater: Sendable {
    func updateBalance(for wallet: Wallet, assetIds: [AssetId]) async
}
