// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store
import WalletConnectorService
import Primitives
import Preferences

public final class ConnectionsService: Sendable {
    private let store: ConnectionsStore
    private let signer: any WalletConnectorSignable
    private let connector: WalletConnectorServiceable
    private let preferences: Preferences

    public var isWalletConnectActivated: Bool {
        get { preferences.isWalletConnectActivated == true }
        set { preferences.isWalletConnectActivated = newValue }
    }

    public init(
        store: ConnectionsStore,
        signer: any WalletConnectorSignable,
        connector: WalletConnectorServiceable,
        preferences: Preferences = .standard
    ) {
        self.store = store
        self.signer = signer
        self.connector = connector
        self.preferences = preferences
    }

    public convenience init(
        store: ConnectionsStore,
        signer: any WalletConnectorSignable,
        nodeProvider: any NodeURLFetchable,
        preferences: Preferences = .standard
    ) {
        self.init(
            store: store,
            signer: signer,
            connector: WalletConnectorService(signer: signer, nodeProvider: nodeProvider),
            preferences: preferences
        )
    }
}

// MARK: - Public

extension ConnectionsService {
    public func setup() async throws {
        checkExistSessions()
        try connector.configure()
        if isWalletConnectActivated {
            try await setupConnector()
        }
    }

    public func pair(uri: String) async throws {
        if !isWalletConnectActivated {
            try await setupConnector()
        }
        try await connector.pair(uri: uri)
    }

    public func disconnect(session: WalletConnectionSession) async throws {
        try await disconnect(sessionId: session.sessionId)
    }

    public func updateSessions() {
        connector.updateSessions()
    }
}

// MARK: - Private

extension ConnectionsService {
    private func disconnect(sessionId: String) async throws {
        try store.delete(ids: [sessionId])
        try await connector.disconnect(sessionId: sessionId)
    }

    private func setupConnector() async throws {
        if !isWalletConnectActivated {
            isWalletConnectActivated = true
        }
        await connector.setup()
    }

    // TODO: - Remove migration 08.2025
    private func checkExistSessions() {
        if preferences.isWalletConnectActivated == nil {
            isWalletConnectActivated = (try? store.getSessions().isNotEmpty) == true
        }
    }
}
