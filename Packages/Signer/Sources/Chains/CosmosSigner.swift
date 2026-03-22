// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import WalletCore
import Primitives

// https://github.com/trustwallet/wallet-core/blob/master/swift/Tests/Blockchains/THORChainTests.swift#L27
struct CosmosSigner: Signable {

    func signTransfer(input: SignerInput, privateKey: Data) throws -> String {
        let chain = try CosmosChain.from(string: input.asset.chain.rawValue)
        let message = getTransferMessage(input: input, denom: chain.denom.rawValue)
        return try sign(input: input, messages: [message], chain: chain, memo: input.memo, privateKey: privateKey)
    }
    
    func signTokenTransfer(input: SignerInput, privateKey: Data) throws -> String {
        let chain = try CosmosChain.from(string: input.asset.chain.rawValue)
        let denom = try input.asset.getTokenId()
        let message = getTransferMessage(input: input, denom: denom)
        return try sign(input: input, messages: [message], chain: chain, memo: input.memo, privateKey: privateKey)
    }
    
    private func sign(input: SignerInput, messages: [CosmosMessage], chain: CosmosChain, memo: String?, privateKey: Data) throws -> String {
        let fee = switch chain {
        case .cosmos,
                .osmosis,
                .celestia,
                .injective,
                .sei,
                .noble:
            CosmosFee.with {
                $0.gas = UInt64(messages.count * input.fee.gasLimit.asInt)
                $0.amounts = [CosmosAmount.with {
                    $0.amount = input.fee.fee.description
                    $0.denom = chain.denom.rawValue
                }]
            }
        case .thorchain:
            CosmosFee.with {
                $0.gas = UInt64(messages.count * input.fee.gasLimit.asInt)
            }
        }

        let signerInput = try CosmosSigningInput.with {
            $0.mode = .sync
            $0.accountNumber = UInt64(try input.metadata.getAccountNumber())
            $0.chainID = try input.metadata.getChainId()
            $0.memo = memo.valueOrEmpty
            $0.sequence = try input.metadata.getSequence()
            $0.messages = messages
            $0.fee = fee
            $0.privateKey = privateKey
            $0.signingMode = .protobuf
        }
        
        let output: CosmosSigningOutput = AnySigner.sign(input: signerInput, coin: input.coinType)
        
        if !output.errorMessage.isEmpty {
            throw AnyError(output.errorMessage)
        }
        
        return output.serialized
    }
    
    func signSwap(input: SignerInput, privateKey: Data) throws -> [String] {
        try ChainSigner(chain: input.asset.chain).signSwap(input: input, privateKey: privateKey)
    }

    func signData(input: Primitives.SignerInput, privateKey: Data) throws -> String {
        fatalError()
    }
    
    func signStake(input: SignerInput, privateKey: Data) throws -> [String] {
        guard case .stake(_, let type) = input.type else {
            throw AnyError("invalid type")
        }
        let messages: [CosmosMessage]
        let chain = try CosmosChain.from(string: input.asset.chain.rawValue)
        let denom = chain.denom.rawValue
        switch type {
        case .stake(let validator):
            let amount = getAmount(input: input, denom: denom)
            messages = [
                getStakeMessage(delegatorAddress: input.senderAddress, validatorAddress: validator.id, amount: amount)
            ]
        case .unstake(let delegation):
            let amount = getAmount(input: input, denom: denom)
            messages = getRewardMessage(delegatorAddress: input.senderAddress, validators: [delegation.validator]) + [
                getUnstakeMessage(delegatorAddress: input.senderAddress, validatorAddress: delegation.validator.id, amount: amount)
            ]
        case .redelegate(let data):
            let amount = getAmount(input: input, denom: denom)
            messages = getRewardMessage(delegatorAddress: input.senderAddress, validators: [data.delegation.validator]) + [
                getRedelegateMessage(
                    delegatorAddress: input.senderAddress,
                    validatorSourceAddress: data.delegation.validator.id,
                    validatorDestinationAddress: data.toValidator.id,
                    amount: amount
                )
            ]
        case .rewards(let validators):
            messages = getRewardMessage(delegatorAddress: input.senderAddress, validators: validators)
        case .withdraw:
            fatalError()
        case .freeze, .unfreeze:
            throw AnyError("Cosmos does not support freeze operations")
        }
        
        return [
            try sign(input: input, messages: messages, chain: chain, memo: input.memo, privateKey: privateKey),
        ]
    }
    
    func getUnstakeMessage(delegatorAddress: String, validatorAddress: String, amount: CosmosAmount) -> CosmosMessage {
        .with {
            $0.unstakeMessage = .with {
                $0.amount = amount
                $0.delegatorAddress = delegatorAddress
                $0.validatorAddress = validatorAddress
            }
        }
    }
    
    func getRedelegateMessage(delegatorAddress: String, validatorSourceAddress: String, validatorDestinationAddress: String, amount: CosmosAmount) -> CosmosMessage {
        .with {
            $0.restakeMessage = .with {
                $0.amount = amount
                $0.delegatorAddress = delegatorAddress
                $0.validatorSrcAddress = validatorSourceAddress
                $0.validatorDstAddress = validatorDestinationAddress
            }
        }
    }
    
    func getStakeMessage(delegatorAddress: String, validatorAddress: String, amount: CosmosAmount) -> CosmosMessage {
        .with {
            $0.stakeMessage = .with {
                $0.amount = amount
                $0.delegatorAddress = delegatorAddress
                $0.validatorAddress = validatorAddress
            }
        }
    }
    
    func getRewardMessage(delegatorAddress: String, validators: [DelegationValidator]) -> [CosmosMessage] {
        return validators.map { validator in
            .with {
                $0.withdrawStakeRewardMessage = .with {
                    $0.delegatorAddress = delegatorAddress
                    $0.validatorAddress = validator.id
                }
            }
        }
    }
    
    func getAmount(input: SignerInput, denom: String) -> CosmosAmount {
        return CosmosAmount.with {
            $0.amount = input.value.description
            $0.denom = denom
        }
    }
    
    func getTransferMessage(input: SignerInput, denom: String) -> CosmosMessage {
        let amounts = [getAmount(input: input, denom: denom)]
        
        return CosmosMessage.with {
            $0.sendCoinsMessage = CosmosMessage.Send.with {
                $0.fromAddress = input.senderAddress
                $0.toAddress = input.destinationAddress
                $0.amounts = amounts
            }
        }
    }
}
