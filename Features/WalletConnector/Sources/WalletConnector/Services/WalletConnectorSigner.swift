// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import WalletConnectorService
import Primitives
import Store
import Preferences
import BigInt
import class Gemstone.Config
import WalletConnectSign
import WalletSessionService
import struct Gemstone.SignMessage
import class Gemstone.MessageSigner
import enum Gemstone.MessagePreview
import GemstonePrimitives

public final class WalletConnectorSigner: WalletConnectorSignable {
    private let connectionsStore: ConnectionsStore
    private let walletConnectorInteractor: any WalletConnectorInteractable
    private let walletSessionService: any WalletSessionManageable

    public init(
        connectionsStore: ConnectionsStore,
        walletSessionService: any WalletSessionManageable,
        walletConnectorInteractor: any WalletConnectorInteractable
    ) {
        self.connectionsStore = connectionsStore
        self.walletConnectorInteractor = walletConnectorInteractor
        self.walletSessionService = walletSessionService
    }

    public var allChains: [Primitives.Chain]  {
        Config.shared.getWalletConnectConfig().chains.compactMap { Chain(rawValue: $0) }
    }

    public func getCurrentWallet() throws -> Wallet {
        try walletSessionService.getCurrentWallet()
    }

    public func getWallet(id: WalletId) throws -> Wallet {
        try walletSessionService.getWallet(walletId: id)
    }

    public func getChains(wallet: Wallet) -> [Primitives.Chain] {
        wallet.accounts.map { $0.chain }.asSet().intersection(allChains).asArray()
    }

    public func getAccounts(wallet: Wallet, chains: [Primitives.Chain]) -> [Primitives.Account] {
        wallet.accounts.filter { chains.contains($0.chain) }
    }

    public func getWallets(for proposal: Session.Proposal) throws -> [Wallet] {
        let requiredChains = proposal.requiredChains
        let optionalChains = proposal.optionalChains

        return try walletSessionService.getWallets()
            .filter {
                guard !$0.isViewOnly else { return false }

                let walletChains = $0.accounts.map(\.chain).asSet()
                if requiredChains.isNotEmpty {
                    return walletChains.isSuperset(of: requiredChains)
                }

                return optionalChains.isEmpty || walletChains.contains(where: optionalChains.contains)
            }
    }

    public func getEvents() -> [WalletConnectionEvents] {
        WalletConnectionEvents.allCases
    }

    public func getMethods() -> [WalletConnectionMethods] {
        WalletConnectionMethods.allCases
    }

    public func sessionApproval(payload: WCPairingProposal) async throws -> WalletId {
        try await walletConnectorInteractor.sessionApproval(payload: payload)
    }

    public func signMessage(sessionId: String, chain: Chain, message: SignMessage) async throws -> String {
        let session = try connectionsStore.getConnection(id: sessionId)
        try validate(chain: chain, session: session.session)
        let payload = SignMessagePayload(
            chain: chain,
            session: session.session,
            wallet: session.wallet,
            message: message
        )
        return try await walletConnectorInteractor.signMessage(payload: payload)
    }

    public func updateSessions(sessions: [WalletConnectionSession]) throws {
        if sessions.isEmpty {
            try? connectionsStore.deleteAll()
        } else {
            let newSessionIds = sessions.map { $0.id }.asSet()
            let sessionIds = try connectionsStore.getSessions().filter { $0.state == .active }.map { $0.id }.asSet()
            let deleteIds = sessionIds.subtracting(newSessionIds).asArray()

            try? connectionsStore.delete(ids: deleteIds)

            for session in sessions {
                try? connectionsStore.updateConnectionSession(session)
            }
        }
    }

    public func sessionReject(id: String, error: any Error) async throws {
        try connectionsStore.delete(ids: [id])
        await walletConnectorInteractor.sessionReject(error: error)
    }

    private func buildTransferData(
        chain: Chain,
        metadata: WalletConnectionSessionAppMetadata,
        transaction: String,
        outputType: TransferDataOutputType,
        outputAction: TransferDataOutputAction
    ) -> TransferData {
        TransferData(
            type: .generic(
                asset: chain.asset,
                metadata: metadata,
                extra: TransferDataExtra(
                    to: "",
                    data: transaction.data(using: .utf8),
                    outputType: outputType,
                    outputAction: outputAction
                )
            ),
            recipientData: RecipientData(
                recipient: Recipient(name: .none, address: "", memo: .none),
                amount: .none
            ),
            value: .zero
        )
    }

