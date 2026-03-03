// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store
import BalanceService
import StakeService
import EarnService
import NFTService
import Primitives

struct TransactionPostProcessingService: Sendable {
    private let transactionStore: TransactionStore
    private let balanceUpdater: any BalanceUpdater
    private let stakeService: StakeService
    private let earnService: EarnService
    private let nftService: NFTService

    init(
        transactionStore: TransactionStore,
        balanceUpdater: any BalanceUpdater,
        stakeService: StakeService,
        earnService: EarnService,
        nftService: NFTService
    ) {
        self.transactionStore = transactionStore
        self.balanceUpdater = balanceUpdater
        self.stakeService = stakeService
        self.earnService = earnService
        self.nftService = nftService
    }

    func process(wallet: Wallet, transaction: Transaction) async throws {
        Task {
            await balanceUpdater.updateBalance(
                for: wallet,
                assetIds: (transaction.assetIds + [transaction.feeAssetId]).unique()
            )
        }

        switch transaction.type {
        case .stakeDelegate, .stakeUndelegate, .stakeRewards, .stakeRedelegate, .stakeWithdraw:
            for assetIdentifier in transaction.assetIds {
                Task {
                    try await stakeService.update(
                        walletId: wallet.walletId,
                        chain: assetIdentifier.chain,
                        address: transaction.from
                    )
                }
            }
        case .earnDeposit, .earnWithdraw:
            for assetIdentifier in transaction.assetIds {
                Task {
                    try await earnService.update(
                        walletId: wallet.walletId,
                        assetId: assetIdentifier,
                        address: transaction.from
                    )
                }
            }
        case .transferNFT:
            Task {
                // TODO: implement nftService.update when ready
            }
        default:
            break
        }
    }
}
