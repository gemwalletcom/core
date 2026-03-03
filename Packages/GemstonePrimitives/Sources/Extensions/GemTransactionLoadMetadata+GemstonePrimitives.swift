// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

extension GemTransactionLoadMetadata {
    public func map() throws -> TransactionLoadMetadata {
        switch self {
        case .none:
            return .none
        case .solana(let senderTokenAddress, let recipientTokenAddress, let tokenProgram, let blockHash):
            return .solana(
                senderTokenAddress: senderTokenAddress,
                recipientTokenAddress: recipientTokenAddress,
                tokenProgram: tokenProgram?.map(),
                blockHash: blockHash
            )
        case .ton(let senderTokenAddress, let recipientTokenAddress, let sequence):
            return .ton(
                senderTokenAddress: senderTokenAddress,
                recipientTokenAddress: recipientTokenAddress,
                sequence: sequence
            )
        case .cosmos(let accountNumber, let sequence, let chainId):
            return .cosmos(accountNumber: UInt64(accountNumber), sequence: sequence, chainId: chainId)
        case .bitcoin(let utxos):
            return .bitcoin(utxos: try utxos.map { try $0.map() })
        case .zcash(let utxos, let branchId):
            return .zcash(utxos: try utxos.map { try $0.map() }, branchId: branchId)
        case .cardano(let utxos):
            return .cardano(utxos: try utxos.map { try $0.map() })
        case .evm(let nonce, let chainId, let contractCall):
            return .evm(nonce: UInt64(nonce), chainId: UInt64(chainId), contractCall: contractCall?.map())
        case .near(let sequence, let blockHash):
            return .near(sequence: sequence, blockHash: blockHash)
        case .stellar(let sequence, let isDestinationAddressExist):
            return .stellar(sequence: sequence, isDestinationAddressExist: isDestinationAddressExist)
        case .xrp(let sequence, let blockNumber):
            return .xrp(sequence: sequence, blockNumber: blockNumber)
        case .algorand(let sequence, let blockHash, let chainId):
            return .algorand(sequence: sequence, blockHash: blockHash, chainId: chainId)
        case .aptos(let sequence, let data):
            return .aptos(sequence: sequence, data: data)
        case .polkadot(let sequence, let genesisHash, let blockHash, let blockNumber, let specVersion, let transactionVersion, let period):
            return .polkadot(
                sequence: sequence,
                genesisHash: genesisHash,
                blockHash: blockHash,
                blockNumber: UInt64(blockNumber),
                specVersion: specVersion,
                transactionVersion: transactionVersion,
                period: UInt64(period)
            )
        case .tron(
            let blockNumber,
            let blockVersion,
            let blockTimestamp,
            let transactionTreeRoot,
            let parentHash,
            let witnessAddress,
            let stakeData
        ):
            return .tron(
                blockNumber: UInt64(blockNumber),
                blockVersion: UInt64(blockVersion),
                blockTimestamp: UInt64(blockTimestamp),
                transactionTreeRoot: transactionTreeRoot,
                parentHash: parentHash,
                witnessAddress: witnessAddress,
                stakeData: stakeData.map()
            )
        case .sui(let messageBytes):
            return .sui(messageBytes: messageBytes)
        case .hyperliquid(let order):
            return .hyperliquid(order: order?.map())
        }
    }
}

