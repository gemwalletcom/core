// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import BalanceService
import Primitives

public struct BalanceUpdaterMock: BalanceUpdater {
    public init() {}
    public func updateBalance(for wallet: Wallet, assetIds: [AssetId]) async {}
}

public extension BalanceUpdater where Self == BalanceUpdaterMock {
    static func mock() -> BalanceUpdaterMock {
        BalanceUpdaterMock()
    }
}
