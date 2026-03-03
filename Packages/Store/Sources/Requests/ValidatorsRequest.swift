// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct ValidatorsRequest: DatabaseQueryable {

    private let chain: Chain
    private let providerType: StakeProviderType

    public init(chain: Chain, providerType: StakeProviderType) {
        self.chain = chain
        self.providerType = providerType
    }

    public func fetch(_ db: Database) throws -> [DelegationValidator] {
        let excludeValidatorIds = [DelegationValidator.systemId, DelegationValidator.legacySystemId]
        return try StakeValidatorRecord
            .filter(StakeValidatorRecord.Columns.assetId == chain.assetId.identifier)
            .filter(StakeValidatorRecord.Columns.providerType == providerType.rawValue)
            .filter(StakeValidatorRecord.Columns.isActive == true)
            .filter(!excludeValidatorIds.contains(StakeValidatorRecord.Columns.validatorId))
            .filter(StakeValidatorRecord.Columns.name != "")
            .order(StakeValidatorRecord.Columns.apr.desc)
            .fetchAll(db)
            .map { $0.validator }
    }
}
