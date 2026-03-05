// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemPerpetualBalance
import struct Gemstone.GemPerpetualPosition
import Primitives

public protocol HyperliquidPerpetualServiceable: PerpetualServiceable {
    func getHypercorePositions(walletId: WalletId) throws -> [GemPerpetualPosition]
    func updateBalance(walletId: WalletId, balance: GemPerpetualBalance) throws
    func diffPositions(deleteIds: [String], positions: [GemPerpetualPosition], walletId: WalletId) throws
    func updatePrices(_ prices: [String: Double]) throws
}
