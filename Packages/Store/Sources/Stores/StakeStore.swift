// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct StakeStore: Sendable {
    
    let db: DatabaseQueue
    
    public init(db: DB) {
        self.db = db.dbQueue
    }
    
    public func getStakeApr(assetId: AssetId) throws -> Double? {
        try db.read { db in
            try AssetRecord
                .filter(key: assetId.identifier)
                .fetchOne(db)
                .map { $0.stakingApr } ?? .none
        }
    }

    public func getEarnApr(assetId: AssetId) throws -> Double? {
        try db.read { db in
            try AssetRecord
                .filter(key: assetId.identifier)
                .fetchOne(db)
                .map { $0.earnApr } ?? .none
        }
    }
    
    public func updateDelegations(walletId: WalletId, delegations: [DelegationBase]) throws {
        try db.write { db in
            for delegation in delegations {
                try delegation.record(walletId: walletId.id).upsert(db)
            }
        }
    }

    public func updateAndDelete(walletId: WalletId, delegations: [DelegationBase], deleteIds: [String]) throws {
        try db.write { db in
            for delegation in delegations {
                try delegation.record(walletId: walletId.id).upsert(db)
            }

            try StakeDelegationRecord
                .filter(StakeDelegationRecord.Columns.walletId == walletId.id)
                .filter(deleteIds.contains(StakeDelegationRecord.Columns.id))
                .deleteAll(db)
        }
    }

    public func getValidator(assetId: AssetId, validatorId: String) throws -> DelegationValidator? {
        try db.read { db in
            try StakeValidatorRecord
                .filter(StakeValidatorRecord.Columns.assetId == assetId.identifier)
                .filter(StakeValidatorRecord.Columns.validatorId == validatorId)
                .fetchOne(db)
                .map { $0.validator }
        }
    }
    
    public func getValidatorsActive(assetId: AssetId, providerType: StakeProviderType) throws -> [DelegationValidator] {
        try db.read { db in
            try ValidatorsRequest(chain: assetId.chain, providerType: providerType).fetch(db)
        }
    }

    public func getValidators(assetId: AssetId, providerType: StakeProviderType) throws -> [DelegationValidator] {
        try db.read { db in
            try StakeValidatorRecord
                .filter(StakeValidatorRecord.Columns.assetId == assetId.identifier)
                .filter(StakeValidatorRecord.Columns.providerType == providerType.rawValue)
                .order(StakeValidatorRecord.Columns.apr.desc)
                .fetchAll(db)
                .map { $0.validator }
        }
    }
    
    public func updateValidators(_ validators: [DelegationValidator]) throws {
        try db.write { db in
            for validator in validators {
                try validator.record.upsert(db)
            }
        }
    }
    
    public func getDelegations(walletId: WalletId, assetId: AssetId, providerType: StakeProviderType) throws -> [Delegation] {
        try db.read { db in
            try DelegationsRequest(walletId: walletId, assetId: assetId, providerType: providerType).fetch(db)
        }
    }

    @discardableResult
    public func deleteDelegations(walletId: WalletId, ids: [String]) throws -> Int {
        try db.write { db in
            try StakeDelegationRecord
                .filter(StakeDelegationRecord.Columns.walletId == walletId.id)
                .filter(ids.contains(StakeDelegationRecord.Columns.id))
                .deleteAll(db)
        }
    }

    @discardableResult
    public func clearDelegations() throws -> Int {
        try db.write { db in
            try StakeDelegationRecord.deleteAll(db)
        }
    }
    
    @discardableResult
    public func clearValidators() throws -> Int {
        try db.write { db in
            try StakeValidatorRecord.deleteAll(db)
        }
    }
    
    public func clear() throws {
        try clearDelegations()
        try clearValidators()
    }
}

