// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemPerpetualBalance
import struct Gemstone.GemPerpetualPosition
import Primitives
import Store
import Formatters
import Preferences

public struct PerpetualService: PerpetualServiceable {

    private let store: PerpetualStore
    private let assetStore: AssetStore
    private let priceStore: PriceStore
    private let balanceStore: BalanceStore
    private let provider: PerpetualProvidable
    private let preferences: Preferences

    public init(
        store: PerpetualStore,
        assetStore: AssetStore,
        priceStore: PriceStore,
        balanceStore: BalanceStore,
        provider: PerpetualProvidable,
        preferences: Preferences
    ) {
        self.store = store
        self.assetStore = assetStore
        self.priceStore = priceStore
        self.balanceStore = balanceStore
        self.provider = provider
        self.preferences = preferences
    }
    
    public func updateMarkets() async throws {
        let perpetualsData = try await provider.getPerpetualsData()
        let perpetuals = perpetualsData.map { $0.perpetual }
        let assets = perpetualsData.map { createPerpetualAssetBasic(from: $0.asset) }

        try assetStore.add(assets: assets)
        try store.upsertPerpetuals(perpetuals)
        // setup prices
        try priceStore.updatePrice(price: AssetPrice(
            assetId: Asset.hypercoreUSDC().id,
            price: 1,
            priceChangePercentage24h: 0,
            updatedAt: .now
        ), currency: Currency.usd.rawValue)
    }

    public func updateMarket(symbol: String) async throws {
        try await updateMarkets()
    }

    public func candlesticks(symbol: String, period: ChartPeriod) async throws -> [ChartCandleStick] {
        return try await provider.getCandlesticks(symbol: symbol, period: period)
    }

    public func portfolio(address: String) async throws -> PerpetualPortfolio {
        try await provider.getPortfolio(address: address)
    }

    public func setPinned(_ isPinned: Bool, perpetualId: String) throws {
        try store.setPinned(for: [perpetualId], value: isPinned)
    }

    public func fetchPositions(walletId: WalletId, address: String) async throws {
        let summary = try await provider.getPositions(address: address)
        let existingPositionIds = Set(try store.getPositions(walletId: walletId, provider: .hypercore).map(\.id))
        let newPositionIds = Set(summary.positions.map(\.id))
        let deleteIds = Array(existingPositionIds.subtracting(newPositionIds))

        try store.diffPositions(deleteIds: deleteIds, positions: summary.positions, walletId: walletId)
        try syncProviderBalances(walletId: walletId, balance: summary.balance)
    }

    public func clear() throws {
        try store.clear()
    }

    // MARK: - Private

    private func syncProviderBalances(walletId: WalletId, balance: PerpetualBalance) throws {
        let usd = Asset.hypercoreUSDC()
        try balanceStore.addMissingBalances(walletId: walletId, assetIds: [usd.id], isEnabled: false)

        let perpetuals = try store.getPerpetuals().map(\.assetId)
        try balanceStore.addMissingBalances(walletId: walletId, assetIds: perpetuals, isEnabled: false)

        try balanceStore.updateBalances(
            [
             UpdateBalance(
                assetId: usd.id,
                type: .perpetual(UpdatePerpetualBalance(
                    available: try perpetualBalanceValue(balance.available),
                    reserved: try perpetualBalanceValue(balance.reserved),
                    withdrawable: try perpetualBalanceValue(balance.withdrawable)
                )),
                updatedAt: .now,
                isActive: true
             ),
            ],
            for: walletId
        )
    }

    private func perpetualBalanceValue(_ amount: Double) throws -> UpdateBalanceValue {
        UpdateBalanceValue(
            value: try ValueFormatter.full.inputNumber(from: amount.description, decimals: 6).description,
            amount: amount
        )
    }
    
    private func createPerpetualAssetBasic(from asset: Asset) -> AssetBasic {
        AssetBasic(
            asset: asset,
            properties: AssetProperties(
                isEnabled: false,
                isBuyable: false,
                isSellable: false,
                isSwapable: false,
                isStakeable: false,
                stakingApr: nil,
                isEarnable: false,
                earnApr: nil,
                hasImage: false
            ),
            score: AssetScore(rank: 0),
            price: nil
        )
    }
}

// MARK: - HyperliquidPerpetualServiceable

extension PerpetualService: HyperliquidPerpetualServiceable {
    public func getHypercorePositions(walletId: WalletId) throws -> [GemPerpetualPosition] {
        try store.getPositions(walletId: walletId, provider: .hypercore).map { $0.map() }
    }

    public func updateBalance(walletId: WalletId, balance: GemPerpetualBalance) throws {
        try syncProviderBalances(walletId: walletId, balance: balance.map())
    }

    public func diffPositions(deleteIds: [String], positions: [GemPerpetualPosition], walletId: WalletId) throws {
        try store.diffPositions(deleteIds: deleteIds, positions: positions.map { try $0.map() }, walletId: walletId)
    }

    public func updatePrices(_ prices: [String: Double]) throws {
        guard preferences.perpetualPricesUpdatedAt.isOutdated(by: 5) else { return }
        try store.updatePrices(prices)
        preferences.perpetualPricesUpdatedAt = .now
    }
}
