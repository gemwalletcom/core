// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import BigInt

extension Delegation: Identifiable {
    public var id: String {
        base.id
    }
}

extension DelegationBase: Identifiable {
    public var id: String {
        [assetId.identifier, validatorId, state.rawValue, delegationId].joined(separator: "_")
    }
}

extension DelegationValidator: Identifiable {}

public extension DelegationBase {
    var balanceValue: BigInt {
        BigInt(stringLiteral: balance)
    }

    var sharesValue: BigInt {
        BigInt(stringLiteral: shares)
    }

    var rewardsValue: BigInt {
        BigInt(stringLiteral: rewards)
    }
}

extension DelegationValidator {
    public static let systemId = "system"
    public static let legacySystemId = "unstaking"

    public static func system(chain: Chain, name: String) -> DelegationValidator {
        DelegationValidator(
            chain: chain,
            id: systemId,
            name: name,
            isActive: true,
            commission: .zero,
            apr: .zero,
            providerType: .stake
        )
    }
}

extension DelegationState {
    public init(id: String) throws {
        if let state = DelegationState(rawValue: id) {
            self = state
        } else {
            throw AnyError("invalid state: \(id)")
        }
    }
}