    private func buildBitcoinTransferData(chain: Chain, transaction: String) throws -> TransferData {
        let transfer = try JSONDecoder().decode(WCBitcoinTransfer.self, from: try transaction.encodedData())
        return TransferData(
            type: .transfer(chain.asset),
            recipientData: RecipientData(
                recipient: Recipient(name: .none, address: transfer.recipientAddress, memo: transfer.memo),
                amount: .none
            ),
            value: BigInt(stringLiteral: transfer.amount),
            canChangeValue: false
        )
    }

    public func signTransaction(sessionId: String, chain: Chain, transaction: WalletConnectorTransaction) async throws -> String {
        let session = try connectionsStore.getConnection(id: sessionId)
        try validate(chain: chain, session: session.session)
        let wallet = try getWallet(id: session.wallet.walletId)

        switch transaction {
        case .ethereum, .bitcoin:
            throw AnyError("Not supported")
        case .solana(let transaction, let outputType),
             .sui(let transaction, let outputType),
             .ton(let transaction, let outputType),
             .tron(let transaction, let outputType):
            let transferData = buildTransferData(
                chain: chain,
                metadata: session.session.metadata,
                transaction: transaction,
                outputType: outputType,
                outputAction: .sign
            )
            return try await walletConnectorInteractor.signTransaction(transferData: WCTransferData(tranferData: transferData, wallet: wallet))
        }
    }

    public func sendTransaction(sessionId: String, chain: Chain, transaction: WalletConnectorTransaction) async throws -> String {
        let session = try connectionsStore.getConnection(id: sessionId)
        try validate(chain: chain, session: session.session)
        let wallet = try getWallet(id: session.wallet.walletId)

        switch transaction {
        case .ethereum(let transaction):
            let address = transaction.to
            let value = try BigInt.fromHex(transaction.value ?? .zero)
            let gasLimit: BigInt? = {
                if let value = transaction.gasLimit {
                    return BigInt(hex: value)
                } else if let gas = transaction.gas {
                    return BigInt(hex: gas)
                }
                return .none
            }()

            let gasPrice: GasPriceType? = {
                if let maxFeePerGas = transaction.maxFeePerGas,
                   let maxPriorityFeePerGas = transaction.maxPriorityFeePerGas,
                   let maxFeePerGasBigInt = BigInt(hex: maxFeePerGas),
                   let maxPriorityFeePerGasBigInt = BigInt(hex: maxPriorityFeePerGas)
                {
                    return .eip1559(gasPrice: maxFeePerGasBigInt, priorityFee: maxPriorityFeePerGasBigInt)
                }
                return .none
            }()
            let data: Data? = {
                if let data = transaction.data {
                    return Data(hex: data)
                }
                return .none
            }()

            let transferData = TransferData(
                type: .generic(asset: chain.asset, metadata: session.session.metadata, extra: TransferDataExtra(
                    to: address,
                    gasLimit: gasLimit,
                    gasPrice: gasPrice,
                    data: data
                )),
                recipientData: RecipientData(
                    recipient: Recipient(name: .none, address: address, memo: .none),
                    amount: .none
                ),
                value: value
            )

            return try await walletConnectorInteractor.sendTransaction(transferData: WCTransferData(tranferData: transferData, wallet: wallet))
        case .solana(let transaction, let outputType),
             .sui(let transaction, let outputType),
             .ton(let transaction, let outputType),
             .tron(let transaction, let outputType):
            let transferData = buildTransferData(
                chain: chain,
                metadata: session.session.metadata,
                transaction: transaction,
                outputType: outputType,
                outputAction: .send
            )
            return try await walletConnectorInteractor.sendTransaction(transferData: WCTransferData(tranferData: transferData, wallet: wallet))
        case .bitcoin(let transaction, _):
            let transferData = try buildBitcoinTransferData(chain: chain, transaction: transaction)
            return try await walletConnectorInteractor.sendTransaction(transferData: WCTransferData(tranferData: transferData, wallet: wallet))
        }
    }

    private func validate(chain: Chain, session: WalletConnectionSession) throws {
        if !session.chains.contains(chain) {
            throw WalletConnectorServiceError.unresolvedChainId(chain.rawValue)
        }
    }

    public func addConnection(connection: WalletConnection) throws {
        try connectionsStore.addConnection(connection)
    }
}

extension Session.Proposal {
    var requiredChains: Set<Chain> {
        requiredNamespaces.values
            .flatMap { $0.chains ?? [] }
            .compactMap(\.chain)
            .asSet()
    }

    var optionalChains: Set<Chain> {
        optionalNamespaces?.values
            .flatMap { $0.chains ?? [] }
            .compactMap(\.chain)
            .asSet() ?? []
    }
}
