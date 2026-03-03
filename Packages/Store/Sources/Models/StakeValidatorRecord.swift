// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import GRDB

struct StakeValidatorRecord: Codable, FetchableRecord, PersistableRecord  {
    static let databaseTableName: String = "stake_validators"
    
    enum Columns {
        static let id = Column("id")
        static let assetId = Column("assetId")
        static let validatorId = Column("validatorId")
        static let name = Column("name")
        static let isActive = Column("isActive")
        static let commission = Column("commission")
        static let apr = Column("apr")
        static let providerType = Column("providerType")
    }

    var id: String
    var assetId: AssetId
    var validatorId: String
    var name: String
    var isActive: Bool
    var commission: Double
    var apr: Double
    var providerType: StakeProviderType
}

extension StakeValidatorRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: Self.databaseTableName) {
            $0.primaryKey(Columns.id.name, .text)
                .notNull()
                .indexed()
            $0.column(Columns.assetId.name, .text)
                .notNull()
                .references(AssetRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
            $0.column(Columns.validatorId.name, .text)
                .notNull()
            $0.column(Columns.name.name, .text)
                .notNull()
            $0.column(Columns.isActive.name, .boolean)
                .notNull()
            $0.column(Columns.commission.name, .double)
                .notNull()
            $0.column(Columns.apr.name, .double)
                .notNull()
            $0.column(Columns.providerType.name, .text)
                .notNull()
        }
    }
}

extension StakeValidatorRecord {
    var validator: DelegationValidator {
        DelegationValidator(
            chain: assetId.chain,
            id: validatorId,
            name: name,
            isActive: isActive,
            commission: commission,
            apr: apr,
            providerType: providerType
        )
    }
}

extension DelegationValidator {
    static func recordId(chain: Chain, validatorId: String) -> String {
        return "\(chain.rawValue)_\(validatorId)"
    }
}

extension DelegationValidator {
    var record: StakeValidatorRecord {
        StakeValidatorRecord(
            id: DelegationValidator.recordId(chain: chain, validatorId: id),
            assetId: chain.assetId,
            validatorId: id,
            name: name,
            isActive: isActive,
            commission: commission,
            apr: apr,
            providerType: providerType
        )
    }
}
