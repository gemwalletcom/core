// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import ReownWalletKit

extension Session {
    var asSession: Primitives.WalletConnectionSession {
        let sessionChains = namespaces.values
            .flatMap { $0.accounts.map(\.blockchain) }
            .compactMap(\.chain)

        return WalletConnectionSession(
            id: topic,
            sessionId: topic,
            state: .active,
            chains: sessionChains,
            createdAt: .now,
            expireAt: expiryDate,
            metadata: peer.metadata
        )
    }
}

extension Session.Proposal {
    var messageId: String {
        "proposal-\(id)"
    }
}
