// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct PriceStore: Sendable {
    
    let db: DatabaseQueue
    
    public init(db: DB) {
        self.db = db.dbQueue
    }
    
    public func updatePrice(price: AssetPrice, currency: String) throws {
        try updatePrices(prices: [price], currency: currency)
    }
    
    public func getRate(currency: String) throws -> FiatRateRecord {
        try db.read { db in
            guard let rate = try FiatRateRecord.filter(key: currency).fetchOne(db) else {
                throw AnyError("unknown currency: \(currency)")
            }
            return rate
        }
    }
    
    public func updatePrices(prices: [AssetPrice], currency: String) throws {
        let rate = try getRate(currency: currency)
        try db.write { db in
            for assetPrice in prices {
                let _ = try assetPrice.record.upsertAndFetch(
                    db,
                    onConflict: [],
                    doUpdate: { _ in [
                        PriceRecord.Columns.price.set(to: assetPrice.price * rate.rate),
                        PriceRecord.Columns.priceUsd.set(to: assetPrice.price),
                        PriceRecord.Columns.priceChangePercentage24h.set(to: assetPrice.priceChangePercentage24h),
                        PriceRecord.Columns.updatedAt.set(to: Date()),
                        PriceRecord.Columns.marketCap.noOverwrite,
                        PriceRecord.Columns.marketCapFdv.noOverwrite,
                        PriceRecord.Columns.marketCapRank.noOverwrite,
                        PriceRecord.Columns.totalVolume.noOverwrite,
                        PriceRecord.Columns.circulatingSupply.noOverwrite,
                        PriceRecord.Columns.totalSupply.noOverwrite,
                        PriceRecord.Columns.maxSupply.noOverwrite,
                        PriceRecord.Columns.allTimeHigh.noOverwrite,
                        PriceRecord.Columns.allTimeHighDate.noOverwrite,
                        PriceRecord.Columns.allTimeHighChangePercentage.noOverwrite,
                        PriceRecord.Columns.allTimeLow.noOverwrite,
                        PriceRecord.Columns.allTimeLowDate.noOverwrite,
                        PriceRecord.Columns.allTimeLowChangePercentage.noOverwrite,
                    ] })
            }
        }
    }
    
    @discardableResult
    public func updateMarket(assetId: String, market: AssetMarket, rate: Double) throws -> Int {
        return try db.write { db in
            try PriceRecord
                .filter(PriceRecord.Columns.assetId == assetId)
                .updateAll(
                    db,
                    PriceRecord.Columns.marketCap.set(to: market.marketCap ?? 0 * rate),
                    PriceRecord.Columns.marketCapFdv.set(to: market.marketCapFdv ?? 0 * rate),
                    PriceRecord.Columns.totalVolume.set(to: market.totalVolume ?? 0 * rate),
                    PriceRecord.Columns.marketCapRank.set(to: market.marketCapRank),
                    PriceRecord.Columns.circulatingSupply.set(to: market.circulatingSupply),
                    PriceRecord.Columns.totalSupply.set(to: market.totalSupply),
                    PriceRecord.Columns.maxSupply.set(to: market.maxSupply),
                    PriceRecord.Columns.allTimeHigh.set(to: market.allTimeHighValue.map { Double($0.value) }),
                    PriceRecord.Columns.allTimeHighDate.set(to: market.allTimeHighValue?.date),
                    PriceRecord.Columns.allTimeHighChangePercentage.set(to: market.allTimeHighValue.map { Double($0.percentage) }),
                    PriceRecord.Columns.allTimeLow.set(to: market.allTimeLowValue.map { Double($0.value) }),
                    PriceRecord.Columns.allTimeLowDate.set(to: market.allTimeLowValue?.date),
                    PriceRecord.Columns.allTimeLowChangePercentage.set(to: market.allTimeLowValue.map { Double($0.percentage) })
                )
        }
    }
    
    public func getPrices(for assetIds: [String]) throws -> [AssetPrice] {
        try db.read { db in
            try PriceRecord
                .filter(assetIds.contains(PriceRecord.Columns.assetId))
                .fetchAll(db)
                .map { $0.mapToAssetPrice() }
        }
    }
    
    public func enabledPriceAssets(walletId: WalletId) throws -> [AssetId] {
        try db.read { db in
            let priceAlertsAssets = try PriceAlertRecord.fetchAll(db).map { $0.assetId }
            let enabledAssets = try BalanceRecord
                .filter(BalanceRecord.Columns.walletId == walletId.id)
                .filter(BalanceRecord.Columns.isEnabled == true)
                .fetchAll(db)
                .compactMap { $0.assetId }

            return priceAlertsAssets.asSet().union(enabledAssets).asArray()
        }
    }
    
    @discardableResult
    public func updateCurrency(currency: String) throws -> Int {
        let rate = try getRate(currency: currency)
        return try db.write { db in
            try PriceRecord.updateAll(db, [
                PriceRecord.Columns.price
                    .set(to: PriceRecord.Columns.priceUsd * rate.rate)
            ])
        }
    }
    
    public func clear() throws -> Int {
        try db.write {
            try PriceRecord
                .deleteAll($0)
        }
    }
}
