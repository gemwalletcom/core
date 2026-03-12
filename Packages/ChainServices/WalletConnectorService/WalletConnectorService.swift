// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@preconcurrency import WalletConnectPairing
@preconcurrency import ReownWalletKit
import Primitives
import GemstonePrimitives
import struct Gemstone.SignMessage
import enum Gemstone.SignDigestType
import class Gemstone.WalletConnect
import enum Gemstone.WalletConnectAction
import enum Gemstone.WalletConnectTransaction
import enum Gemstone.WalletConnectTransactionType
import enum Gemstone.WalletConnectChainOperation
import enum Gemstone.WalletConnectResponseType

public final class WalletConnectorService {
    private let interactor = WCConnectionsInteractor()
    private let signer: WalletConnectorSignable
    private let messageTracker = MessageTracker()
    private let walletConnect = WalletConnect()
    public init(signer: WalletConnectorSignable) {
        self.signer = signer
    }
}

// MARK: - WalletConnectorService

extension WalletConnectorService: WalletConnectorServiceable {
    public func configure() throws {
        Networking.configure(
            groupIdentifier: Constants.WalletConnect.groupIdentifier,
            projectId: Constants.WalletConnect.projectId,
            socketFactory: DefaultSocketFactory()
        )

        try WalletKit.configure(
            metadata: AppMetadata(
                name: Constants.App.name,
                description: "Gem Web3 Wallet",
                url: Constants.App.website,
                icons: ["https://gemwallet.com/images/gem-logo-256x256.png"],
                redirect: AppMetadata.Redirect(
                    native: "gem://",
                    universal: .none
                )
            ),
            crypto: DefaultCryptoProvider()
        )
    }

    public func setup() async {
        Events.instance.setTelemetryEnabled(false)
        await withTaskGroup(of: Void.self) { group in
            group.addTask {
                await self.handleSessions()
            }

            group.addTask {
                await self.handleSessionProposals()
            }

            group.addTask {
                await self.handleSessionRequests()
            }
        }
    }

    public func pair(uri: String) async throws {
        let uri = try WalletConnectURI(uriString: uri)
        try await Pair.instance.pair(uri: uri)
    }

    public func disconnect(sessionId: String) async throws {
        try await WalletKit.instance.disconnect(topic: sessionId)
    }

    public func updateSessions() {
        updateSessions(interactor.sessions)
    }
}

// MARK: - Private

extension WalletConnectorService {
    private func handleSessions() async {
        for await sessions in interactor.sessionsStream {
            updateSessions(sessions)
        }
    }

    private func handleSessionProposals() async {
        for await (proposal, verifyContext) in interactor.sessionProposalStream {
            debugLog("Session proposal received: \(proposal)")
            debugLog("Verify context: \(String(describing: verifyContext))")

            guard let verifyContext = verifyContext else {
                await handleRejectSession(proposal: proposal, error: WalletConnectorServiceError.invalidOrigin)
                continue
            }

            do {
                try await processSession(proposal: proposal, verifyContext: verifyContext)
            } catch {
                debugLog("Error accepting proposal: \(error)")

                await handleRejectSession(proposal: proposal, error: error)
            }
        }
    }

    private func handleRejectSession(proposal: Session.Proposal, error: Error) async {
        try? await signer.sessionReject(id: proposal.pairingTopic, error: error)
    }

    private func handleSessionRequests() async {
        for await (request, verifyContext) in interactor.sessionRequestStream {
            debugLog("Session request received: \(request.method)")
            debugLog("Verify context: \(String(describing: verifyContext))")

            let session = WalletKit.instance.getSessions().first { $0.topic == request.topic }

            guard let verifyContext, let session else {
                continue
            }

            do {
                let status = walletConnect.validateOrigin(metadataUrl: session.peer.metadata.url, origin: verifyContext.origin, validation: verifyContext.validation.map()).map()

                debugLog("Verification status for request: \(status)")

                switch status {
                case .verified, .unknown: break
                case .invalid, .malicious:
                    // show toast with an error
                    debugLog("Warning: Request status error (\(status)")
                    try await rejectRequest(request)
                    continue
                }

                try await handleRequest(request: request, session: session)
            } catch {
                debugLog("Error handling request: \(error)")
            }
        }
    }

    private func updateSessions(_ sessions: [Session]) {
        debugLog("Received sessions: \(sessions)")
        do {
            try signer.updateSessions(sessions: sessions.map { $0.asSession })
        } catch {
            debugLog("Error updating sessions: \(error)")
        }
    }

    private func handleRequest(request: WalletConnectSign.Request, session: Session) async throws {
        let messageId = request.messageId

        guard await messageTracker.shouldProcess(messageId) else {
            debugLog("Ignoring duplicate request with ID: \(messageId)")
            try await rejectRequest(request)
            return
        }

        debugLog("handleMethod received: \(request.method), params: \(request.params)")

        do {
            let params = try JSONEncoder().encode(request.params).encodeString()
            let action = try walletConnect.parseRequest(
                topic: request.topic,
                method: request.method,
                params: params,
                chainId: request.chainId.absoluteString,
                domain: session.peer.metadata.url
            )

            debugLog("parse request result: \(action)")

            let response = try await handleAction(action: action, sessionId: request.topic, sessionDomain: session.peer.metadata.url)

            debugLog("handle method result: \(request.method) \(response)")
            try await WalletKit.instance.respond(topic: request.topic, requestId: request.id, response: response)
        } catch {
            debugLog("handle method error: \(error)")
            try await rejectRequest(request)
        }
    }