extension TransactionLoadMetadata {
    public func map() -> GemTransactionLoadMetadata {
        switch self {
        case .none:
            return .none
        case .solana(let senderTokenAddress, let recipientTokenAddress, let tokenProgram, let blockHash):
            return .solana(
                senderTokenAddress: senderTokenAddress,
                recipientTokenAddress: recipientTokenAddress,
                tokenProgram: tokenProgram?.map(),
                blockHash: blockHash
            )
        case .ton(let senderTokenAddress, let recipientTokenAddress, let sequence):
            return .ton(senderTokenAddress: senderTokenAddress, recipientTokenAddress: recipientTokenAddress, sequence: sequence)
        case .cosmos(let accountNumber, let sequence, let chainId):
            return .cosmos(accountNumber: UInt64(accountNumber), sequence: sequence, chainId: chainId)
        case .bitcoin(let utxos):
            return .bitcoin(utxos: utxos.map { $0.map() })
        case .zcash(let utxos, let branchId):
            return .zcash(utxos: utxos.map { $0.map() }, branchId: branchId)
        case .cardano(let utxos):
            return .cardano(utxos: utxos.map { $0.map() })
        case .evm(let nonce, let chainId, let contractCall):
            return .evm(
                nonce: UInt64(nonce),
                chainId: UInt64(chainId),
                contractCall: contractCall?.map()
            )
        case .near(let sequence, let blockHash):
            return .near(sequence: sequence, blockHash: blockHash)
        case .stellar(let sequence, let isDestinationAddressExist):
            return .stellar(sequence: sequence, isDestinationAddressExist: isDestinationAddressExist)
        case .xrp(let sequence, let blockNumber):
            return .xrp(sequence: sequence, blockNumber: blockNumber)
        case .algorand(let sequence, let blockHash, let chainId):
            return .algorand(sequence: sequence, blockHash: blockHash, chainId: chainId)
        case .aptos(let sequence, let data):
            return .aptos(sequence: sequence, data: data)
        case .polkadot(let sequence, let genesisHash, let blockHash, let blockNumber, let specVersion, let transactionVersion, let period):
            return .polkadot(
                sequence: sequence,
                genesisHash: genesisHash,
                blockHash: blockHash,
                blockNumber: UInt64(blockNumber),
                specVersion: specVersion,
                transactionVersion: transactionVersion,
                period: UInt64(period)
            )
        case .tron(
            let blockNumber,
            let blockVersion,
            let blockTimestamp,
            let transactionTreeRoot,
            let parentHash,
            let witnessAddress,
            let stakeData
        ):
            return .tron(
                blockNumber: UInt64(blockNumber),
                blockVersion: UInt64(blockVersion),
                blockTimestamp: UInt64(blockTimestamp),
                transactionTreeRoot: transactionTreeRoot,
                parentHash: parentHash,
                witnessAddress: witnessAddress,
                stakeData: stakeData.map()
            )
        case .sui(let messageBytes):
            return .sui(messageBytes: messageBytes)
        case .hyperliquid(let order):
            return .hyperliquid(order: order?.map())
        }
    }
}

extension Gemstone.TronStakeData {
    func map() -> Primitives.TronStakeData {
        switch self {
        case .votes(let votes): .votes(votes.map { $0.map() })
        case .unfreeze(let amounts): .unfreeze(amounts.map { $0.map() })
        }
    }
}

extension Primitives.TronStakeData {
    func map() -> Gemstone.TronStakeData {
        switch self {
        case .votes(let votes): .votes(votes.map { $0.map() })
        case .unfreeze(let amounts): .unfreeze(amounts.map { $0.map() })
        }
    }
}

extension Gemstone.TronVote {
    func map() -> Primitives.TronVote {
        Primitives.TronVote(validator: validator, count: count)
    }
}

extension Primitives.TronVote {
    func map() -> Gemstone.TronVote {
        Gemstone.TronVote(validator: validator, count: count)
    }
}

extension Gemstone.TronUnfreeze {
    func map() -> Primitives.TronUnfreeze {
        Primitives.TronUnfreeze(resource: resource.map(), amount: amount)
    }
}

extension Primitives.TronUnfreeze {
    func map() -> Gemstone.TronUnfreeze {
        Gemstone.TronUnfreeze(resource: resource.map(), amount: amount)
    }
}

extension GemHyperliquidOrder {
    func map() -> HyperliquidOrder {
        HyperliquidOrder(
            approveAgentRequired: approveAgentRequired,
            approveReferralRequired: approveReferralRequired,
            approveBuilderRequired: approveBuilderRequired,
            builderFeeBps: UInt32(builderFeeBps),
            agentAddress: agentAddress,
            agentPrivateKey: agentPrivateKey
        )
    }
}

extension HyperliquidOrder {
    func map() -> GemHyperliquidOrder {
        GemHyperliquidOrder(
            approveAgentRequired: approveAgentRequired,
            approveReferralRequired: approveReferralRequired,
            approveBuilderRequired: approveBuilderRequired,
            builderFeeBps: builderFeeBps,
            agentAddress: agentAddress,
            agentPrivateKey: agentPrivateKey
        )
    }
}
