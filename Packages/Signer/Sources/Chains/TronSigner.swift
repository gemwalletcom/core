// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import WalletCore

internal import BigInt

struct TronSigner: Signable {
    func sign(
        input: SignerInput,
        contract: WalletCore.TronTransaction.OneOf_ContractOneof,
        feeLimit: Int?,
        memo: String? = .none,
        privateKey: Data
    ) throws -> String {
        guard case .tron(
            let blockNumber,
            let blockVersion,
            let blockTimestamp,
            let transactionTreeRoot,
            let parentHash,
            let witnessAddress,
            _
        ) = input.metadata
        else {
            throw AnyError("Missing tron metadata")
        }
        let signingInput = try TronSigningInput.with {
            $0.transaction = try TronTransaction.with {
                $0.contractOneof = contract
                $0.timestamp = Int64(blockTimestamp)
                $0.blockHeader = try TronBlockHeader.with {
                    $0.timestamp = Int64(blockTimestamp)
                    $0.number = Int64(blockNumber)
                    $0.version = Int32(blockVersion)
                    $0.txTrieRoot = try Data.from(hex: transactionTreeRoot)
                    $0.parentHash = try Data.from(hex: parentHash)
                    $0.witnessAddress = try Data.from(hex: witnessAddress)
                }
                if let feeLimit = feeLimit {
                    $0.feeLimit = Int64(feeLimit)
                }
                $0.expiration = Int64(blockTimestamp) + 10 * 60 * 60 * 1000
                if let memo = memo {
                    $0.memo = memo
                }
            }
            $0.privateKey = privateKey
        }
        let output: TronSigningOutput = AnySigner.sign(input: signingInput, coin: input.coinType)

        if !output.errorMessage.isEmpty {
            throw AnyError(output.errorMessage)
        }

        return output.json
    }

    private func createVoteWitnessContract(input: SignerInput, votes: [TronVote]) -> TronVoteWitnessContract {
        TronVoteWitnessContract.with {
            $0.ownerAddress = input.senderAddress
            $0.support = true
            $0.votes = votes.map { vote in
                TronVoteWitnessContract.Vote.with {
                    $0.voteAddress = vote.validator
                    $0.voteCount = Int64(vote.count)
                }
            }
        }
    }

    func signTransfer(input: SignerInput, privateKey: Data) throws -> String {
        let contract = TronTransferContract.with {
            $0.ownerAddress = input.senderAddress
            $0.toAddress = input.destinationAddress
            $0.amount = input.value.asInt64
        }
        return try sign(input: input, contract: .transfer(contract), feeLimit: .none, memo: input.memo, privateKey: privateKey)
    }

    func signTokenTransfer(input: SignerInput, privateKey: Data) throws -> String {
        let contract = try TronTransferTRC20Contract.with {
            $0.contractAddress = try input.asset.getTokenId()
            $0.ownerAddress = input.senderAddress
            $0.toAddress = input.destinationAddress
            $0.amount = input.value.magnitude.serialize()
        }
        return try sign(
            input: input,
            contract: .transferTrc20Contract(contract),
            feeLimit: input.fee.gasLimit.asInt,
            memo: input.memo,
            privateKey: privateKey
        )
    }

    func signData(input: SignerInput, privateKey: Data) throws -> String {
        try ChainSigner(chain: input.asset.chain)
            .signData(input: input, privateKey: privateKey)
    }

    func signStake(input: SignerInput, privateKey: Data) throws -> [String] {
        guard case .stake(_, let stakeType) = input.type else {
            throw AnyError("Invalid input type for staking")
        }
        guard case .tron(_, _, _, _, _, _, let stakeData) = input.metadata else {
            throw AnyError("Missing tron metadata")
        }

        let contract: WalletCore.TronTransaction.OneOf_ContractOneof
        switch stakeType {
        case .stake, .redelegate:
            guard case .votes(let votes) = stakeData else {
                throw AnyError("Expected votes for stake/redelegate")
            }
            contract = .voteWitness(createVoteWitnessContract(input: input, votes: votes))
        case .unstake:
            switch stakeData {
            case .unfreeze(let amounts):
                return try amounts.map { unfreeze in
                    try sign(
                        input: input,
                        contract: .unfreezeBalanceV2(
                            TronUnfreezeBalanceV2Contract.with {
                                $0.ownerAddress = input.senderAddress
                                $0.unfreezeBalance = Int64(unfreeze.amount)
                                $0.resource = unfreeze.resource.key
                            }
                        ),
                        feeLimit: .none,
                        privateKey: privateKey
                    )
                }
            case .votes(let votes):
                contract = .voteWitness(createVoteWitnessContract(input: input, votes: votes))
            }
        case .rewards:
            contract = .withdrawBalance(
                TronWithdrawBalanceContract.with {
                    $0.ownerAddress = input.senderAddress
                }
            )
        case .withdraw:
            contract = .withdrawExpireUnfreeze(
                TronWithdrawExpireUnfreezeContract.with {
                    $0.ownerAddress = input.senderAddress
                }
            )
        case .freeze(let resource):
            contract = .freezeBalanceV2(
                TronFreezeBalanceV2Contract.with {
                    $0.ownerAddress = input.senderAddress
                    $0.frozenBalance = input.value.asInt64
                    $0.resource = resource.key
                }
            )
        case .unfreeze(let resource):
            contract = .unfreezeBalanceV2(
                TronUnfreezeBalanceV2Contract.with {
                    $0.ownerAddress = input.senderAddress
                    $0.unfreezeBalance = input.value.asInt64
                    $0.resource = resource.key
                }
            )
        }
        return try [
            sign(input: input, contract: contract, feeLimit: .none, privateKey: privateKey)
        ]
    }
}
