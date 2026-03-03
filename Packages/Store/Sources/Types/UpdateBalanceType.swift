// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public enum UpdateBalanceType {
    case coin(UpdateCoinBalance)
    case token(UpdateTokenBalance)
    case stake(UpdateStakeBalance)
    case perpetual(UpdatePerpetualBalance)
    case earn(UpdateEarnBalance)

    var metadata: BalanceMetadata? {
        switch self {
        case .stake(let balance): balance.metadata
        case .coin, .perpetual, .token, .earn: .none
        }
    }
}

public struct UpdateCoinBalance {
    public let available: UpdateBalanceValue
    public let reserved: UpdateBalanceValue
    public let pendingUnconfirmed: UpdateBalanceValue

    public init(
        available: UpdateBalanceValue,
        reserved: UpdateBalanceValue,
        pendingUnconfirmed: UpdateBalanceValue = .zero
    ) {
        self.available = available
        self.reserved = reserved
        self.pendingUnconfirmed = pendingUnconfirmed
    }
}

public struct UpdateTokenBalance {
    public let available: UpdateBalanceValue

    public init(available: UpdateBalanceValue) {
        self.available = available
    }
}

public struct UpdateStakeBalance {
    public let staked: UpdateBalanceValue
    public let pending: UpdateBalanceValue
    public let frozen: UpdateBalanceValue
    public let locked: UpdateBalanceValue
    public let rewards: UpdateBalanceValue
    public let metadata: BalanceMetadata?
    
    public init(
        staked: UpdateBalanceValue,
        pending: UpdateBalanceValue,
        frozen: UpdateBalanceValue,
        locked: UpdateBalanceValue,
        rewards: UpdateBalanceValue,
        metadata: BalanceMetadata? = nil
    ) {
        self.staked = staked
        self.pending = pending
        self.frozen = frozen
        self.locked = locked
        self.rewards = rewards
        self.metadata = metadata
    }
}

public struct UpdatePerpetualBalance {
    public let available: UpdateBalanceValue
    public let reserved: UpdateBalanceValue
    public let withdrawable: UpdateBalanceValue

    public init(
        available: UpdateBalanceValue,
        reserved: UpdateBalanceValue,
        withdrawable: UpdateBalanceValue
    ) {
        self.available = available
        self.reserved = reserved
        self.withdrawable = withdrawable
    }
}

public struct UpdateEarnBalance {
    public let balance: UpdateBalanceValue

    public init(balance: UpdateBalanceValue) {
        self.balance = balance
    }
}
