// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import PerpetualService
import Primitives

public struct PerpetualServiceMock: PerpetualServiceable {
    public init() {}

    public func updateMarkets() async throws {}

    public func updateMarket(symbol: String) async throws {}

    public func candlesticks(symbol: String, period: ChartPeriod) async throws -> [ChartCandleStick] {
        []
    }

    public func portfolio(address: String) async throws -> PerpetualPortfolio {
        PerpetualPortfolio(day: nil, week: nil, month: nil, allTime: nil, accountSummary: nil)
    }

    public func setPinned(_ isPinned: Bool, perpetualId: String) throws {}
}

// MARK: - HyperliquidPerpetualServiceable

extension PerpetualServiceMock: HyperliquidPerpetualServiceable {
    public func getHypercorePositions(walletId: WalletId) throws -> [GemPerpetualPosition] {
        []
    }

    public func updateBalance(walletId: WalletId, balance: GemPerpetualBalance) throws {}

    public func diffPositions(deleteIds: [String], positions: [GemPerpetualPosition], walletId: WalletId) throws {}

    public func updatePrices(_ prices: [String: Double]) throws {}

    public func fetchPositions(walletId: WalletId, address: String) async throws {}
}
