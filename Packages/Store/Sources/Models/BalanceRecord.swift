// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import GRDB

internal import BigInt

struct BalanceRecord: Codable, FetchableRecord, PersistableRecord  {
    
    static let databaseTableName: String = "balances"
    
    enum Columns {
        static let assetId = Column("assetId")
        static let walletId = Column("walletId")
        static let isEnabled = Column("isEnabled")
        static let isPinned = Column("isPinned")
        static let isActive = Column("isActive")
        static let available = Column("available")
        static let availableAmount = Column("availableAmount")
        static let frozen = Column("frozen")
        static let frozenAmount = Column("frozenAmount")
        static let locked = Column("locked")
        static let lockedAmount = Column("lockedAmount")
        static let staked = Column("staked")
        static let stakedAmount = Column("stakedAmount")
        static let pending = Column("pending")
        static let pendingAmount = Column("pendingAmount")
        static let pendingUnconfirmed = Column("pendingUnconfirmed")
        static let pendingUnconfirmedAmount = Column("pendingUnconfirmedAmount")
        static let rewards = Column("rewards")
        static let rewardsAmount = Column("rewardsAmount")
        static let reserved = Column("reserved")
        static let reservedAmount = Column("reservedAmount")
        static let withdrawable = Column("withdrawable")
        static let withdrawableAmount = Column("withdrawableAmount")
        static let earn = Column("earn")
        static let earnAmount = Column("earnAmount")
        static let totalAmount = Column("totalAmount")
        static let metadata = Column("metadata")
        static let lastUsedAt = Column("lastUsedAt")
        static let updatedAt = Column("updatedAt")
    }

    var assetId: AssetId
    var walletId: String
    
    var available: String
    var availableAmount: Double
    
    var frozen: String
    var frozenAmount: Double
    
    var locked: String
    var lockedAmount: Double
    
    var staked: String
    var stakedAmount: Double
    
    var pending: String
    var pendingAmount: Double

    var pendingUnconfirmed: String
    var pendingUnconfirmedAmount: Double

    var rewards: String
    var rewardsAmount: Double
    
    var reserved: String
    var reservedAmount: Double
    
    var withdrawable: String
    var withdrawableAmount: Double

    var earn: String
    var earnAmount: Double

    var totalAmount: Double
    
    var isEnabled: Bool
    var isPinned: Bool
    var isActive: Bool
    
    var metadata: BalanceMetadata?

    var lastUsedAt: Date?
    var updatedAt: Date?
}

extension BalanceRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: Self.databaseTableName, ifNotExists: true) {
            $0.column(Columns.assetId.name, .text)
                .notNull()
                .references(AssetRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
            $0.column(Columns.walletId.name, .text)
                .notNull()
                .indexed()
                .references(WalletRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
            
            // balances
            $0.column(Columns.available.name, .text).defaults(to: "0")
            $0.column(Columns.availableAmount.name, .numeric).defaults(to: 0)
            
            $0.column(Columns.frozen.name, .text).defaults(to: "0")
            $0.column(Columns.frozenAmount.name, .double).defaults(to: 0)
            
            $0.column(Columns.locked.name, .text).defaults(to: "0")
            $0.column(Columns.lockedAmount.name, .double).defaults(to: 0)
            
            $0.column(Columns.staked.name, .text).defaults(to: "0")
            $0.column(Columns.stakedAmount.name, .double).defaults(to: 0)
            
            $0.column(Columns.pending.name, .text).defaults(to: "0")
            $0.column(Columns.pendingAmount.name, .double).defaults(to: 0)

            $0.column(Columns.pendingUnconfirmed.name, .text).defaults(to: "0")
            $0.column(Columns.pendingUnconfirmedAmount.name, .double).defaults(to: 0)

            $0.column(Columns.rewards.name, .text).defaults(to: "0")
            $0.column(Columns.rewardsAmount.name, .double).defaults(to: 0)
            
            $0.column(Columns.reserved.name, .text).defaults(to: "0")
            $0.column(Columns.reservedAmount.name, .double).defaults(to: 0)
            
            $0.column(Columns.withdrawable.name, .text).defaults(to: "0")
            $0.column(Columns.withdrawableAmount.name, .double).defaults(to: 0)

            $0.column(Columns.earn.name, .text).defaults(to: "0")
            $0.column(Columns.earnAmount.name, .double).defaults(to: 0)

            $0.column(sql: totalAmountSQlCreation)
            
            $0.column(Columns.isEnabled.name, .boolean).defaults(to: true).indexed()
            $0.column(Columns.isPinned.name, .boolean).defaults(to: false).indexed()
            $0.column(Columns.isActive.name, .boolean).defaults(to: true).indexed()
            
            $0.column(Columns.metadata.name, .jsonText)
            $0.column(Columns.lastUsedAt.name, .date)
            $0.column(Columns.updatedAt.name, .date)
            $0.uniqueKey([
                Columns.assetId.name,
                Columns.walletId.name,
            ])
        }
    }
    
    static let totalAmountSQlCreation = "totalAmount DOUBLE AS (availableAmount + frozenAmount + lockedAmount + stakedAmount + pendingAmount + rewardsAmount + earnAmount)"
}

extension BalanceRecord: Identifiable {
    var id: String { assetId.identifier }
}

extension BalanceRecord {
    func mapToBalance() -> Balance {
        return Balance(
            available: BigInt(stringLiteral: available),
            frozen: BigInt(stringLiteral: frozen),
            locked: BigInt(stringLiteral: locked),
            staked: BigInt(stringLiteral: staked),
            pending: BigInt(stringLiteral: pending),
            pendingUnconfirmed: BigInt(stringLiteral: pendingUnconfirmed),
            rewards: BigInt(stringLiteral: rewards),
            reserved: BigInt(stringLiteral: reserved),
            withdrawable: BigInt(stringLiteral: withdrawable),
            earn: BigInt(stringLiteral: earn),
            metadata: metadata
        )
    }
    
    func mapToAssetBalance() -> AssetBalance {
        return AssetBalance(
            assetId: assetId,
            balance: mapToBalance()
        )
    }
    
    func mapToWalletAssetBalanace() -> WalletAssetBalance {
        return WalletAssetBalance(
            walletId: walletId,
            balance: mapToAssetBalance()
        )
    }
}
