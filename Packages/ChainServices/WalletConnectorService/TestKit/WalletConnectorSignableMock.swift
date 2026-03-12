// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import WalletConnectSign
import struct Gemstone.SignMessage
import WalletConnectorService

public struct WalletConnectorSignableMock: WalletConnectorSignable {
    public var allChains: [Chain] = []

    public init() {}

    public func addConnection(connection: WalletConnection) throws {}
    public func updateSessions(sessions: [WalletConnectionSession]) throws {}
    public func sessionReject(id: String, error: any Error) async throws {}
    public func getCurrentWallet() throws -> Wallet { throw AnyError("not implemented") }
    public func getWallet(id: WalletId) throws -> Wallet { throw AnyError("not implemented") }
    public func getChains(wallet: Wallet) -> [Chain] { [] }
    public func getAccounts(wallet: Wallet, chains: [Chain]) -> [Primitives.Account] { [] }
    public func getWallets(for proposal: Session.Proposal) throws -> [Wallet] { [] }
    public func getMethods() -> [WalletConnectionMethods] { [] }
    public func getEvents() -> [WalletConnectionEvents] { [] }
    public func sessionApproval(payload: WCPairingProposal) async throws -> WalletId { throw AnyError("not implemented") }
    public func signMessage(sessionId: String, chain: Chain, message: SignMessage) async throws -> String { "" }
    public func signTransaction(sessionId: String, chain: Chain, transaction: WalletConnectorTransaction) async throws -> String { "" }
    public func sendTransaction(sessionId: String, chain: Chain, transaction: WalletConnectorTransaction) async throws -> String { "" }
}
