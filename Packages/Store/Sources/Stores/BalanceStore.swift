// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct BalanceStore: Sendable {
    
    let db: DatabaseQueue

    public init(db: DB) {
        self.db = db.dbQueue
    }
    
    public func addBalance(
        _ balances: [AddBalance],
        for walletId: WalletId
    ) throws {
        try db.write { (db: Database) in
            for balance in balances {
                try balance.mapToAssetBalanceRecord(walletId: walletId.id)
                    .insert(db, onConflict: .ignore)
            }
        }
    }

    public func updateBalances(
        _ balances: [UpdateBalance],
        for walletId: WalletId
    ) throws {
        try db.write { (db: Database) in
            for balance in balances {
                let balanceFields: [ColumnAssignment] = switch balance.type {
                case .coin(let balance):
                    [
                        BalanceRecord.Columns.available.set(to: balance.available.value),
                        BalanceRecord.Columns.availableAmount.set(to: balance.available.amount),
                        BalanceRecord.Columns.reserved.set(to: balance.reserved.value),
                        BalanceRecord.Columns.reservedAmount.set(to: balance.reserved.amount),
                        BalanceRecord.Columns.pendingUnconfirmed.set(to: balance.pendingUnconfirmed.value),
                        BalanceRecord.Columns.pendingUnconfirmedAmount.set(to: balance.pendingUnconfirmed.amount)
                    ]
                case .token(let balance):
                    [
                        BalanceRecord.Columns.available.set(to: balance.available.value),
                        BalanceRecord.Columns.availableAmount.set(to: balance.available.amount),
                    ]
                case .stake(let balance):
                    [
                        BalanceRecord.Columns.staked.set(to: balance.staked.value),
                        BalanceRecord.Columns.stakedAmount.set(to: balance.staked.amount),
                        BalanceRecord.Columns.frozen.set(to: balance.frozen.value),
                        BalanceRecord.Columns.frozenAmount.set(to: balance.frozen.amount),
                        BalanceRecord.Columns.locked.set(to: balance.locked.value),
                        BalanceRecord.Columns.lockedAmount.set(to: balance.locked.amount),
                        BalanceRecord.Columns.pending.set(to: balance.pending.value),
                        BalanceRecord.Columns.pendingAmount.set(to: balance.pending.amount),
                        BalanceRecord.Columns.rewards.set(to: balance.rewards.value),
                        BalanceRecord.Columns.rewardsAmount.set(to: balance.rewards.amount),
                    ]
                case .perpetual(let balance):
                    [
                        BalanceRecord.Columns.available.set(to: balance.available.value),
                        BalanceRecord.Columns.availableAmount.set(to: balance.available.amount),
                        BalanceRecord.Columns.reserved.set(to: balance.reserved.value),
                        BalanceRecord.Columns.reservedAmount.set(to: balance.reserved.amount),
                        BalanceRecord.Columns.withdrawable.set(to: balance.withdrawable.value),
                        BalanceRecord.Columns.withdrawableAmount.set(to: balance.withdrawable.amount)
                    ]
                case .earn(let balance):
                    [
                        BalanceRecord.Columns.earn.set(to: balance.balance.value),
                        BalanceRecord.Columns.earnAmount.set(to: balance.balance.amount)
                    ]
                }

                let defaultFields: [ColumnAssignment] = try {
                    var items: [ColumnAssignment] = [
                        BalanceRecord.Columns.updatedAt.set(to: balance.updatedAt),
                        BalanceRecord.Columns.isActive.set(to: balance.isActive),
                    ]

                    if let metadata = balance.type.metadata {
                        let metadataString = try JSONEncoder().encode(metadata).encodeString()
                        items.append(BalanceRecord.Columns.metadata.set(to: metadataString))
                    }
                    return items
                }()

                let assignments = balanceFields + defaultFields
                
                try BalanceRecord
                    .filter(BalanceRecord.Columns.walletId == walletId.id)
                    .filter(BalanceRecord.Columns.assetId == balance.assetId.identifier)
                    .updateAll(db, assignments)
            }
        }
    }

    @discardableResult
    public func getBalance(walletId: WalletId, assetId: AssetId) throws -> Balance? {
        try db.read { db in
            return try BalanceRecord
                .filter(BalanceRecord.Columns.walletId == walletId.id)
                .filter(BalanceRecord.Columns.assetId == assetId.identifier)
                .fetchOne(db)?.mapToBalance()
        }
    }

    @discardableResult
    func getBalanceRecord(walletId: WalletId, assetId: AssetId) throws -> BalanceRecord? {
        try db.read { db in
            return try BalanceRecord
                .filter(BalanceRecord.Columns.walletId == walletId.id)
                .filter(BalanceRecord.Columns.assetId == assetId.identifier)
                .fetchOne(db)
        }
    }

    @discardableResult
    public func getBalances(assetIds: [String]) throws -> [WalletAssetBalance] {
        try db.read { db in
            return try BalanceRecord
                .filter(assetIds.contains(BalanceRecord.Columns.assetId))
                .fetchAll(db)
                .map { $0.mapToWalletAssetBalanace() }
        }
    }
    
    @discardableResult
    public func getBalances(walletId: WalletId, assetIds: [AssetId]) throws -> [AssetBalance] {
        try db.read { db in
            return try BalanceRecord
                .filter(BalanceRecord.Columns.walletId == walletId.id)
                .filter(assetIds.map { $0.identifier }.contains(BalanceRecord.Columns.assetId))
                .distinct()
                .fetchAll(db)
                .map { $0.mapToAssetBalance() }
        }
    }

    @discardableResult
    public func isBalanceExist(walletId: WalletId, assetId: AssetId) throws -> Bool {
        try db.read { db in
            return try BalanceRecord
                .filter(BalanceRecord.Columns.walletId == walletId.id)
                .filter(BalanceRecord.Columns.assetId == assetId.identifier)
                .fetchCount(db) > 0
        }
    }

    @discardableResult
    public func setIsEnabled(walletId: WalletId, assetIds: [AssetId], value: Bool) throws -> Int {
        try db.write { db in
            let assignments = switch value {
            case true: [
                BalanceRecord.Columns.isEnabled.set(to: true),
            ]
            case false: [
                BalanceRecord.Columns.isEnabled.set(to: false),
                BalanceRecord.Columns.isPinned.set(to: false)
            ]}

            return try BalanceRecord
                .filter(BalanceRecord.Columns.walletId == walletId.id)
                .filter(assetIds.map { $0.identifier }.contains(BalanceRecord.Columns.assetId))
                .updateAll(db, assignments)
        }
    }

    @discardableResult
    public func pinAsset(walletId: WalletId, assetId: AssetId, value: Bool) throws -> Int {
        try db.write { db in
            return try BalanceRecord
                .filter(BalanceRecord.Columns.walletId == walletId.id)
                .filter(BalanceRecord.Columns.assetId == assetId.identifier)
                .updateAll(db, BalanceRecord.Columns.isPinned.set(to: value))
        }
    }

    private func getMissingAssetIds(walletId: WalletId, assetIds: [AssetId]) throws -> [AssetId] {
        try db.read { db in
            let existingAssetIds = try BalanceRecord
                .filter(BalanceRecord.Columns.walletId == walletId.id)
                .filter(assetIds.map { $0.identifier }.contains(BalanceRecord.Columns.assetId))
                .select(BalanceRecord.Columns.assetId)
                .fetchAll(db)
                .map { $0[BalanceRecord.Columns.assetId] as String }
                .asSet()

            return assetIds.filter { !existingAssetIds.contains($0.identifier) }
        }
    }

    public func addMissingBalances(walletId: WalletId, assetIds: [AssetId], isEnabled: Bool = false) throws {
        let missingAssetIds = try getMissingAssetIds(walletId: walletId, assetIds: assetIds)
        let missingBalances = missingAssetIds.map { assetId in
            AddBalance(assetId: assetId, isEnabled: isEnabled)
        }

        if !missingBalances.isEmpty {
            try addBalance(missingBalances, for: walletId)
        }
    }
}
