// Copyright (c). Gem Wallet. All rights reserved.

public enum DelegationActionType: Hashable, Identifiable {
    public var id: Self { self }

    case stake, unstake, redelegate
    case deposit
    case withdraw
}
