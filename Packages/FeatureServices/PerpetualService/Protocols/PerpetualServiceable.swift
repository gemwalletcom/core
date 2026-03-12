// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol PerpetualServiceable: Sendable {
    func updateMarkets() async throws
    func updateMarket(symbol: String) async throws
    func candlesticks(symbol: String, period: ChartPeriod) async throws -> [ChartCandleStick]
    func portfolio(address: String) async throws -> PerpetualPortfolio
    func setPinned(_ isPinned: Bool, perpetualId: String) throws
    func fetchPositions(walletId: WalletId, address: String) async throws
}
