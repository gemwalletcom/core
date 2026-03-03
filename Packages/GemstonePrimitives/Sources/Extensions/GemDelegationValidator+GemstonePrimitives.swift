// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

extension GemStakeProviderType {
    public func map() -> StakeProviderType {
        switch self {
        case .stake: .stake
        case .earn: .earn
        }
    }
}

extension StakeProviderType {
    public func map() -> GemStakeProviderType {
        switch self {
        case .stake: .stake
        case .earn: .earn
        }
    }
}

extension GemDelegationValidator {
    public func map() throws -> DelegationValidator {
        DelegationValidator(
            chain: try chain.map(),
            id: id,
            name: name,
            isActive: isActive,
            commission: commission,
            apr: apr,
            providerType: providerType.map()
        )
    }
}

extension DelegationValidator {
    public func map() -> GemDelegationValidator {
        return GemDelegationValidator(
            chain: chain.rawValue,
            id: id,
            name: name,
            isActive: isActive,
            commission: commission,
            apr: apr,
            providerType: providerType.map()
        )
    }
}
