// Copyright (c). Gem Wallet. All rights reserved.

import Observation
import Testing
import Primitives
import Keystore
import KeystoreTestKit
import Store
import StoreTestKit
import Preferences
import PreferencesTestKit
import WalletServiceTestKit
@testable import WalletService

struct WalletServiceTests {

    @Test
    func importSecretPhraseDuplicateSameChain() async throws {
        let service = WalletService.mock(walletStore: .mock(db: .mockWithChains([.ethereum, .aptos])))

        let wallet1 = try await service.loadOrCreateWallet(
            name: "First Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            source: .import
        )
        try await service.setCurrent(wallet: wallet1)

        let wallet2 = try await service.loadOrCreateWallet(
            name: "Second Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum, .aptos]),
            source: .import
        )
        try await service.setCurrent(wallet: wallet2)

        #expect(wallet1.id == wallet2.id)
        #expect(wallet1.type == WalletType.multicoin)
        #expect(wallet2.type == WalletType.multicoin)
    }

    @Test
    func importSecretPhraseNoDuplicateDifferentWords() async throws {
        let service = WalletService.mock(walletStore: .mock(db: .mockWithChains([.ethereum])))

        let wallet1 = try await service.loadOrCreateWallet(
            name: "First Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            source: .import
        )
        try await service.setCurrent(wallet: wallet1)

        let differentWords = try service.createWallet()
        let wallet2 = try await service.loadOrCreateWallet(
            name: "Second Wallet",
            type: .phrase(words: differentWords, chains: [.ethereum]),
            source: .import
        )
        try await service.setCurrent(wallet: wallet2)

        #expect(wallet1.id != wallet2.id)
    }

    @Test
    func importSingleDuplicateSameChain() async throws {
        let service = WalletService.mock(walletStore: .mock(db: .mockWithChains([.bitcoin])))

        let wallet1 = try await service.loadOrCreateWallet(
            name: "First Single",
            type: .single(words: LocalKeystore.words, chain: .bitcoin),
            source: .import
        )
        try await service.setCurrent(wallet: wallet1)

        let wallet2 = try await service.loadOrCreateWallet(
            name: "Second Single",
            type: .single(words: LocalKeystore.words, chain: .bitcoin),
            source: .import
        )
        try await service.setCurrent(wallet: wallet2)

        #expect(wallet1.id == wallet2.id)
        #expect(wallet1.type == WalletType.single)
        #expect(wallet2.type == WalletType.single)
    }

    @Test
    func importSingleNoDuplicateDifferentChain() async throws {
        let service = WalletService.mock(walletStore: .mock(db: .mockWithChains([.bitcoin, .litecoin])))

        let wallet1 = try await service.loadOrCreateWallet(
            name: "BTC Single",
            type: .single(words: LocalKeystore.words, chain: .bitcoin),
            source: .import
        )
        try await service.setCurrent(wallet: wallet1)

        let wallet2 = try await service.loadOrCreateWallet(
            name: "LTC Single",
            type: .single(words: LocalKeystore.words, chain: .litecoin),
            source: .import
        )
        try await service.setCurrent(wallet: wallet2)

        #expect(wallet1.id != wallet2.id)
    }

    @Test
    func importPrivateKeyDuplicateSameChain() async throws {
        let service = WalletService.mock(walletStore: .mock(db: .mockWithChains([.ethereum])))

        let wallet1 = try await service.loadOrCreateWallet(
            name: "First Wallet",
            type: .privateKey(text: LocalKeystore.privateKey, chain: .ethereum),
            source: .import
        )
        try await service.setCurrent(wallet: wallet1)

        let wallet2 = try await service.loadOrCreateWallet(
            name: "Second Wallet",
            type: .privateKey(text: LocalKeystore.privateKey, chain: .ethereum),
            source: .import
        )
        try await service.setCurrent(wallet: wallet2)

        #expect(wallet1.id == wallet2.id)
        #expect(wallet1.type == WalletType.privateKey)
        #expect(wallet2.type == WalletType.privateKey)
    }

    @Test
    func importPrivateKeyNoDuplicateDifferentChain() async throws {
        let service = WalletService.mock(walletStore: .mock(db: .mockWithChains([.ethereum, .smartChain])))

        let wallet1 = try await service.loadOrCreateWallet(
            name: "ETH Wallet",
            type: .privateKey(text: LocalKeystore.privateKey, chain: .ethereum),
            source: .import
        )
        try await service.setCurrent(wallet: wallet1)

        let wallet2 = try await service.loadOrCreateWallet(
            name: "BSC Wallet",
            type: .privateKey(text: LocalKeystore.privateKey, chain: .smartChain),
            source: .import
        )
        try await service.setCurrent(wallet: wallet2)

        #expect(wallet1.id != wallet2.id)
    }

    @Test
    func importViewOnlyDuplicateSameChain() async throws {
        let service = WalletService.mock(walletStore: .mock(db: .mockWithChains([.ethereum])))

        let wallet1 = try await service.loadOrCreateWallet(
            name: "First View",
            type: .address(address: LocalKeystore.address, chain: .ethereum),
            source: .import
        )
        try await service.setCurrent(wallet: wallet1)

        let wallet2 = try await service.loadOrCreateWallet(
            name: "Second View",
            type: .address(address: LocalKeystore.address, chain: .ethereum),
            source: .import
        )
        try await service.setCurrent(wallet: wallet2)

        #expect(wallet1.id == wallet2.id)
        #expect(wallet1.type == WalletType.view)
        #expect(wallet2.type == WalletType.view)
    }

