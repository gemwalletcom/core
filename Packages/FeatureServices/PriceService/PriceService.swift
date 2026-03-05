// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Store

public struct PriceService: Sendable {
    private let priceStore: PriceStore
    private let fiatRateStore: FiatRateStore

    public init(
        priceStore: PriceStore,
        fiatRateStore: FiatRateStore
    ) {
        self.priceStore = priceStore
        self.fiatRateStore = fiatRateStore
    }

    public func updatePrices(_ prices: [AssetPrice], currency: String) throws {
        try priceStore.updatePrices(prices: prices, currency: currency)
    }

    public func updateMarketPrice(assetId: AssetId, market: AssetMarket, currency: String) throws {
        try priceStore.updateMarket(
            assetId: assetId.identifier,
            market: market,
            rate: try getRate(currency: currency)
        )
    }

    public func getPrice(for assetId: AssetId) throws -> AssetPrice? {
        try priceStore.getPrices(for: [assetId.identifier]).first
    }
    
    public func getPrices(for assetIds: [AssetId]) throws -> [AssetPrice] {
        try priceStore.getPrices(for: assetIds.map { $0.identifier })
    }
    
    public func observableAssets(walletId: WalletId) throws -> [AssetId] {
        let priceAssets = try priceStore.enabledPriceAssets(walletId: walletId)
        if priceAssets.isEmpty {
            return [Chain.bitcoin, Chain.ethereum, Chain.smartChain, Chain.solana].map { $0.assetId }
        }
        return priceAssets
    }
    
    public func changeCurrency(currency: String) throws {
        try priceStore.updateCurrency(currency: currency)
    }
    
    public func getRate(currency: String) throws -> Double {
        try priceStore.getRate(currency: currency).rate
    }
    
    public func addRates(_ rates: [FiatRate]) throws {
        guard rates.isNotEmpty else { return }
        
        try fiatRateStore.add(rates)
    }
    
    @discardableResult
    public func clear() throws -> Int {
        try priceStore.clear()
    }
}
