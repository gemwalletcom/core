// Copyright (c). Gem Wallet. All rights reserved.

import Testing
@testable import Transfer
import Primitives
import PrimitivesTestKit
import Blockchain
import BlockchainTestKit
import SignerTestKit
import BalanceServiceTestKit
import TransactionStateService
import TransactionStateServiceTestKit
import Store
import StoreTestKit
import ChainServiceTestKit

struct TransferExecutorTests {

    @Test
    func hyperCorePerpetualFiltersSetupTransactions() async throws {
        let db = DB.mockAssets(assets: [.mock(asset: .hypercoreUSDC())])
        let transactionStore = TransactionStore(db: db)
        let executor = TransferExecutor(
            signer: TransactionSignerMock(signedData: ["setup1", "setup2", "setup3", "actual_order"]),
            chainService: ChainServiceMock.mock(broadcastResponses: ["hash0", "hash1", "hash2", "hash3"]),
            assetsEnabler: .mock(),
            balanceService: .mock(),
            transactionStateService: .mock(transactionStore: transactionStore)
        )

        let input = TransferConfirmationInput(
            data: .mock(type: .perpetual(.hypercoreUSDC(), .open(.mock()))),
            wallet: .mock(accounts: [Account.mock(chain: .hyperCore)]),
            transactionData: .mock(),
            amount: .mock(),
            delegate: nil
        )
        try await executor.execute(input: input)

        let transactions = try transactionStore.getTransactions(state: .pending)
        #expect(transactions.count == 1)
        #expect(transactions.first?.id.hash == "hash3")
    }

    @Test
    func swapTransactions() async throws {
        let db = DB.mockAssets()
        let transactionStore = TransactionStore(db: db)
        let executor = TransferExecutor(
            signer: TransactionSignerMock(signedData: ["approval_tx", "swap_tx"]),
            chainService: ChainServiceMock.mock(broadcastResponses: ["hash0", "hash1"]),
            assetsEnabler: .mock(),
            balanceService: .mock(),
            transactionStateService: .mock(transactionStore: transactionStore)
        )

        let input = TransferConfirmationInput(
            data: .mock(type: .swap(.mockEthereum(), .mock(), .mock())),
            wallet: .mock(accounts: [.mock(chain: .ethereum), .mock(chain: .bitcoin)]),
            transactionData: .mock(),
            amount: .mock(),
            delegate: nil
        )
        try await executor.execute(input: input)

        let transactions = try transactionStore.getTransactions(state: .pending)
        #expect(transactions.count == 2)
        #expect(transactions.map(\.id.hash).sorted() == ["hash0", "hash1"])
    }

    @Test
    func hyperCoreTransferKeepsTransaction() async throws {
        let db = DB.mockAssets()
        let transactionStore = TransactionStore(db: db)
        let executor = TransferExecutor(
            signer: TransactionSignerMock(signedData: ["tx"]),
            chainService: ChainServiceMock.mock(broadcastResponses: ["hash"]),
            assetsEnabler: .mock(),
            balanceService: .mock(),
            transactionStateService: .mock(transactionStore: transactionStore)
        )

        let input = TransferConfirmationInput(
            data: .mock(type: .transfer(.mockEthereum())),
            wallet: .mock(accounts: [.mock(chain: .ethereum)]),
            transactionData: .mock(),
            amount: .mock(),
            delegate: nil
        )

        try await executor.execute(input: input)

        let transactions = try transactionStore.getTransactions(state: .pending)
        #expect(transactions.count == 1)
        #expect(transactions.first?.id.hash == "hash")
    }

    @Test
    func perpetualModifyDoesNotStoreTransaction() async throws {
        let db = DB.mockAssets(assets: [.mock(asset: .hypercoreUSDC())])
        let transactionStore = TransactionStore(db: db)
        let executor = TransferExecutor(
            signer: TransactionSignerMock(signedData: ["modify_tx"]),
            chainService: ChainServiceMock.mock(broadcastResponses: ["hash"]),
            assetsEnabler: .mock(),
            balanceService: .mock(),
            transactionStateService: .mock(transactionStore: transactionStore)
        )

        let input = TransferConfirmationInput(
            data: .mock(type: .perpetual(.hypercoreUSDC(), .mockModify())),
            wallet: .mock(accounts: [Account.mock(chain: .hyperCore)]),
            transactionData: .mock(),
            amount: .mock(),
            delegate: nil
        )

        try await executor.execute(input: input)

        let transactions = try transactionStore.getTransactions(state: .pending)
        #expect(transactions.isEmpty)
    }
}

extension ChainServiceMock {
    static func mock(broadcastResponses: [String]) -> ChainServiceMock {
        let mock = ChainServiceMock()
        mock.broadcastResponses = broadcastResponses
        return mock
    }
}
