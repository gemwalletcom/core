// Copyright (c). Gem Wallet. All rights reserved.

extension StakeType {
    public var validatorId: String {
        return switch self {
        case .stake(let validator): validator.id
        case .unstake(let delegation): delegation.validator.id
        case .redelegate(let data): data.delegation.validator.id
        case .rewards(let validators): validators.first?.id ?? .empty
        case .withdraw(let delegation): delegation.validator.id
        case .freeze, .unfreeze: .empty
        }
    }
}
