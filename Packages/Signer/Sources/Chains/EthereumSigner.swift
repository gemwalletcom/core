// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Keystore
import Primitives
import WalletCore

internal import BigInt
internal import GemstonePrimitives

class EthereumSigner: Signable {
    func signTransfer(input: SignerInput, privateKey: Data) throws -> String {
        let signingInput = try buildSigningInput(
            input: input,
            transaction: .with {
                $0.transfer = EthereumTransaction.Transfer.with {
                    $0.amount = input.value.magnitude.serialize()
                }
            },
            toAddress: input.destinationAddress,
            privateKey: privateKey
        )
        return try sign(coinType: input.coinType, input: signingInput)
    }

    func signTokenTransfer(input: SignerInput, privateKey: Data) throws -> String {
        let signingInput = try buildSigningInput(
            input: input,
            transaction: .with {
                $0.erc20Transfer = EthereumTransaction.ERC20Transfer.with {
                    $0.to = input.destinationAddress
                    $0.amount = input.value.magnitude.serialize()
                }
            },
            toAddress: input.asset.getTokenId(),
            privateKey: privateKey
        )
        return try sign(coinType: input.coinType, input: signingInput)
    }

    func signNftTransfer(input: SignerInput, privateKey: Data) throws -> String {
        guard case .transferNft(let asset) = input.type else {
            throw AnyError("Invalid type for NFT transfer")
        }
        let transaction: EthereumTransaction = switch asset.tokenType {
        case .erc721: EthereumTransaction.with {
            $0.erc721Transfer = .with {
                $0.from = input.senderAddress
                $0.to = input.destinationAddress
                $0.tokenID = BigInt(stringLiteral: asset.tokenId).magnitude.serialize()
            }
        }
        case .erc1155: EthereumTransaction.with {
            $0.erc1155Transfer = .with {
                $0.from = input.senderAddress
                $0.to = input.destinationAddress
                $0.tokenID = BigInt(stringLiteral: asset.tokenId).magnitude.serialize()
                $0.value = BigInt(1).magnitude.serialize()
            }
        }
        case .jetton, .spl: throw AnyError("Unsupported NFT token type for Ethereum")
        }

        let signingInput = try buildSigningInput(
            input: input,
            transaction: transaction,
            toAddress: asset.getContractAddress(),
            privateKey: privateKey
        )
        return try sign(coinType: input.coinType, input: signingInput)
    }

    func signData(input: Primitives.SignerInput, privateKey: Data) throws -> String {
        guard case .generic(_, _, let extra) = input.type else {
            throw AnyError("Invalid type for generic data signing")
        }
        let signingInput = try buildSigningInput(
            input: input,
            transaction: .with {
                $0.contractGeneric = EthereumTransaction.ContractGeneric.with {
                    $0.amount = input.value.magnitude.serialize()
                    $0.data = extra.data ?? Data()
                }
            },
            toAddress: input.destinationAddress,
            privateKey: privateKey
        )
        return try sign(coinType: input.coinType, input: signingInput)
    }

    func signTokenApproval(input: SignerInput, privateKey: Data) throws -> String {
        guard case .tokenApprove(_, let approvalData) = input.type else {
            throw AnyError("Invalid type for token approval")
        }
        return try sign(coinType: input.coinType, input: buildApprovalInput(input: input, approval: approvalData, privateKey: privateKey))
    }

    func signSwap(input: SignerInput, privateKey: Data) throws -> [String] {
        let swapData = try input.type.swap().data.data
        let callData = try Data.from(hex: swapData.data)
        let amount = swapData.asValue().magnitude.serialize()
        let gasLimit = swapData.approval != nil ? try swapData.gasLimitBigInt() : input.fee.gasLimit

        return try signContractCall(
            input: input,
            approval: swapData.approval,
            contractAddress: swapData.to,
            amount: amount,
            callData: callData,
            gasLimit: gasLimit,
            privateKey: privateKey
        )
    }

    func signEarn(input: SignerInput, privateKey: Data) throws -> [String] {
        let earnData = try input.type.earn().data
        let callData = try Data.from(hex: earnData.callData)
        let gasLimit = earnData.gasLimit.flatMap { BigInt($0) } ?? input.fee.gasLimit

        return try signContractCall(
            input: input,
            approval: earnData.approval,
            contractAddress: earnData.contractAddress,
            amount: Data(),
            callData: callData,
            gasLimit: gasLimit,
            privateKey: privateKey
        )
    }

