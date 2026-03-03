// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemSwapperProtocol
import Primitives
import Store
import ChainService
import StakeService
import EarnService
import BalanceService
import NFTService

public struct TransactionStateService: Sendable {
    private let transactionStore: TransactionStore
    private let stateService: TransactionStateProvider
    private let swapResultProvider: SwapResultProvider
    private let postProcessingService: TransactionPostProcessingService
    private let runner: JobRunner = JobRunner()

    public init(
        transactionStore: TransactionStore,
        swapper: any GemSwapperProtocol,
        stakeService: StakeService,
        earnService: EarnService,
        nftService: NFTService,
        chainServiceFactory: any ChainServiceFactorable,
        balanceUpdater: any BalanceUpdater
    ) {
        self.transactionStore = transactionStore
        self.stateService = TransactionStateProvider(
            transactionStore: transactionStore,
            chainServiceFactory: chainServiceFactory
        )
        self.swapResultProvider = SwapResultProvider(swapper: swapper)
        self.postProcessingService = TransactionPostProcessingService(
            transactionStore: transactionStore,
            balanceUpdater: balanceUpdater,
            stakeService: stakeService,
            earnService: earnService,
            nftService: nftService
        )
    }

    public func setup() {
        if let walletsTransactions = try? transactionStore.getTransactionWallets(states: [.pending, .inTransit]) {
            runUpdate(for: walletsTransactions)
        }
    }

    public func addTransactions(wallet: Wallet, transactions: [Transaction]) throws {
        try transactionStore.addTransactions(
            walletId: wallet.walletId,
            transactions: transactions
        )
        runUpdate(for: transactions.map({ TransactionWallet(transaction: $0, wallet: wallet) }))
    }
}

// MARK: - Private

extension TransactionStateService {
    private func runUpdate(for transactionWallets: [TransactionWallet]) {
        let jobs = transactionWallets.map {
            TransactionStateUpdateJob(
                transactionWallet: $0,
                stateService: stateService,
                swapResultProvider: swapResultProvider,
                postProcessingService: postProcessingService
            )
        }
        Task {
            for job in jobs {
                await runner.addJob(job: job)
            }
        }
    }
}
