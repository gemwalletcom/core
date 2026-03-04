// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol NameServiceable: Sendable {
    func getName(name: String, chain: String) async throws -> NameRecord?
}

public extension NameServiceable {
    func canResolveName(name: String) -> Bool {
        let nameParts = name.split(separator: ".")
        guard nameParts.count >= 2 && nameParts.last?.count ?? 0 >= 1 else {
            return false
        }
        return true
    }
}
