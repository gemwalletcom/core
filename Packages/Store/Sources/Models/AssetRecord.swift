// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

internal import BigInt

struct AssetRecord: Identifiable, Codable, PersistableRecord, FetchableRecord, TableRecord  {
    
    static let databaseTableName: String = "assets"
    
    enum Columns {
        static let id = Column("id")
        static let rank = Column("rank")
        static let type = Column("type")
        static let chain = Column("chain")
        static let name = Column("name")
        static let symbol = Column("symbol")
        static let decimals = Column("decimals")
        static let tokenId = Column("tokenId")
        static let isEnabled = Column("isEnabled")
        static let isBuyable = Column("isBuyable")
        static let isSellable = Column("isSellable")
        static let isSwappable = Column("isSwappable")
        static let isStakeable = Column("isStakeable")
        static let isEarnable = Column("isEarnable")
        static let stakingApr = Column("stakingApr")
        static let earnApr = Column("earnApr")
        static let hasImage = Column("hasImage")
    }
    
    var id: String
    var chain: Chain
    var tokenId: String
    var name: String
    var symbol: String
    var decimals: Int
    var type: AssetType
    
    var isEnabled: Bool
    var isBuyable: Bool
    var isSellable: Bool
    var isSwappable: Bool
    var isStakeable: Bool
    var isEarnable: Bool
    var rank: Int
    var stakingApr: Double?
    var earnApr: Double?
    var hasImage: Bool
    
    static let price = hasOne(PriceRecord.self)
    static let links = hasMany(AssetLinkRecord.self, key: "links")
    static let balance = hasOne(BalanceRecord.self)
    static let account = hasOne(AccountRecord.self, key: "account", using: ForeignKey(["chain"], to: ["chain"]))
    static let priceAlert = hasOne(PriceAlertRecord.self).forKey("priceAlert")
    static let priceAlerts = hasMany(PriceAlertRecord.self).forKey("priceAlerts")
    static let recentActivities = hasMany(RecentActivityRecord.self, using: ForeignKey(["assetId"], to: ["id"]))
    static let search = hasOne(SearchRecord.self, using: ForeignKey(["assetId"], to: ["id"]))
}

extension AssetRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: Self.databaseTableName, ifNotExists: true) {
            $0.column(Columns.id.name, .text)
                .primaryKey()
                .notNull()
            $0.column(Columns.chain.name, .text)
                .notNull()
                .indexed()
            $0.column(Columns.tokenId.name, .text)
                .indexed()
            $0.column(Columns.name.name, .text)
                .notNull()
                .indexed()
            $0.column(Columns.symbol.name, .text)
                .notNull()
                .indexed()
            $0.column(Columns.decimals.name, .numeric)
                .notNull()
            $0.column(Columns.type.name, .text)
                .notNull()
            $0.column(Columns.isEnabled.name, .boolean)
                .defaults(to: true)
            $0.column(Columns.isBuyable.name, .boolean)
                .defaults(to: false)
            $0.column(Columns.isSellable.name, .boolean)
                .defaults(to: false)
            $0.column(Columns.isSwappable.name, .boolean)
                .defaults(to: false)
            $0.column(Columns.isStakeable.name, .boolean)
                .defaults(to: false)
            $0.column(Columns.isEarnable.name, .boolean)
                .defaults(to: false)
            $0.column(Columns.rank.name, .numeric)
                .defaults(to: 0)
            $0.column(Columns.stakingApr.name, .double)
            $0.column(Columns.earnApr.name, .double)
            $0.column(Columns.hasImage.name, .boolean)
                .defaults(to: false)
        }
    }
}

extension Asset {
    var record: AssetRecord {
        AssetRecord(
            id: id.identifier,
            chain: chain,
            tokenId: tokenId ?? "",
            name: name,
            symbol: symbol,
            decimals: Int(decimals),
            type: type,
            isEnabled: false,
            isBuyable: false,
            isSellable: false,
            isSwappable: false,
            isStakeable: false,
            isEarnable: false,
            rank: 0,
            hasImage: false
        )
    }
}

extension AssetRecord {
    func mapToAsset() -> Asset {
        let tokenId = tokenId.isEmpty ? nil : tokenId
        return Asset(
            id: AssetId(chain: chain, tokenId: tokenId),
            name: name,
            symbol: symbol,
            decimals: decimals.asInt32,
            type: type
        )
    }
    
    func mapToBasic() -> AssetBasic {
        AssetBasic(
            asset: mapToAsset(),
            properties: AssetProperties(
                isEnabled: isEnabled,
                isBuyable: isBuyable,
                isSellable: isSellable,
                isSwapable: isSwappable,
                isStakeable: isStakeable,
                stakingApr: stakingApr,
                isEarnable: isEarnable,
                earnApr: earnApr,
                hasImage: hasImage
            ),
            score: AssetScore(rank: rank.asInt32),
            price: nil
        )
    }
}

extension PriceRecordInfo {
    var priceData: PriceData {
        PriceData(
            asset: asset.mapToAsset(),
            price: price?.mapToPrice(),
            priceAlerts: priceAlerts.or([]).map { $0.map() },
            market: price?.mapToMarket(),
            links: links.map { $0.link }
        )
    }
}

extension AssetRecordInfo {
    var assetData: AssetData {
        AssetData(
            asset: asset.mapToAsset(),
            balance: balance?.mapToBalance() ?? .zero,
            account: account.mapToAccount(),
            price: price?.mapToPrice(),
            priceAlerts: priceAlerts.or([]).compactMap { $0.map() },
            metadata: metadata
        )
    }

    var metadata: AssetMetaData {
        AssetMetaData(
            isEnabled: asset.isEnabled,
            isBalanceEnabled: balance?.isEnabled ?? false,
            isBuyEnabled: asset.isBuyable,
            isSellEnabled: asset.isSellable,
            isSwapEnabled: asset.isSwappable,
            isStakeEnabled: asset.isStakeable,
            isEarnEnabled: asset.isEarnable,
            isPinned: balance?.isPinned ?? false,
            isActive: balance?.isActive ?? true,
            stakingApr: asset.stakingApr,
            earnApr: asset.earnApr,
            rankScore: asset.rank.asInt32
        )
    }
}

extension AssetBasic {
    var record: AssetRecord {
        AssetRecord(
            id: asset.id.identifier,
            chain: asset.chain,
            tokenId: asset.tokenId ?? "",
            name: asset.name,
            symbol: asset.symbol,
            decimals: Int(asset.decimals),
            type: asset.type,
            isEnabled: properties.isEnabled,
            isBuyable: properties.isBuyable,
            isSellable: properties.isSellable,
            isSwappable: properties.isSwapable,
            isStakeable: properties.isStakeable,
            isEarnable: properties.isEarnable,
            rank: score.rank.asInt,
            stakingApr: properties.stakingApr,
            earnApr: properties.earnApr,
            hasImage: properties.hasImage
        )
    }
}
