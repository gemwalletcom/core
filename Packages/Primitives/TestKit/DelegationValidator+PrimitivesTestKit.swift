// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension DelegationValidator {
    static func mock(
        _ chain: Chain = Chain.bitcoin,
        id: String = "1",
        name: String = "Test Delegation Validator",
        isActive: Bool = true,
        commission: Double = 5,
        apr: Double = 1,
        providerType: StakeProviderType = .stake
    ) -> DelegationValidator {
        DelegationValidator(
            chain: chain,
            id: id,
            name: name,
            isActive: isActive,
            commission: commission,
            apr: apr,
            providerType: providerType
        )
    }
}
