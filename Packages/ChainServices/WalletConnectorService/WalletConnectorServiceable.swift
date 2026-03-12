// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol WalletConnectorServiceable: Sendable {
    func configure() throws
    func setup() async
    func pair(uri: String) async throws
    func disconnect(sessionId: String) async throws
    func updateSessions()
}
