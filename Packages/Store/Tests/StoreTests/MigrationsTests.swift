// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Foundation
import GRDB
import StoreTestKit
@testable import Store

struct MigrationsTests {
    
    @Test
    func run() throws {
        let db = DB.mock()
        
        try db.dbQueue.read { db in
            #expect(try! db.tableExists(WalletRecord.databaseTableName))
            #expect(try! db.tableExists(AccountRecord.databaseTableName))
            #expect(try! db.tableExists(AssetRecord.databaseTableName))
            #expect(try! db.tableExists(BalanceRecord.databaseTableName))
            #expect(try! db.tableExists(TransactionRecord.databaseTableName))
            #expect(try! db.tableExists(NodeRecord.databaseTableName))
            #expect(try! db.tableExists(BannerRecord.databaseTableName))
            #expect(try! db.tableExists(NFTCollectionRecord.databaseTableName))
        }
    }
    
    @Test
    func runChanges() throws {
        let db = DB.mock()
        var migrations = Migrations()
        
        try migrations.run(dbQueue: db.dbQueue)
        try migrations.runChanges(dbQueue: db.dbQueue)
        
        try db.dbQueue.read { db in
            let walletColumns = try db.columns(in: WalletRecord.databaseTableName)
            #expect(walletColumns.contains(where: { $0.name == WalletRecord.Columns.isPinned.name }))
            
            let balanceColumns = try db.columns(in: BalanceRecord.databaseTableName)
            #expect(balanceColumns.contains(where: { $0.name == BalanceRecord.Columns.availableAmount.name }))
            #expect(balanceColumns.contains(where: { $0.name == BalanceRecord.Columns.isActive.name }))
            #expect(balanceColumns.contains(where: { $0.name == BalanceRecord.Columns.earn.name }))
            #expect(balanceColumns.contains(where: { $0.name == BalanceRecord.Columns.earnAmount.name }))
            
            let assetColumns = try db.columns(in: AssetRecord.databaseTableName)
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.isSellable.name }))
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.isStakeable.name }))
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.isEarnable.name }))
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.earnApr.name }))
            #expect(assetColumns.contains(where: { $0.name == AssetRecord.Columns.rank.name }))

            let validatorColumns = try db.columns(in: StakeValidatorRecord.databaseTableName)
            #expect(validatorColumns.contains(where: { $0.name == StakeValidatorRecord.Columns.providerType.name }))
            
            let priceColumns = try db.columns(in: PriceRecord.databaseTableName)
            #expect(priceColumns.contains(where: { $0.name == PriceRecord.Columns.marketCap.name }))
            #expect(priceColumns.contains(where: { $0.name == PriceRecord.Columns.priceUsd.name }))
            
            #expect(try! db.tableExists(AssetLinkRecord.databaseTableName))
            #expect(try! db.tableExists(SearchRecord.databaseTableName))
            #expect(try! db.tableExists(FiatRateRecord.databaseTableName))
            #expect(try! db.tableExists(AddressRecord.databaseTableName))
        }
    }
}
