// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Foundation
import GRDB
import Primitives
import PrimitivesTestKit
import StoreTestKit
@testable import Store

private extension UserDefaults {
    static func mock() -> UserDefaults {
        let suiteName = UUID().uuidString
        let defaults = UserDefaults(suiteName: suiteName)!
        defaults.removePersistentDomain(forName: suiteName)
        return defaults
    }
}

@Suite(.serialized)
struct WalletIdMigrationTests {

    private let currentWalletKey = "currentWallet"

    @Test
    func migrateMulticoinWallet() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum, .bitcoin])
        let walletStore = WalletStore(db: db)

        let oldId = "uuid-multicoin-1"
        let ethAddress = "0x1234567890abcdef"
        let wallet = Wallet.mock(
            id: oldId,
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: ethAddress), .mock(chain: .bitcoin)]
        )
        try walletStore.addWallet(wallet)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == "multicoin_\(ethAddress)")
        #expect(wallets.first?.externalId == oldId)
    }

    @Test
    func migrateViewWallet() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let oldId = "uuid-view-1"
        let address = "0xviewaddress"
        let wallet = Wallet.mock(
            id: oldId,
            type: .view,
            accounts: [.mock(chain: .ethereum, address: address)]
        )
        try walletStore.addWallet(wallet)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == "view_ethereum_\(address)")
        #expect(wallets.first?.externalId == oldId)
    }

    @Test
    func migrateSingleWallet() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.bitcoin])
        let walletStore = WalletStore(db: db)

        let oldId = "uuid-single-1"
        let address = "bc1qsingleaddress"
        let wallet = Wallet.mock(
            id: oldId,
            type: .single,
            accounts: [.mock(chain: .bitcoin, address: address)]
        )
        try walletStore.addWallet(wallet)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == "single_bitcoin_\(address)")
        #expect(wallets.first?.externalId == oldId)
    }

    @Test
    func migratePrivateKeyWallet() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let oldId = "uuid-pk-1"
        let address = "0xprivatekey"
        let wallet = Wallet.mock(
            id: oldId,
            type: .privateKey,
            accounts: [.mock(chain: .ethereum, address: address)]
        )
        try walletStore.addWallet(wallet)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == "privateKey_ethereum_\(address)")
        #expect(wallets.first?.externalId == oldId)
    }

    @Test
    func removeDuplicateMulticoinWallets() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let ethAddress = "0xsameaddress"

        let wallet1 = Wallet.mock(
            id: "uuid-1",
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: ethAddress)],
            order: 0
        )
        let wallet2 = Wallet.mock(
            id: "uuid-2",
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: ethAddress)],
            order: 1
        )
        let wallet3 = Wallet.mock(
            id: "uuid-3",
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: ethAddress)],
            order: 2
        )

        try walletStore.addWallet(wallet1)
        try walletStore.addWallet(wallet2)
        try walletStore.addWallet(wallet3)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == "multicoin_\(ethAddress)")
        #expect(wallets.first?.externalId == "uuid-1")
    }

    @Test
    func mixedWalletTypes() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum, .bitcoin, .solana])
        let walletStore = WalletStore(db: db)

        let multicoin = Wallet.mock(
            id: "uuid-multicoin",
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: "0xmulti")],
            order: 0
        )
        let single = Wallet.mock(
            id: "uuid-single",
            type: .single,
            accounts: [.mock(chain: .bitcoin, address: "bc1single")],
            order: 1
        )
        let view = Wallet.mock(
            id: "uuid-view",
            type: .view,
            accounts: [.mock(chain: .solana, address: "solview")],
            order: 2
        )

        try walletStore.addWallet(multicoin)
        try walletStore.addWallet(single)
        try walletStore.addWallet(view)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 3)

        let ids = Set(wallets.map { $0.id })
        #expect(ids.contains("multicoin_0xmulti"))
        #expect(ids.contains("single_bitcoin_bc1single"))
        #expect(ids.contains("view_solana_solview"))
    }

    @Test
    func updateChildTableReferences() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)
        let balanceStore = BalanceStore(db: db)
        let assetStore = AssetStore(db: db)

        let oldId = "uuid-with-balances"
        let ethAddress = "0xwithbalances"
        let wallet = Wallet.mock(
            id: oldId,
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: ethAddress)]
        )
        try walletStore.addWallet(wallet)

        let asset = AssetBasic.mock(asset: .mock(id: .mockEthereum()))
        try assetStore.add(assets: [asset])
        try db.dbQueue.write { db in
            try db.execute(
                sql: "INSERT INTO balances (assetId, walletId, isEnabled) VALUES (?, ?, ?)",
                arguments: [asset.asset.id.identifier, oldId, true]
            )
        }

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let newWalletId = try WalletId.from(id: "multicoin_\(ethAddress)")
        let balances = try balanceStore.getBalances(walletId: newWalletId, assetIds: [asset.asset.id])
        #expect(balances.count == 1)
    }

    @Test
    func multipleDuplicateGroups() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let address1 = "0xaddress1"
        try walletStore.addWallet(.mock(id: "uuid-1a", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address1)], order: 0))
        try walletStore.addWallet(.mock(id: "uuid-1b", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address1)], order: 1))
        try walletStore.addWallet(.mock(id: "uuid-1c", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address1)], order: 2))

        let address2 = "0xaddress2"
        try walletStore.addWallet(.mock(id: "uuid-2a", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address2)], order: 3))
        try walletStore.addWallet(.mock(id: "uuid-2b", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address2)], order: 4))

        let address3 = "0xaddress3"
        try walletStore.addWallet(.mock(id: "uuid-3", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address3)], order: 5))

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 3)

        let ids = Set(wallets.map { $0.id })
        #expect(ids.contains("multicoin_\(address1)"))
        #expect(ids.contains("multicoin_\(address2)"))
        #expect(ids.contains("multicoin_\(address3)"))

        let wallet1 = wallets.first { $0.id == "multicoin_\(address1)" }
        #expect(wallet1?.externalId == "uuid-1a")

        let wallet2 = wallets.first { $0.id == "multicoin_\(address2)" }
        #expect(wallet2?.externalId == "uuid-2a")
    }

    @Test
    func duplicateViewWallets() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let address = "0xviewaddr"
        try walletStore.addWallet(.mock(id: "uuid-view-1", type: .view, accounts: [.mock(chain: .ethereum, address: address)]))
        try walletStore.addWallet(.mock(id: "uuid-view-2", type: .view, accounts: [.mock(chain: .ethereum, address: address)]))
        try walletStore.setOrder(walletId: "uuid-view-1", order: 1)
        try walletStore.setOrder(walletId: "uuid-view-2", order: 0)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == "view_ethereum_\(address)")
        #expect(wallets.first?.externalId == "uuid-view-2")
    }

    @Test
    func duplicateSingleWallets() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.bitcoin])
        let walletStore = WalletStore(db: db)

        let address = "bc1qsingle"
        try walletStore.addWallet(.mock(id: "uuid-single-1", type: .single, accounts: [.mock(chain: .bitcoin, address: address)]))
        try walletStore.addWallet(.mock(id: "uuid-single-2", type: .single, accounts: [.mock(chain: .bitcoin, address: address)]))
        try walletStore.addWallet(.mock(id: "uuid-single-3", type: .single, accounts: [.mock(chain: .bitcoin, address: address)]))
        try walletStore.setOrder(walletId: "uuid-single-1", order: 2)
        try walletStore.setOrder(walletId: "uuid-single-2", order: 0)
        try walletStore.setOrder(walletId: "uuid-single-3", order: 1)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == "single_bitcoin_\(address)")
        #expect(wallets.first?.externalId == "uuid-single-2")
    }

    @Test
    func keepWalletWithLowestOrder() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let address = "0xordertest"
        try walletStore.addWallet(.mock(id: "uuid-order-5", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address)]))
        try walletStore.addWallet(.mock(id: "uuid-order-2", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address)]))
        try walletStore.addWallet(.mock(id: "uuid-order-8", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address)]))
        try walletStore.addWallet(.mock(id: "uuid-order-1", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address)]))
        try walletStore.addWallet(.mock(id: "uuid-order-3", type: .multicoin, accounts: [.mock(chain: .ethereum, address: address)]))
        try walletStore.setOrder(walletId: "uuid-order-5", order: 5)
        try walletStore.setOrder(walletId: "uuid-order-2", order: 2)
        try walletStore.setOrder(walletId: "uuid-order-8", order: 8)
        try walletStore.setOrder(walletId: "uuid-order-1", order: 1)
        try walletStore.setOrder(walletId: "uuid-order-3", order: 3)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.externalId == "uuid-order-1")
    }

    @Test
    func accountsUpdatedAfterMigration() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum, .bitcoin])
        let walletStore = WalletStore(db: db)

        let oldId = "uuid-accounts"
        let ethAddress = "0xaccounts"
        let btcAddress = "bc1accounts"
        let wallet = Wallet.mock(
            id: oldId,
            type: .multicoin,
            accounts: [
                .mock(chain: .ethereum, address: ethAddress),
                .mock(chain: .bitcoin, address: btcAddress)
            ]
        )
        try walletStore.addWallet(wallet)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.accounts.count == 2)

        let chains = Set(wallets.first?.accounts.map { $0.chain } ?? [])
        #expect(chains.contains(.ethereum))
        #expect(chains.contains(.bitcoin))
    }

    @Test
    func walletAlreadyInNewFormat() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let ethAddress = "0xalreadymigrated"
        let newFormatId = "multicoin_\(ethAddress)"
        let wallet = Wallet.mock(
            id: newFormatId,
            externalId: "old-uuid",
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: ethAddress)]
        )
        try walletStore.addWallet(wallet)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == newFormatId)
        #expect(wallets.first?.externalId == "old-uuid")
    }

    @Test
    func emptyDatabase() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mock()

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let walletStore = WalletStore(db: db)
        let wallets = try walletStore.getWallets()
        #expect(wallets.isEmpty)
    }

    @Test
    func multipleAccountsConsistentSelection() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum, .bitcoin])
        let walletStore = WalletStore(db: db)

        let oldId = "uuid-multi-accounts"
        let btcAddress = "bc1bitcoin"
        let ethAddress = "0xethereum"
        let wallet = Wallet.mock(
            id: oldId,
            type: .single,
            accounts: [
                .mock(chain: .bitcoin, address: btcAddress),
                .mock(chain: .ethereum, address: ethAddress)
            ]
        )
        try walletStore.addWallet(wallet)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let wallets = try walletStore.getWallets()
        #expect(wallets.count == 1)
        #expect(wallets.first?.id == "single_ethereum_\(ethAddress)")
    }
}

