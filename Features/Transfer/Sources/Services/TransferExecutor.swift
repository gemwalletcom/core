// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import BalanceService
import Foundation
import Primitives
import Signer
import TransactionStateService

public protocol TransferExecutable: Sendable {
    func execute(input: TransferConfirmationInput) async throws
}

public struct TransferExecutor: TransferExecutable {
    private static let ignoredTransactionTypes: Set<TransactionType> = [.perpetualModifyPosition]
    private static let ignoredAssetChains: Set<Chain> = [.hyperCore]

    private let signer: any TransactionSigneable
    private let chainService: any ChainServiceable
    private let assetsEnabler: any AssetsEnabler
    private let balanceService: BalanceService
    private let transactionStateService: TransactionStateService

    public init(
        signer: any TransactionSigneable,
        chainService: any ChainServiceable,
        assetsEnabler: any AssetsEnabler,
        balanceService: BalanceService,
        transactionStateService: TransactionStateService
    ) {
        self.signer = signer
        self.chainService = chainService
        self.assetsEnabler = assetsEnabler
        self.balanceService = balanceService
        self.transactionStateService = transactionStateService
    }

    public func execute(input: TransferConfirmationInput) async throws {
        let signedData = try await sign(input: input)

        for (index, transactionData) in signedData.enumerated() {
            debugLog("TransferExecutor data \(transactionData)")

            switch input.data.type.outputAction {
            case .sign:
                input.delegate?(.success(transactionData))
            case .send:
                try await send(
                    input: input,
                    transactionData: transactionData,
                    transactionIndex: index,
                    totalTransactions: signedData.count
                )
            }
        }
    }
}

// MARK: - Private

extension TransferExecutor {
    private func send(
        input: TransferConfirmationInput,
        transactionData: String,
        transactionIndex: Int,
        totalTransactions: Int
    ) async throws {
        let hash = try await chainService.broadcast(data: transactionData, options: broadcastOptions(data: input.data))

        debugLog("TransferExecutor broadcast response hash \(hash)")

        input.delegate?(.success(hash))

        let transaction = try TransactionFactory.makePendingTransaction(
            wallet: input.wallet,
            transferData: input.data,
            transactionData: input.transactionData,
            amount: input.amount,
            hash: hash,
            transactionIndex: transactionIndex
        )
        let assetIds = assetIdsToEnable(for: transaction)
        let transactions = pendingTransactions(
            for: transaction,
            transferData: input.data,
            transactionIndex: transactionIndex,
            totalTransactions: totalTransactions
        )

        try transactionStateService.addTransactions(wallet: input.wallet, transactions: transactions)
        Task {
            do {
                try balanceService.addAssetsBalancesIfMissing(assetIds: assetIds, wallet: input.wallet, isEnabled: true)
                try await assetsEnabler.enableAssets(wallet: input.wallet, assetIds: assetIds, enabled: true)
            } catch {
                debugLog("TransferExecutor post-transfer asset update error: \(error)")
            }
        }

        if totalTransactions > 1, transactionIndex < totalTransactions - 1 {
            try await Task.sleep(for: transactionDelay(for: input.data.chain.type))
        }
    }

    private func sign(input: TransferConfirmationInput) async throws -> [String] {
        try await signer.sign(
            transfer: input.data,
            transactionData: input.transactionData,
            amount: input.amount,
            wallet: input.wallet
        )
    }

    private func pendingTransactions(
        for transaction: Transaction,
        transferData: TransferData,
        transactionIndex: Int,
        totalTransactions: Int
    ) -> [Transaction] {
        guard !Self.ignoredTransactionTypes.contains(transaction.type) else {
            return []
        }

        if case .perpetual = transferData.type,
           Self.ignoredAssetChains.contains(transaction.assetId.chain),
           transactionIndex < totalTransactions - 1 {
            return []
        }

        return [transaction]
    }

    private func assetIdsToEnable(for transaction: Transaction) -> [AssetId] {
        transaction.assetIds.filter { !Self.ignoredAssetChains.contains($0.chain) }
    }

    private func broadcastOptions(data: TransferData) -> BroadcastOptions {
        switch data.chain {
        case .solana:
            switch data.type {
            case .transfer, .deposit, .withdrawal, .transferNft, .stake, .account, .tokenApprove, .perpetual, .earn: BroadcastOptions(
                skipPreflight: false
            )
            case .swap, .generic: BroadcastOptions(skipPreflight: true)
            }
        default: BroadcastOptions(skipPreflight: false)
        }
    }

    private func transactionDelay(for type: ChainType) -> Duration {
        switch type {
        case .ethereum, .hyperCore: .milliseconds(0)
        case .tron: .milliseconds(500)
        default: .milliseconds(500)
        }
    }
}
