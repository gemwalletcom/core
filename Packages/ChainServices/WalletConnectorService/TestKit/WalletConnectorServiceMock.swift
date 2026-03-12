// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import WalletConnectorService

public actor WalletConnectorServiceMock: WalletConnectorServiceable {
    public var isSetup: Bool = false
    
    public init() {}

    public func setup() async { isSetup = true }
    public func pair(uri: String) async throws {}
    public func disconnect(sessionId: String) async throws {}
    nonisolated public func configure() throws {}
    nonisolated public func updateSessions() {}
}
