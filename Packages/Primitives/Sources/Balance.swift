// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import BigInt

public struct Balance: Codable, Equatable, Hashable, Sendable {
    public var available: BigInt
    public var frozen: BigInt
    public var locked: BigInt
    public var staked: BigInt
    public var pending: BigInt
    public var pendingUnconfirmed: BigInt
    public var rewards: BigInt
    public var reserved: BigInt
    public var withdrawable: BigInt
    public var earn: BigInt
    public var metadata: BalanceMetadata?

    public init(
        available: BigInt = .zero,
        frozen: BigInt = .zero,
        locked: BigInt = .zero,
        staked: BigInt = .zero,
        pending: BigInt = .zero,
        pendingUnconfirmed: BigInt = .zero,
        rewards: BigInt = .zero,
        reserved: BigInt = .zero,
        withdrawable: BigInt = .zero,
        earn: BigInt = .zero,
        metadata: BalanceMetadata? = .none
    ) {
        self.available = available
        self.frozen = frozen
        self.locked = locked
        self.staked = staked
        self.pending = pending
        self.pendingUnconfirmed = pendingUnconfirmed
        self.rewards = rewards
        self.reserved = reserved
        self.withdrawable = withdrawable
        self.earn = earn
        self.metadata = metadata
    }
}
