// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Keystore
import Primitives
import WalletCore

internal import BigInt

struct SolanaSigner: Signable {
    func signTransfer(input: SignerInput, privateKey: Data) throws -> String {
        let coinType = input.coinType
        let type = SolanaSigningInput.OneOf_TransactionType.transferTransaction(.with {
            $0.recipient = input.destinationAddress
            $0.value = input.value.asUInt
            $0.memo = input.memo.valueOrEmpty
        })

        return try sign(input: input, type: type, coinType: coinType, privateKey: privateKey)
    }

    func signTokenTransfer(input: SignerInput, privateKey: Data) throws -> String {
        let coinType = input.coinType
        let decimals = UInt32(input.asset.decimals)
        let tokenId = try input.asset.getTokenId()
        let amount = input.value.asUInt
        let destinationAddress = input.destinationAddress

        guard case .solana(let senderTokenAddress, let recipientTokenAddress, let solanaTokenProgram, _) = input.metadata,
              let token = solanaTokenProgram, let senderTokenAddress = senderTokenAddress
        else {
            throw AnyError("unknown solana metadata")
        }

        switch recipientTokenAddress {
        case .some(let recipientTokenAddress):
            let type = SolanaSigningInput.OneOf_TransactionType.tokenTransferTransaction(.with {
                $0.amount = amount
                $0.decimals = decimals
                $0.tokenMintAddress = tokenId
                $0.senderTokenAddress = senderTokenAddress
                $0.recipientTokenAddress = recipientTokenAddress
                $0.memo = input.memo.valueOrEmpty
                $0.tokenProgramID = token.program
            })
            return try sign(input: input, type: type, coinType: coinType, privateKey: privateKey)
        case .none:
            let walletAddress = SolanaAddress(string: destinationAddress)!
            let calculatedRecipientTokenAddress = switch token {
            case .token:
                walletAddress.defaultTokenAddress(tokenMintAddress: tokenId)!
            case .token2022:
                walletAddress.token2022Address(tokenMintAddress: tokenId)!
            }
            let type = SolanaSigningInput.OneOf_TransactionType.createAndTransferTokenTransaction(.with {
                $0.amount = amount
                $0.decimals = decimals
                $0.recipientMainAddress = destinationAddress
                $0.tokenMintAddress = tokenId
                $0.senderTokenAddress = senderTokenAddress
                $0.recipientTokenAddress = calculatedRecipientTokenAddress
                $0.memo = input.memo.valueOrEmpty
                $0.tokenProgramID = token.program
            })
            return try sign(input: input, type: type, coinType: coinType, privateKey: privateKey)
        }
    }

    func signNftTransfer(input: SignerInput, privateKey: Data) throws -> String {
        throw AnyError.notImplemented
    }

    private func sign(input: SignerInput, type: SolanaSigningInput.OneOf_TransactionType, coinType: CoinType, privateKey: Data) throws -> String {
        let unitPrice = input.fee.gasPriceType.unitPrice

        let signingInput = try SolanaSigningInput.with {
            $0.transactionType = type
            $0.recentBlockhash = try input.metadata.getBlockHash()
            $0.priorityFeeLimit = .with {
                $0.limit = UInt32(input.fee.gasLimit)
            }
            if unitPrice > 0 {
                $0.priorityFeePrice = .with {
                    $0.price = UInt64(unitPrice)
                }
            }
            $0.privateKey = privateKey
        }
        let output: SolanaSigningOutput = AnySigner.sign(input: signingInput, coin: coinType)

        if !output.errorMessage.isEmpty {
            throw AnyError(output.errorMessage)
        }

        let encoded = try transcodeBase58ToBase64(output.encoded)
        return try signRawTransaction(transaction: encoded, privateKey: privateKey)
    }

    func signData(input: SignerInput, privateKey: Data) throws -> String {
        try ChainSigner(chain: .solana).signData(input: input, privateKey: privateKey)
    }

    func signRawTransaction(transaction: String, privateKey: Data) throws -> String {
        guard let transactionData = Base64.decode(string: transaction) else {
            throw AnyError("unable to decode base64 string")
        }
        let decodeOutputData = TransactionDecoder.decode(coinType: .solana, encodedTx: transactionData)
        let decodeOutput = try SolanaDecodingTransactionOutput(serializedBytes: decodeOutputData)

        let signingInput = SolanaSigningInput.with {
            $0.privateKey = privateKey
            $0.rawMessage = decodeOutput.transaction
            $0.txEncoding = .base64
        }
        let output: SolanaSigningOutput = AnySigner.sign(input: signingInput, coin: .solana)

        if !output.errorMessage.isEmpty {
            throw AnyError(output.errorMessage)
        }
        return output.encoded
    }

    func signSwap(input: SignerInput, privateKey: Data) throws -> [String] {
        try ChainSigner(chain: .solana).signSwap(input: input, privateKey: privateKey)
    }

    func signStake(input: SignerInput, privateKey: Data) throws -> [String] {
        guard case .stake(_, let type) = input.type else {
            throw AnyError("invalid type")
        }
        let transactionType: SolanaSigningInput.OneOf_TransactionType
        switch type {
        case .stake(let validator):
            transactionType = .delegateStakeTransaction(.with {
                $0.validatorPubkey = validator.id
                $0.value = input.value.asUInt
            })
            let encoded = try sign(input: input, type: transactionType, coinType: input.coinType, privateKey: privateKey)
            let memo = input.memo ?? ""
            let instruction = try SolanaInstruction(
                programId: "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
                accounts: [
                    SolanaAccountMeta(pubkey: input.senderAddress, isSigner: true, isWritable: true),
                ],
                data: Base58.encodeNoCheck(data: memo.encodedData())
            )
            let data = try JSONEncoder().encode(instruction)
            let instructionJson = try data.encodeString()
            guard let transaction = SolanaTransaction.insertInstruction(encodedTx: encoded, insertAt: -1, instruction: instructionJson) else {
                throw AnyError("Unable to add instruction")
            }
            return try [
                signRawTransaction(transaction: transaction, privateKey: privateKey),
            ]
        case .unstake(let delegation):
            transactionType = .deactivateStakeTransaction(.with {
                $0.stakeAccount = delegation.base.delegationId
            })
        case .withdraw(let delegation):
            transactionType = .withdrawTransaction(.with {
                $0.stakeAccount = delegation.base.delegationId
                $0.value = delegation.base.balanceValue.asUInt
            })
        case .redelegate,
             .rewards:
            fatalError()
        case .freeze,
             .unfreeze:
            throw AnyError("Solana does not support freeze operations")
        }
        return try [
            sign(input: input, type: transactionType, coinType: input.coinType, privateKey: privateKey),
        ]
    }

    private func transcodeBase58ToBase64(_ string: String) throws -> String {
        return try Base58.decodeNoCheck(string: string)
            .base64EncodedString()
            .paddded
    }
}

extension String {
    var paddded: Self {
        let offset = count % 4
        guard offset != 0 else { return self }
        return padding(toLength: count + 4 - offset, withPad: "=", startingAt: 0)
    }
}
