// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct HyperliquidOrder: Sendable {
    public let approveAgentRequired: Bool
    public let approveReferralRequired: Bool
    public let approveBuilderRequired: Bool
    public let builderFeeBps: UInt32
    public let agentAddress: String
    public let agentPrivateKey: String

    public init(
        approveAgentRequired: Bool,
        approveReferralRequired: Bool,
        approveBuilderRequired: Bool,
        builderFeeBps: UInt32,
        agentAddress: String,
        agentPrivateKey: String
    ) {
        self.approveAgentRequired = approveAgentRequired
        self.approveReferralRequired = approveReferralRequired
        self.approveBuilderRequired = approveBuilderRequired
        self.builderFeeBps = builderFeeBps
        self.agentAddress = agentAddress
        self.agentPrivateKey = agentPrivateKey
    }
}

public enum TransactionLoadMetadata: Sendable {
    case none
    case solana(
        senderTokenAddress: String?,
        recipientTokenAddress: String?,
        tokenProgram: SolanaTokenProgramId?,
        blockHash: String
    )
    case ton(
        senderTokenAddress: String?,
        recipientTokenAddress: String?,
        sequence: UInt64
    )
    case cosmos(
        accountNumber: UInt64,
        sequence: UInt64,
        chainId: String
    )
    case bitcoin(utxos: [UTXO])
    case zcash(utxos: [UTXO], branchId: String)
    case cardano(utxos: [UTXO])
    case evm(nonce: UInt64, chainId: UInt64, contractCall: ContractCallData? = nil)
    case near(
        sequence: UInt64,
        blockHash: String
    )
    case stellar(sequence: UInt64, isDestinationAddressExist: Bool)
    case xrp(sequence: UInt64, blockNumber: UInt64)
    case algorand(
        sequence: UInt64,
        blockHash: String,
        chainId: String
    )
    case aptos(sequence: UInt64, data: String? = nil)
    case polkadot(
        sequence: UInt64,
        genesisHash: String,
        blockHash: String,
        blockNumber: UInt64,
        specVersion: UInt64,
        transactionVersion: UInt64,
        period: UInt64
    )
    case tron(
        blockNumber: UInt64,
        blockVersion: UInt64,
        blockTimestamp: UInt64,
        transactionTreeRoot: String,
        parentHash: String,
        witnessAddress: String,
        stakeData: TronStakeData
    )
    case sui(messageBytes: String)
    case hyperliquid(order: HyperliquidOrder?)
}

extension TransactionLoadMetadata {
    public func getSequence() throws -> UInt64 {
        switch self {
        case .ton(_, _, let sequence),
             .cosmos(_, let sequence, _),
             .near(let sequence, _),
             .stellar(let sequence, _),
             .xrp(let sequence, _),
             .algorand(let sequence, _, _),
             .aptos(let sequence, _),
             .polkadot(let sequence, _, _, _, _, _, _),
             .evm(let sequence, _, _):
            return sequence
        case .none, .bitcoin, .zcash, .cardano, .tron, .solana, .sui, .hyperliquid:
            throw AnyError("Sequence not available for this metadata type")
        }
    }

    public func getBlockNumber() throws -> UInt64 {
        switch self {
        case .polkadot(_, _, _, let blockNumber, _, _, _),
             .tron(let blockNumber, _, _, _, _, _, _),
             .xrp(_, let blockNumber):
            return blockNumber
        default:
            throw AnyError("Block number not available for this metadata type")
        }
    }

    public func getBlockHash() throws -> String {
        switch self {
        case .solana(_, _, _, let blockHash),
            .near(_, let blockHash),
             .algorand(_, let blockHash, _),
             .polkadot(_, _, let blockHash, _, _, _, _):
            return blockHash
        default:
            throw AnyError("Block hash not available for this metadata type")
        }
    }

    public func getChainId() throws -> String {
        switch self {
        case .cosmos(_, _, let chainId):
            return chainId
        case .algorand(_, _, let chainId):
            return chainId
        case .evm(_, let chainId, _):
            return String(chainId)
        default:
            throw AnyError("Chain ID not available for this metadata type")
        }
    }

    public func getUtxos() throws -> [UTXO] {
        switch self {
        case .bitcoin(let utxos),
             .zcash(let utxos, _),
             .cardano(let utxos):
            return utxos
        default:
            throw AnyError("UTXOs not available for this metadata type")
        }
    }

    public func getIsDestinationAddressExist() throws -> Bool {
        switch self {
            case .stellar(_, let isDestinationAddressExist):
            return isDestinationAddressExist
        default:
            throw AnyError("Destination existence flag not available for this metadata type")
        }
    }

    public func getAccountNumber() throws -> UInt64 {
        switch self {
        case .cosmos(let accountNumber, _, _):
            return accountNumber
        default:
            throw AnyError("Account number not available for this metadata type")
        }
    }

    public func getMessageBytes() throws -> String {
        switch self {
        case .sui(let messageBytes):
            return messageBytes
        default:
            throw AnyError("Message bytes not available for this metadata type")
        }
    }

    public func senderTokenAddress() throws -> String? {
        switch self {
        case .ton(let senderTokenAddress, _, _):
            return senderTokenAddress
        default:
            throw AnyError("Sender token address not available for this metadata type")
        }
    }

    public func getData() throws -> String {
        let data: String? = switch self {
        case .aptos(_, let data): data
        default: .none
        }
        guard let data = data else {
            throw AnyError("Data not available for this metadata type")
        }
        return data
    }

}
