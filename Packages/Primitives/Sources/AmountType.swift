// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum StakeAmountType: Equatable, Hashable, Sendable {
    case stake(validators: [DelegationValidator], recommended: DelegationValidator?)
    case unstake(Delegation)
    case redelegate(Delegation, validators: [DelegationValidator], recommended: DelegationValidator?)
    case withdraw(Delegation)
}

public enum AmountType: Equatable, Hashable, Sendable {
    case transfer(recipient: RecipientData)
    case deposit(recipient: RecipientData)
    case withdraw(recipient: RecipientData)
    case stake(StakeAmountType)
    case freeze(data: FreezeData)
    case perpetual(PerpetualRecipientData)
    case earn(EarnType)
}
