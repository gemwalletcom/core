// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import StoreTestKit
import BalanceServiceTestKit
import PrimitivesTestKit
import Primitives

@testable import Store
@testable import WalletService

struct WalletSetupServiceTests {
    @Test
    func setupMulticoinWallet() throws {
        let (db, balanceStore, walletStore, service) = setupService()
        let wallet = Wallet.mock(id: "multicoin_0xtest", type: .multicoin, accounts: [.mock(chain: .cosmos), .mock(chain: .ethereum)])

        try addAsset(db: db, chain: .cosmos)
        try addAsset(db: db, chain: .ethereum)
        try walletStore.addWallet(wallet)
        try service.setup(wallet: wallet)

        let isEnabled = try balanceStore.getBalanceRecord(walletId: wallet.walletId, assetId: AssetId(chain: .cosmos))?.isEnabled

        #expect(isEnabled == false)
    }

    @Test
    func setupSingleWallet() throws {
        let (db, balanceStore, walletStore, service) = setupService()
        let wallet = Wallet.mock(id: "single_cosmos_0xtest", type: .single, accounts: [.mock(chain: .cosmos)])

        try addAsset(db: db, chain: .cosmos)
        try walletStore.addWallet(wallet)
        try service.setup(wallet: wallet)

        let isEnabled = try balanceStore.getBalanceRecord(walletId: wallet.walletId, assetId: AssetId(chain: .cosmos))?.isEnabled

        #expect(isEnabled == true)
    }

    private func setupService() -> (DB, BalanceStore, WalletStore, WalletSetupService) {
        let db = DB.mock()
        let balanceStore = BalanceStore.mock(db: db)
        let walletStore = WalletStore.mock(db: db)
        let service = WalletSetupService(balanceService: .mock(balanceStore: balanceStore))
        return (db, balanceStore, walletStore, service)
    }

    private func addAsset(db: DB, chain: Chain) throws {
        try db.dbQueue.write { db in
            try Asset.mock(id: AssetId(chain: chain)).record.insert(db)
        }
    }
}
