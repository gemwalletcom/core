// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol StakeServiceable: Sendable {
    func stakeApr(assetId: AssetId) throws -> Double?
    func update(walletId: WalletId, chain: Chain, address: String) async throws
}
