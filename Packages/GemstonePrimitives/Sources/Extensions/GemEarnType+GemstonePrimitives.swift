// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

extension Gemstone.GemEarnType {
    public func map() throws -> Primitives.EarnType {
        switch self {
        case .deposit(let validator): .deposit(try validator.map())
        case .withdraw(let delegation): .withdraw(try delegation.map())
        }
    }
}

extension Primitives.EarnType {
    public func map() -> Gemstone.GemEarnType {
        switch self {
        case .deposit(let validator): .deposit(validator.map())
        case .withdraw(let delegation): .withdraw(delegation.map())
        }
    }
}