    @Test
    func importViewOnlyNoDuplicateDifferentChain() async throws {
        let service = WalletService.mock(walletStore: .mock(db: .mockWithChains([.ethereum, .polygon])))

        let wallet1 = try await service.loadOrCreateWallet(
            name: "ETH View",
            type: .address(address: LocalKeystore.address, chain: .ethereum),
            source: .import
        )
        try await service.setCurrent(wallet: wallet1)

        let wallet2 = try await service.loadOrCreateWallet(
            name: "Polygon View",
            type: .address(address: LocalKeystore.address, chain: .polygon),
            source: .import
        )
        try await service.setCurrent(wallet: wallet2)

        #expect(wallet1.id != wallet2.id)
    }

    @Test
    func importTypeMatchingExact() async throws {
        let service = WalletService.mock(walletStore: .mock(db: .mockWithChains([.ethereum, .aptos])))

        let mnemonicWallet = try await service.loadOrCreateWallet(
            name: "Mnemonic",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum, .aptos]),
            source: .import
        )
        try await service.setCurrent(wallet: mnemonicWallet)

        let privateKeyWallet = try await service.loadOrCreateWallet(
            name: "Private Key",
            type: .privateKey(text: LocalKeystore.privateKey, chain: .ethereum),
            source: .import
        )
        try await service.setCurrent(wallet: privateKeyWallet)

        #expect(mnemonicWallet.id != privateKeyWallet.id)
        #expect(mnemonicWallet.type == WalletType.multicoin)
        #expect(privateKeyWallet.type == WalletType.privateKey)
    }

    @Test
    func deleteLastWalletNotifiesObservers() async throws {
        let preferences = ObservablePreferences.mock()
        let service = WalletService.mock(walletStore: .mock(db: .mockWithChains([.ethereum])), preferences: preferences)

        let wallet = try await service.loadOrCreateWallet(
            name: "Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            source: .import
        )
        try await service.setCurrent(wallet: wallet)

        try await confirmation { confirm in
            withObservationTracking {
                _ = preferences.currentWalletId
            } onChange: {
                confirm()
            }
            try await service.delete(wallet)
        }
    }

    @Test
    func loadOrCreateWalletMarksSubscriptionsDirty() async throws {
        let rawPreferences = Preferences.mock()
        rawPreferences.subscriptionsVersion = 4
        rawPreferences.subscriptionsVersionHasChange = false

        let service = WalletService.mock(
            walletStore: .mock(db: .mockWithChains([.ethereum])),
            preferences: .mock(preferences: rawPreferences)
        )

        _ = try await service.loadOrCreateWallet(
            name: "Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            source: .import
        )

        #expect(rawPreferences.subscriptionsVersion == 5)
        #expect(rawPreferences.subscriptionsVersionHasChange)
    }

    @Test
    func deleteWalletMarksSubscriptionsDirty() async throws {
        let rawPreferences = Preferences.mock()
        let service = WalletService.mock(
            walletStore: .mock(db: .mockWithChains([.ethereum])),
            preferences: .mock(preferences: rawPreferences)
        )

        let wallet = try await service.loadOrCreateWallet(
            name: "Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            source: .import
        )
        _ = try await service.loadOrCreateWallet(
            name: "Second Wallet",
            type: .phrase(words: try service.createWallet(), chains: [.ethereum]),
            source: .import
        )

        rawPreferences.subscriptionsVersion = 7
        rawPreferences.subscriptionsVersionHasChange = false

        try await service.delete(wallet)

        #expect(rawPreferences.subscriptionsVersion == 8)
        #expect(rawPreferences.subscriptionsVersionHasChange)
    }

    @Test
    func deleteLastWalletMarksSubscriptionsDirty() async throws {
        let rawPreferences = Preferences.mock()
        let service = WalletService.mock(
            walletStore: .mock(db: .mockWithChains([.ethereum])),
            preferences: .mock(preferences: rawPreferences)
        )

        let wallet = try await service.loadOrCreateWallet(
            name: "Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            source: .import
        )

        rawPreferences.subscriptionsVersion = 7
        rawPreferences.subscriptionsVersionHasChange = false

        try await service.delete(wallet)

        #expect(rawPreferences.subscriptionsVersion == 1)
        #expect(rawPreferences.subscriptionsVersionHasChange)
    }

    @Test
    func setupChainsMarksSubscriptionsDirty() async throws {
        let rawPreferences = Preferences.mock()
        let service = WalletService.mock(
            walletStore: .mock(db: .mockWithChains([.ethereum, .bitcoin])),
            preferences: .mock(preferences: rawPreferences)
        )

        _ = try await service.loadOrCreateWallet(
            name: "Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            source: .import
        )

        rawPreferences.subscriptionsVersion = 10
        rawPreferences.subscriptionsVersionHasChange = false

        try service.setup(chains: [.bitcoin])

        #expect(rawPreferences.subscriptionsVersion == 11)
        #expect(rawPreferences.subscriptionsVersionHasChange)
    }
}