@Suite(.serialized)
struct WalletIdMigrationPreferenceTests {

    private let currentWalletKey = "currentWallet"

    @Test
    func migrateCurrentWalletPreference() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let oldId = "uuid-current"
        let ethAddress = "0xcurrent"
        let wallet = Wallet.mock(
            id: oldId,
            type: .multicoin,
            accounts: [.mock(chain: .ethereum, address: ethAddress)]
        )
        try walletStore.addWallet(wallet)

        userDefaults.set(oldId, forKey: currentWalletKey)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let newCurrentWalletId = userDefaults.string(forKey: currentWalletKey)
        #expect(newCurrentWalletId == "multicoin_\(ethAddress)")
    }

    @Test
    func setCurrentWalletWhenNoneSet() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        try walletStore.addWallet(.mock(id: "uuid-first", type: .multicoin, accounts: [.mock(chain: .ethereum, address: "0xfirst")]))
        try walletStore.setOrder(walletId: "uuid-first", order: 0)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let currentWalletId = userDefaults.string(forKey: currentWalletKey)
        #expect(currentWalletId == "multicoin_0xfirst")
    }

    @Test
    func fallbackCurrentWalletWhenInvalid() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        userDefaults.set("deleted-wallet-id", forKey: currentWalletKey)

        try walletStore.addWallet(.mock(id: "uuid-fallback", type: .multicoin, accounts: [.mock(chain: .ethereum, address: "0xfallback")]))
        try walletStore.setOrder(walletId: "uuid-fallback", order: 0)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let currentWalletId = userDefaults.string(forKey: currentWalletKey)
        #expect(currentWalletId == "multicoin_0xfallback")
    }

    @Test
    func preserveCurrentWalletWhenAlreadyMigrated() throws {
        let userDefaults = UserDefaults.mock()
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)

        let ethAddress = "0xalready"
        let newFormatId = "multicoin_\(ethAddress)"
        try walletStore.addWallet(.mock(id: newFormatId, type: .multicoin, accounts: [.mock(chain: .ethereum, address: ethAddress)], order: 1))
        try walletStore.addWallet(.mock(id: "uuid-other", type: .multicoin, accounts: [.mock(chain: .ethereum, address: "0xother")], order: 0))

        userDefaults.set(newFormatId, forKey: currentWalletKey)

        try db.dbQueue.write { db in
            try WalletIdMigration.migrate(db: db, userDefaults: userDefaults)
        }

        let currentWalletId = userDefaults.string(forKey: currentWalletKey)
        #expect(currentWalletId == newFormatId)
    }
}