    func signStake(input: SignerInput, privateKey: Data) throws -> [String] {
        guard case .stake(_, let stakeType) = input.type else {
            throw AnyError("Invalid type for staking")
        }

        guard
            case .evm(_, _, let contractCall) = input.metadata,
            let contractCall = contractCall
        else {
            throw AnyError("Invalid metadata for {\(input.asset.chain)} staking")
        }

        let callData = try Data.from(hex: contractCall.callData)
        let valueData = try {
            switch input.asset.chain {
            case .ethereum:
                return switch stakeType {
                case .stake: input.value.magnitude.serialize()
                case .unstake, .withdraw: Data()
                case .freeze, .redelegate, .rewards:
                    throw AnyError("Ethereum doesn't support this stake type")
                }
            case .smartChain:
                return switch stakeType {
                case .stake: input.value.magnitude.serialize()
                case .redelegate, .unstake, .rewards, .withdraw: Data()
                case .freeze: throw AnyError("SmartChain does not support freeze operations")
                }
            case .monad:
                return switch stakeType {
                case .stake: input.value.magnitude.serialize()
                case .unstake, .withdraw, .rewards: Data()
                case .redelegate, .freeze: throw AnyError("Monad doesn't support this stake type")
                }
            default:
                throw AnyError("\(input.asset.chain) native staking not supported")
            }
        }()

        return try signContractCall(
            input: input,
            approval: nil,
            contractAddress: contractCall.contractAddress,
            amount: valueData,
            callData: callData,
            gasLimit: input.fee.gasLimit,
            privateKey: privateKey
        )
    }

    func signMessage(message: SignMessage, privateKey: Data) throws -> String {
        guard let privateKey = PrivateKey(data: privateKey) else {
            throw AnyError("Unable to get private key")
        }
        switch message {
        case .typed(let message):
            return EthereumMessageSigner.signTypedMessage(privateKey: privateKey, messageJson: message)
        case .raw:
            throw AnyError("Raw message signing is not supported for Ethereum")
        }
    }
}

// MARK: - Private

extension EthereumSigner {
    private func buildSigningInput(
        input: SignerInput,
        transaction: EthereumTransaction,
        toAddress: String,
        nonce: BigInt? = nil,
        gasLimit: BigInt? = nil,
        privateKey: Data
    ) throws -> EthereumSigningInput {
        guard case .eip1559(let gasPrice, let priorityFee) = input.fee.gasPriceType else {
            throw AnyError("no longer supported")
        }
        let nonce = try nonce ?? BigInt(input.metadata.getSequence())
        let gasLimit = gasLimit ?? input.fee.gasLimit
        return try EthereumSigningInput.with {
            $0.txMode = .enveloped
            $0.maxFeePerGas = gasPrice.magnitude.serialize()
            $0.maxInclusionFeePerGas = priorityFee.magnitude.serialize()
            $0.gasLimit = gasLimit.magnitude.serialize()
            $0.chainID = try BigInt(stringLiteral: input.metadata.getChainId()).magnitude.serialize()
            $0.nonce = nonce.magnitude.serialize()
            $0.transaction = transaction
            $0.toAddress = toAddress
            $0.privateKey = privateKey
        }
    }

    private func buildApprovalInput(
        input: SignerInput,
        approval: ApprovalData,
        privateKey: Data
    ) throws -> EthereumSigningInput {
        try buildSigningInput(
            input: input,
            transaction: .with {
                $0.erc20Approve = EthereumTransaction.ERC20Approve.with {
                    $0.spender = approval.spender
                    $0.amount = BigInt.MAX_256.magnitude.serialize()
                }
            },
            toAddress: approval.token,
            privateKey: privateKey
        )
    }

    // https://github.com/trustwallet/wallet-core/blob/master/swift/Tests/Blockchains/EthereumTests.swift
    private func sign(coinType: CoinType, input: EthereumSigningInput) throws -> String {
        let output: EthereumSigningOutput = AnySigner.sign(input: input, coin: coinType)
        guard output.error == .ok else {
            throw AnyError("Failed to sign Ethereum tx: " + String(reflecting: output.error))
        }
        return output.encoded.hexString
    }

    private func signContractCall(
        input: SignerInput,
        approval: ApprovalData?,
        contractAddress: String,
        amount: Data,
        callData: Data,
        gasLimit: BigInt,
        privateKey: Data
    ) throws -> [String] {
        let nonce = try BigInt(input.metadata.getSequence())
        let contractInput = try buildSigningInput(
            input: input,
            transaction: .with {
                $0.contractGeneric = EthereumTransaction.ContractGeneric.with {
                    $0.amount = amount
                    $0.data = callData
                }
            },
            toAddress: contractAddress,
            nonce: approval != nil ? nonce + 1 : nonce,
            gasLimit: gasLimit,
            privateKey: privateKey
        )

        if let approval {
            return try [
                sign(coinType: input.coinType, input: buildApprovalInput(input: input, approval: approval, privateKey: privateKey)),
                sign(coinType: input.coinType, input: contractInput),
            ]
        } else {
            return try [sign(coinType: input.coinType, input: contractInput)]
        }
    }
}
