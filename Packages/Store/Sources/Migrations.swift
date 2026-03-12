// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct Migrations {
    
    var migrator = DatabaseMigrator()
    
    init(
        migrator: DatabaseMigrator = DatabaseMigrator()
    ) {
        self.migrator = migrator
    }
    
    private static func clearChainData(_ db: Database, chain: String) throws {
        try? db.execute(sql: "DELETE FROM \(TransactionAssetAssociationRecord.databaseTableName) WHERE assetId LIKE ? COLLATE NOCASE", arguments: ["\(chain)%"])
        try? db.execute(sql: "DELETE FROM \(TransactionRecord.databaseTableName) WHERE chain = ?", arguments: [chain])
        try? db.execute(sql: "DELETE FROM \(BalanceRecord.databaseTableName) WHERE assetId LIKE ? COLLATE NOCASE", arguments: ["\(chain)%"])
        try? db.execute(sql: "DELETE FROM \(PriceRecord.databaseTableName) WHERE assetId LIKE ? COLLATE NOCASE", arguments: ["\(chain)%"])
        try? db.execute(sql: "DELETE FROM \(AssetLinkRecord.databaseTableName) WHERE assetId LIKE ? COLLATE NOCASE", arguments: ["\(chain)%"])
        try? db.execute(sql: "DELETE FROM \(PerpetualRecord.databaseTableName) WHERE assetId LIKE ? COLLATE NOCASE", arguments: ["\(chain)%"])
        try? db.execute(sql: "DELETE FROM \(AssetRecord.databaseTableName) WHERE chain = ?", arguments: [chain])
    }
    
    mutating func run(dbQueue: DatabaseQueue) throws {
        migrator.registerMigration("Create all start table") { db in
            // wallet
            try WalletRecord.create(db: db)
            try AssetRecord.create(db: db)
            try AccountRecord.create(db: db)
            try BalanceRecord.create(db: db)

            // asset
            try FiatRateRecord.create(db: db)
            try PriceRecord.create(db: db)
            try AssetLinkRecord.create(db: db)
            //TODO: Market. try MarketAssetRecord.create(db: db)
            
            // transactions
            try TransactionRecord.create(db: db)
            try TransactionAssetAssociationRecord.create(db: db)
            try AddressRecord.create(db: db)
            
            // nodes
            try NodeRecord.create(db: db)
            try NodeSelectedRecord.create(db: db)
            
            // stake
            try StakeValidatorRecord.create(db: db)
            try StakeDelegationRecord.create(db: db)
            
            // connections
            try WalletConnectionRecord.create(db: db)

            // others
            try BannerRecord.create(db: db)
            try PriceAlertRecord.create(db: db)
            try ContactRecord.create(db: db)
            try ContactAddressRecord.create(db: db)
            
            // nft
            try NFTCollectionRecord.create(db: db)
            try NFTAssetRecord.create(db: db)
            try NFTAssetAssociationRecord.create(db: db)
            
            // perpetuals
            try PerpetualRecord.create(db: db)
            try PerpetualPositionRecord.create(db: db)

            try RecentActivityRecord.create(db: db)
            try SearchRecord.create(db: db)
            try NotificationRecord.create(db: db)
        }
        try migrator.migrate(dbQueue)
    }
    
    mutating func runChanges(dbQueue: DatabaseQueue) throws {
        migrator.registerMigration("Delete missing assetId in \(PriceRecord.databaseTableName), \(BalanceRecord.databaseTableName)") {
            try? $0.execute(sql: "DELETE FROM prices WHERE assetId NOT IN (SELECT id FROM assets)")
            try? $0.execute(sql: "DELETE FROM balances WHERE assetId NOT IN (SELECT id FROM assets)")
        }
        
        migrator.registerMigration("Add isPinned to \(WalletRecord.databaseTableName)") { db in
            try? db.alter(table: WalletRecord.databaseTableName) {
                $0.add(column: WalletRecord.Columns.isPinned.name, .boolean).defaults(to: false)
            }
        }

        migrator.registerMigration("Set order as index in \(WalletRecord.databaseTableName)") { db in
            try? db.execute(sql: "UPDATE wallets SET \"order\" = \"index\"")
        }
        
        migrator.registerMigration("Create \(PriceAlertRecord.databaseTableName)") { db in
            try? PriceAlertRecord.create(db: db)
        }
        
        migrator.registerMigration("Recreate \(BannerRecord.databaseTableName)") { db in
            try? db.drop(table: BannerRecord.databaseTableName)
            try? BannerRecord.create(db: db)
        }
        
        migrator.registerMigration("Add balances value to \(BalanceRecord.databaseTableName)") { db in
            try? db.alter(table: BalanceRecord.databaseTableName) {
                $0.add(column: BalanceRecord.Columns.availableAmount.name, .double).defaults(to: 0)
                $0.add(column: BalanceRecord.Columns.frozenAmount.name, .double).defaults(to: 0)
                $0.add(column: BalanceRecord.Columns.lockedAmount.name, .double).defaults(to: 0)
                $0.add(column: BalanceRecord.Columns.stakedAmount.name, .double).defaults(to: 0)
                $0.add(column: BalanceRecord.Columns.pendingAmount.name, .double).defaults(to: 0)
                $0.add(column: BalanceRecord.Columns.rewardsAmount.name, .double).defaults(to: 0)
                $0.add(column: BalanceRecord.Columns.reservedAmount.name, .double).defaults(to: 0)
                $0.addColumn(sql: BalanceRecord.totalAmountSQlCreation)
            }
        }
        
        migrator.registerMigration("Add rewards to \(BalanceRecord.databaseTableName)") { db in
            try? db.alter(table: BalanceRecord.databaseTableName) {
                $0.add(column: BalanceRecord.Columns.rewards.name, .text)
                    .defaults(to: "0")
            }
        }
        
        migrator.registerMigration("Add reserved to \(BalanceRecord.databaseTableName)") { db in
            try? db.alter(table: BalanceRecord.databaseTableName) {
                $0.add(column: BalanceRecord.Columns.reserved.name, .text)
                    .defaults(to: "0")
            }
        }
        
        migrator.registerMigration("Add updatedAt to \(BalanceRecord.databaseTableName)") { db in
            try? db.alter(table: BalanceRecord.databaseTableName) {
                $0.add(column: BalanceRecord.Columns.updatedAt.name, .date)
            }
        }
        
        migrator.registerMigration("Add isSellable to \(AssetRecord.databaseTableName)") { db in
            try? db.alter(table: AssetRecord.databaseTableName) {
                $0.add(column: AssetRecord.Columns.isSellable.name, .boolean)
                    .defaults(to: false)
            }
        }
        
        migrator.registerMigration("Add isStakeable to \(AssetRecord.databaseTableName)") { db in
            try? db.alter(table: AssetRecord.databaseTableName) {
                $0.add(column: AssetRecord.Columns.isStakeable.name, .boolean)
                    .defaults(to: false)
            }
        }
        
        migrator.registerMigration("Add rank to \(AssetRecord.databaseTableName)") { db in
            try? db.alter(table: AssetRecord.databaseTableName) {
                $0.add(column: AssetRecord.Columns.rank.name, .numeric).defaults(to: 0)
            }
        }
        
        migrator.registerMigration("Add lastUsedAt to \(BalanceRecord.databaseTableName)") { db in
            try? db.alter(table: BalanceRecord.databaseTableName) {
                $0.add(column: BalanceRecord.Columns.lastUsedAt.name, .date)
            }
        }
        
        migrator.registerMigration("Create \(AssetLinkRecord.databaseTableName)") { db in
            try? AssetLinkRecord.create(db: db)
        }
        
        migrator.registerMigration("Add market values to prices table \(PriceRecord.databaseTableName)") { db in
            try? db.alter(table: PriceRecord.databaseTableName) {
                $0.add(column: PriceRecord.Columns.marketCap.name, .double)
                $0.add(column: PriceRecord.Columns.marketCapRank.name, .integer)
                $0.add(column: PriceRecord.Columns.totalVolume.name, .double)
                $0.add(column: PriceRecord.Columns.circulatingSupply.name, .double)
                $0.add(column: PriceRecord.Columns.totalSupply.name, .double)
                $0.add(column: PriceRecord.Columns.maxSupply.name, .double)
            }
        }
        
        migrator.registerMigration("Add stakingApr to \(AssetRecord.databaseTableName)") { db in
            try? db.alter(table: AssetRecord.databaseTableName) {
                $0.add(column: AssetRecord.Columns.stakingApr.name, .double)
            }
        }
        
        migrator.registerMigration("Update \(BalanceRecord.Columns.totalAmount.name) column") { db in
            try? db.alter(table: BalanceRecord.databaseTableName) {
                $0.drop(column: BalanceRecord.Columns.totalAmount.name)
                $0.addColumn(sql: BalanceRecord.totalAmountSQlCreation)
            }
        }
        
        migrator.registerMigration("Add isActive to \(BalanceRecord.databaseTableName)") { db in
            try? db.alter(table: BalanceRecord.databaseTableName) {
                $0.add(column: BalanceRecord.Columns.isActive.name, .boolean).defaults(to: true)
            }
        }
      
        migrator.registerMigration("Add marketCapFdv table \(PriceRecord.databaseTableName)") { db in
            try? db.alter(table: PriceRecord.databaseTableName) {
                $0.add(column: PriceRecord.Columns.marketCapFdv.name, .double)
            }
        }
      
        // not revelevant for new users, only debug
        migrator.registerMigration("Add initial nft setup tables drop") { db in
            try? db.drop(table: NFTCollectionRecord.databaseTableName)
            try? db.drop(table: NFTAssetRecord.databaseTableName)
            try? db.drop(table: NFTAssetAssociationRecord.databaseTableName)
            try? db.drop(table: "nft_collection_images")
            try? db.drop(table: "nft_images")
            try? db.drop(table: "nft_attributes")
        }
        
        migrator.registerMigration("Add initial nft tables setup") { db in
            try NFTCollectionRecord.create(db: db)
            try NFTAssetRecord.create(db: db)
            try NFTAssetAssociationRecord.create(db: db)
        }
        
        migrator.registerMigration("Add links to \(NFTCollectionRecord.databaseTableName)") { db in
            try? db.alter(table: NFTCollectionRecord.databaseTableName) {
                $0.add(column: NFTCollectionRecord.Columns.links.name, .jsonText)
            }
        }
        
        migrator.registerMigration("Add attributes to \(NFTAssetRecord.databaseTableName)") { db in
            try? db.drop(table: "nft_attributes")
            try? db.alter(table: NFTAssetRecord.databaseTableName) {
                $0.add(column: NFTAssetRecord.Columns.attributes.name, .jsonText)
            }
        }
        
        migrator.registerMigration("Add contractAddress to \(NFTAssetRecord.databaseTableName)") { db in
            try? db.alter(table: NFTAssetRecord.databaseTableName) {
                $0.add(column: NFTAssetRecord.Columns.contractAddress.name, .text)
            }
        }

        migrator.registerMigration("Add imageUrl to \(WalletRecord.databaseTableName)") { db in
            try? db.alter(table: WalletRecord.databaseTableName) {
                $0.add(column: WalletRecord.Columns.imageUrl.name, .text)
                $0.add(column: WalletRecord.Columns.updatedAt.name, .date)
            }
        }
        
        migrator.registerMigration("Add currency to \(PriceAlertRecord.databaseTableName)") { db in
            try? db.alter(table: PriceAlertRecord.databaseTableName) {
                $0.add(column: PriceAlertRecord.Columns.currency.name, .text).defaults(to: "USD")
            }
        }
        
        migrator.registerMigration("Re-create nft tables") { db in
            try? db.drop(table: NFTAssetAssociationRecord.databaseTableName)
            try? db.drop(table: NFTAssetRecord.databaseTableName)
            try? db.drop(table: NFTCollectionRecord.databaseTableName)
            
            try NFTCollectionRecord.create(db: db)
            try NFTAssetRecord.create(db: db)
            try NFTAssetAssociationRecord.create(db: db)
        }
        
        migrator.registerMigration("Add fiat rates") { db in
            try? FiatRateRecord.create(db: db)
        }
        
        migrator.registerMigration("Add priceUsd to prices table \(PriceRecord.databaseTableName)") { db in
            try? db.alter(table: PriceRecord.databaseTableName) {
                $0.add(column: PriceRecord.Columns.priceUsd.name, .double)
                    .notNull()
                    .defaults(to: 0)
            }
        }
        
//        TODO: Market.
//        migrator.registerMigration("Add markets_assets table") { db in
//            try? MarketAssetRecord.create(db: db)
//        }

        migrator.registerMigration("Add updatedAt to \(PriceRecord.databaseTableName)") { db in
            try? db.alter(table: PriceRecord.databaseTableName) {
                $0.add(column: PriceRecord.Columns.updatedAt.name, .date)
            }
        }
        
        migrator.registerMigration("Add \(AddressRecord.databaseTableName) table") { db in
            try? AddressRecord.create(db: db)
        }
        
        migrator.registerMigration("Add Perpetuals tables") { db in
            try? Self.clearChainData(db, chain: "hypercore")
            
            try? db.drop(table: PerpetualRecord.databaseTableName)
            try? db.drop(table: PerpetualPositionRecord.databaseTableName)
            
            try? PerpetualRecord.create(db: db)
            try? PerpetualPositionRecord.create(db: db)
        }
        
        migrator.registerMigration("Add withdrawable to \(BalanceRecord.databaseTableName)") { db in
            try? db.alter(table: BalanceRecord.databaseTableName) {
                $0.add(column: BalanceRecord.Columns.withdrawable.name, .text)
                    .defaults(to: "0")
                $0.add(column: BalanceRecord.Columns.withdrawableAmount.name, .double)
                    .defaults(to: 0)
            }
        }
        
        migrator.registerMigration("Add metadata to \(BalanceRecord.databaseTableName)") { db in
            try? db.alter(table: BalanceRecord.databaseTableName) {
                $0.add(column: BalanceRecord.Columns.metadata.name, .jsonText)
            }
        }

        migrator.registerMigration("Clear metadata from \(BalanceRecord.databaseTableName)") { db in
            try? db.execute(sql: "UPDATE \(BalanceRecord.databaseTableName) SET metadata = NULL WHERE metadata IS NOT NULL")
        }
        
        migrator.registerMigration("Add isEnabled to \(AssetRecord.databaseTableName)") { db in
            try? db.alter(table: AssetRecord.databaseTableName) {
                $0.add(column: AssetRecord.Columns.isEnabled.name, .boolean).defaults(to: true)
            }
        }

        migrator.registerMigration("Add source to \(WalletRecord.databaseTableName)") { db in
            try? db.alter(table: WalletRecord.databaseTableName) {
                $0.add(column: WalletRecord.Columns.source.name, .text).defaults(to: WalletSource.create.rawValue)
            }
        }
        migrator.registerMigration("Add maxLeverage to \(PerpetualRecord.databaseTableName)") { db in
            try? db.alter(table: PerpetualRecord.databaseTableName) {
                $0.add(column: PerpetualRecord.Columns.maxLeverage.name, .integer)
                    .notNull()
                    .defaults(to: 1)
            }
        }

        migrator.registerMigration("Create \(RecentActivityRecord.databaseTableName)") { db in
            try? RecentActivityRecord.create(db: db)
        }

        migrator.registerMigration("Add pendingUnconfirmed to \(BalanceRecord.databaseTableName)") { db in
            try? db.alter(table: BalanceRecord.databaseTableName) {
                $0.add(column: BalanceRecord.Columns.pendingUnconfirmed.name, .text)
                    .defaults(to: "0")
                $0.add(column: BalanceRecord.Columns.pendingUnconfirmedAmount.name, .double)
                    .defaults(to: 0)
            }
        }

        migrator.registerMigration("Migrate nodes_selected_v1 to \(NodeSelectedRecord.databaseTableName)") { db in
            try? db.drop(table: NodeSelectedRecord.databaseTableName)
            try? NodeSelectedRecord.create(db: db)
            try? db.execute(sql: """
                INSERT INTO \(NodeSelectedRecord.databaseTableName) (chain, nodeUrl)
                SELECT ns.chain, n.url
                FROM nodes_selected_v1 ns
                INNER JOIN \(NodeRecord.databaseTableName) n ON ns.nodeId = n.id
            """)
            try? db.drop(table: "nodes_selected_v1")
        }

        migrator.registerMigration("Create \(SearchRecord.databaseTableName) and drop assets_search") { db in
            try? SearchRecord.create(db: db)
            try? db.drop(table: "assets_search")
        }

        migrator.registerMigration("Create \(NotificationRecord.databaseTableName)") { db in
            try? db.drop(table: NotificationRecord.databaseTableName)
            try? NotificationRecord.create(db: db)
        }

        migrator.registerMigration("Add allTimeHigh/Low to \(PriceRecord.databaseTableName)") { db in
            try? db.alter(table: PriceRecord.databaseTableName) {
                $0.add(column: PriceRecord.Columns.allTimeHigh.name, .double)
                $0.add(column: PriceRecord.Columns.allTimeHighDate.name, .date)
                $0.add(column: PriceRecord.Columns.allTimeHighChangePercentage.name, .double)
                $0.add(column: PriceRecord.Columns.allTimeLow.name, .double)
                $0.add(column: PriceRecord.Columns.allTimeLowDate.name, .date)
                $0.add(column: PriceRecord.Columns.allTimeLowChangePercentage.name, .double)
            }
        }

        migrator.registerMigration("Migrate wallet IDs to WalletIdentifier format") { db in
            try? db.alter(table: WalletRecord.databaseTableName) {
                $0.add(column: WalletRecord.Columns.externalId.name, .text)
            }
            try WalletIdMigration.migrate(db: db)
        }

        migrator.registerMigration("Add hasImage to \(AssetRecord.databaseTableName)") { db in
            try? db.alter(table: AssetRecord.databaseTableName) {
                $0.add(column: AssetRecord.Columns.hasImage.name, .boolean).defaults(to: false)
            }
        }

        migrator.registerMigration("Create \(ContactRecord.databaseTableName) and \(ContactAddressRecord.databaseTableName)") { db in
            try? ContactRecord.create(db: db)
            try? ContactAddressRecord.create(db: db)
        }

        migrator.registerMigration("Add type to \(AddressRecord.databaseTableName)") { db in
            try? db.alter(table: AddressRecord.databaseTableName) {
                $0.add(column: AddressRecord.Columns.type.name, .text)
            }
        }

        migrator.registerMigration("Add earn support") { db in
            try? db.alter(table: AssetRecord.databaseTableName) {
                $0.add(column: AssetRecord.Columns.isEarnable.name, .boolean).defaults(to: false)
                $0.add(column: AssetRecord.Columns.earnApr.name, .double)
            }

            try? db.alter(table: BalanceRecord.databaseTableName) {
                $0.add(column: BalanceRecord.Columns.earn.name, .text)
                    .defaults(to: "0")
                $0.add(column: BalanceRecord.Columns.earnAmount.name, .double)
                    .defaults(to: 0)
            }

            try? db.alter(table: BalanceRecord.databaseTableName) {
                $0.drop(column: BalanceRecord.Columns.totalAmount.name)
                $0.addColumn(sql: BalanceRecord.totalAmountSQlCreation)
            }
        }

        migrator.registerMigration("Add providerType to stake_validators") { db in
            try? db.alter(table: StakeValidatorRecord.databaseTableName) {
                $0.add(column: StakeValidatorRecord.Columns.providerType.name, .text)
                    .defaults(to: StakeProviderType.stake.rawValue)
            }
        }

        migrator.registerMigration("Add status to \(AddressRecord.databaseTableName) and \(NFTCollectionRecord.databaseTableName)") { db in
            try? db.alter(table: AddressRecord.databaseTableName) {
                $0.add(column: AddressRecord.Columns.status.name, .text)
                    .notNull()
                    .defaults(to: VerificationStatus.unverified.rawValue)
            }

            try? db.alter(table: NFTCollectionRecord.databaseTableName) {
                $0.add(column: NFTCollectionRecord.Columns.status.name, .text)
                    .notNull()
                    .defaults(to: VerificationStatus.unverified.rawValue)
            }
        }

        try migrator.migrate(dbQueue)
    }
}
