// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import WalletConnectSign
import struct Gemstone.SignMessage

public protocol WalletConnectorSignable: Sendable {
    var allChains: [Primitives.Chain] { get }

    func addConnection(connection: WalletConnection) throws
    func updateSessions(sessions: [WalletConnectionSession]) throws
    func sessionReject(id: String, error: any Error) async throws
    func getCurrentWallet() throws -> Primitives.Wallet
    func getWallet(id: WalletId) throws -> Primitives.Wallet
    func getChains(wallet: Wallet) -> [Primitives.Chain]
    func getAccounts(wallet: Wallet, chains: [Primitives.Chain]) -> [Primitives.Account]
    func getWallets(for proposal: Session.Proposal) throws -> [Wallet]
    func getMethods() -> [WalletConnectionMethods]
    func getEvents() -> [WalletConnectionEvents]
    func sessionApproval(payload: WCPairingProposal) async throws -> WalletId
    func signMessage(sessionId: String, chain: Chain, message: SignMessage) async throws -> String
    func signTransaction(sessionId: String, chain: Chain, transaction: WalletConnectorTransaction) async throws -> String
    func sendTransaction(sessionId: String, chain: Chain, transaction: WalletConnectorTransaction) async throws -> String
}