    private func handleAction(action: WalletConnectAction, sessionId: String, sessionDomain: String) async throws -> RPCResult {
        switch action {
        case .signMessage(let chain, let signType, let data):
            try walletConnect.validateSignMessage(chain: chain, signType: signType, data: data, sessionDomain: sessionDomain)
            let message = walletConnect.decodeSignMessage(chain: chain, signType: signType, data: data)
            let signature = try await signer.signMessage(
                sessionId: sessionId,
                chain: chain.map(),
                message: message
            )
            let response = walletConnect.encodeSignMessage(chain: chain, signature: signature)
            return .response(response.map())
        case .signTransaction(let chain, let type, let data):
            try walletConnect.validateSendTransaction(transactionType: type, data: data)
            let transaction = try walletConnect.decodeSendTransaction(transactionType: type, data: data)
            let transactionId = try await signer.signTransaction(sessionId: sessionId, chain: chain.map(), transaction: transaction.map())
            let response = walletConnect.encodeSignTransaction(chain: chain, transactionId: transactionId)
            return .response(response.map())
        case .sendTransaction(let chain, let type, let data):
            try walletConnect.validateSendTransaction(transactionType: type, data: data)
            let transaction = try walletConnect.decodeSendTransaction(transactionType: type, data: data)
            let transactionId = try await signer.sendTransaction(
                sessionId: sessionId,
                chain: chain.map(),
                transaction: transaction.map()
            )
            let response = walletConnect.encodeSendTransaction(chain: chain, transactionId: transactionId)
            return .response(response.map())
        case .chainOperation(let operation):
            return handleChainOperation(operation: operation)
        case .unsupported(let method):
            throw WalletConnectorServiceError.unresolvedMethod(method)
        }
    }

    private func handleChainOperation(operation: WalletConnectChainOperation) -> RPCResult {
        switch operation {
        case .addChain, .switchChain: .response(AnyCodable.null())
        case .getChainId: .error(.methodNotFound)
        }
    }

    private func rejectRequest(_ request: WalletConnectSign.Request) async throws {
        try await WalletKit.instance.respond(topic: request.topic, requestId: request.id, response: .error(JSONRPCError(code: 4001, message: "User rejected the request")))
    }

    private func processSession(proposal: Session.Proposal, verifyContext: VerifyContext) async throws {
        let messageId = proposal.messageId

        guard await messageTracker.shouldProcess(messageId) else {
            debugLog("Ignoring duplicate proposal with ID: \(messageId)")
            return
        }

        let wallets = try signer.getWallets(for: proposal)
        let currentWalletId = try signer.getCurrentWallet().walletId

        guard let preselectedWallet = wallets.first(where: { $0.walletId == currentWalletId }) ?? wallets.first else {
            throw WalletConnectorServiceError.walletsUnsupported
        }

        let metadata = proposal.proposer.metadata
        let status = walletConnect.validateOrigin(metadataUrl: metadata.url, origin: verifyContext.origin, validation: verifyContext.validation.map()).map()

        debugLog("Verification status: \(status)")

        switch status {
        case .verified, .unknown: break
        case .invalid, .malicious:
            throw WalletConnectorServiceError.invalidOrigin
        }

        let payload = WalletConnectionSessionProposal(
            defaultWallet: preselectedWallet,
            wallets: wallets,
            metadata: metadata
        )

        let payloadTopic = WCPairingProposal(
            pairingId: proposal.pairingTopic,
            proposal: payload,
            verificationStatus: status
        )
        let approvedWalletId = try await signer.sessionApproval(payload: payloadTopic)
        let selectedWallet = try signer.getWallet(id: approvedWalletId)

        let session = try await acceptProposal(proposal: proposal, wallet: selectedWallet)
        try signer.addConnection(connection: WalletConnection(session: session.asSession, wallet: selectedWallet))
    }

    private func acceptProposal(proposal: Session.Proposal, wallet: Wallet) async throws -> Session {
        let chains = signer.getChains(wallet: wallet)
        let accounts = signer.getAccounts(wallet: wallet, chains: chains)
        let events = signer.getEvents()
        let methods = signer.getMethods()
        let supportedAccounts = accounts.compactMap { $0.blockchain }
        let supportedChains = chains.compactMap { $0.blockchain }

        let sessionNamespaces = try AutoNamespaces.build(
            sessionProposal: proposal,
            chains: supportedChains,
            methods: methods.map { $0.rawValue },
            events: events.map { $0.rawValue },
            accounts: supportedAccounts
        )
        let sessionProperties = walletConnect.configSessionProperties(
            properties: proposal.sessionProperties ?? [:],
            chains: chains.map { $0.id }
        )

        return try await WalletKit.instance.approve(
            proposalId: proposal.id,
            namespaces: sessionNamespaces,
            sessionProperties: sessionProperties
        )
    }
}
