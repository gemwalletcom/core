import KeystoreTestKit
import Primitives
import Testing
import WalletCore

@testable import Keystore

struct LocalKeystoreTests {
    let chains: [Chain] = [.ethereum, .solana]

    @Test
    func testCreateWallet() throws {
        let keystore = LocalKeystore.mock()
        let createdWords = try keystore.createWallet()
        #expect(createdWords.count == 12)
    }

    @Test
    func testImportWallet() async {
        await #expect(throws: Never.self) {
            let keystore = LocalKeystore.mock()
            let words = try keystore.createWallet()
            let wallet = try await keystore.importWallet(
                name: "test",
                type: .phrase(words: words, chains: [.ethereum]),
                isWalletsEmpty: true,
                source: .import
            )

            #expect(wallet.accounts.count == 1)
            #expect(wallet.accounts.first?.chain == .ethereum)
        }
    }

    @Test
    func importSolanaWallet() async {
        await #expect(throws: Never.self) {
            let keystore = LocalKeystore.mock()
            let wallet = try await keystore.importWallet(
                name: "Solana Wallet",
                type: .phrase(words: LocalKeystore.words, chains: [.solana]),
                isWalletsEmpty: true,
                source: .import
            )

            #expect(wallet.accounts.count == 1)
            #expect(wallet.accounts.first?.chain == .solana)
            #expect(wallet.accounts.first?.address == "57mwmnV2rFuVDmhiJEjonD7cfuFtcaP9QvYNGfDEWK71")
        }
    }

    @Test
    func importEthereumWallet() async {
        await #expect(throws: Never.self) {
            let keystore = LocalKeystore.mock()
            let chains: [Chain] = [.ethereum, .smartChain, .blast]

            let wallet = try await keystore.importWallet(
                name: "test",
                type: .phrase(words: LocalKeystore.words, chains: chains),
                isWalletsEmpty: true,
                source: .import
            )

            #expect(wallet.accounts == chains.map {
                Account(chain: $0,
                        address: "0x8f348F300873Fd5DA36950B2aC75a26584584feE",
                        derivationPath: "m/44'/60'/0'/0/0",
                        extendedPublicKey: "")
            })
        }
    }

    @Test
    func exportSolanaPrivateKey() async {
        await #expect(throws: Never.self) {
            let keystore = LocalKeystore.mock()
            let hex = "0xb9095df5360714a69bc86ca92f6191e60355f206909982a8409f7b8358cf41b0"
            let wallet = try await keystore.importWallet(
                name: "Test Solana",
                type: .privateKey(text: hex, chain: .solana),
                isWalletsEmpty: true,
                source: .import
            )

            let exported = try await keystore.getPrivateKeyEncoded(wallet: wallet, chain: .solana)
            #expect(exported == "DTJi5pMtSKZHdkLX4wxwvjGjf2xwXx1LSuuUZhugYWDV")

            let keystore2 = LocalKeystore.mock()
            let wallet2 = try await keystore2.importWallet(
                name: "Test Solana 2",
                type: .privateKey(text: exported, chain: .solana),
                isWalletsEmpty: true,
                source: .import
            )
            let exportedKey = try await keystore2.getPrivateKey(wallet: wallet2, chain: .solana)

            #expect(Base58.encodeNoCheck(data: exportedKey) == exported)
        }
    }

    @Test
    func exportEthereumPrivateKey() async {
        await #expect(throws: Never.self) {
            let keystore = LocalKeystore.mock()
            let hex = "0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e"
            let wallet = try await keystore.importWallet(
                name: "Test Ethereum",
                type: .privateKey(text: hex, chain: .ethereum),
                isWalletsEmpty: true,
                source: .import
            )

            let exported = try await keystore.getPrivateKeyEncoded(wallet: wallet, chain: .ethereum)
            #expect(exported == hex)
        }
    }

    @Test
    func signSolanaMessage() async throws {
        let keystore = LocalKeystore.mock()
        let wallet = try await keystore.importWallet(
            name: "Test Solana",
            type: .phrase(words: LocalKeystore.words, chains: [.solana]),
            isWalletsEmpty: true,
            source: .import
        )

        let text = "5A2EYggC6hiAAuRArnkAANGySDyqQUGrbBHXfKQD9DQ5XcSkReDswnRqb7x3KRrnie9qSL"
        let hash = Base58.decodeNoCheck(string: text)!
        let signature = try await keystore.sign(hash: hash, wallet: wallet, chain: .solana)
        let encoded = Base58.encodeNoCheck(data: signature)

        #expect(encoded == "5ZRaXVuDePowJjZmKaMjfcuqBVZet6e8QiCjTkGXBn7xhCvoEswUKXiGs2wmPxcqTfJUH28eCC91J1vLSjANNM9v")
    }


    @Test
    func deriveAddress() async {
        await #expect(throws: Never.self) {
            let keystore = LocalKeystore.mock()
            let chains = Chain.allCases
            let wallet = try await keystore.importWallet(
                name: "test",
                type: .phrase(words: LocalKeystore.words, chains: chains),
                isWalletsEmpty: true,
                source: .import
            )

            #expect(wallet.accounts.count == chains.count)

            for account in wallet.accounts {
                let chain = account.chain
                let derivedAddress = account.address
                let expected: String
                switch chain {
                case .bitcoin:
                    expected = "bc1quvuarfksewfeuevuc6tn0kfyptgjvwsvrprk9d"
                case .litecoin:
                    expected = "ltc1qhd8fxxp2dx3vsmpac43z6ev0kllm4n53t5sk0u"
                case .ethereum,
                     .smartChain,
                     .polygon,
                     .arbitrum,
                     .optimism,
                     .base,
                     .avalancheC,
                     .opBNB,
                     .fantom,
                     .gnosis,
                     .manta,
                     .blast,
                     .zkSync,
                     .linea,
                     .mantle,
                     .celo,
                     .world,
                     .sonic,
                     .abstract,
                     .berachain,
                     .ink,
                     .unichain,
                     .hyperliquid,
                     .monad,
                     .hyperCore,
                     .plasma,
                     .xLayer,
                     .stable:
                    expected = "0x8f348F300873Fd5DA36950B2aC75a26584584feE"
                case .solana:
                    expected = "57mwmnV2rFuVDmhiJEjonD7cfuFtcaP9QvYNGfDEWK71"
                case .thorchain:
                    expected = "thor1c8jd7ad9pcw4k3wkuqlkz4auv95mldr2kyhc65"
                case .cosmos:
                    expected = "cosmos142j9u5eaduzd7faumygud6ruhdwme98qsy2ekn"
                case .osmosis:
                    expected = "osmo142j9u5eaduzd7faumygud6ruhdwme98qclefqp"
                case .ton:
                    expected = "UQDgEMqToTacHic7SnvnPFmvceG5auFkCcAw0mSCvzvKUaT4"
                case .tron:
                    expected = "TQ5NMqJjhpQGK7YJbESKtNCo86PJ89ujio"
                case .doge:
                    expected = "DJRFZNg8jkUtjcpo2zJd92FUAzwRjitw6f"
                case .aptos:
                    expected = "0x7968dab936c1bad187c60ce4082f307d030d780e91e694ae03aef16aba73f30"
                case .sui:
                    expected = "0xada112cfb90b44ba889cc5d39ac2bf46281e4a91f7919c693bcd9b8323e81ed2"
                case .xrp:
                    expected = "rPwE3gChNKtZ1mhH3Ko8YFGqKmGRWLWXV3"
                case .celestia:
                    expected = "celestia142j9u5eaduzd7faumygud6ruhdwme98qpwmfv7"
                case .injective:
                    expected = "inj13u6g7vqgw074mgmf2ze2cadzvkz9snlwcrtq8a"
                case .sei:
                    expected = "sei142j9u5eaduzd7faumygud6ruhdwme98qagm0sj"
                case .noble:
                    expected = "noble142j9u5eaduzd7faumygud6ruhdwme98qc8l3wa"
                case .near:
                    expected = "0c91f6106ff835c0195d5388565a2d69e25038a7e23d26198f85caf6594117ec"
                case .stellar:
                    expected = "GA3H6I4C5XUBYGVB66KXR27JV5KS3APSTKRUWOIXZ5MVWZKVTLXWKZ2P"
                case .bitcoinCash:
                    expected = "qpzl3jxkzgvfd9flnd26leud5duv795fnv7vuaha70"
                case .algorand:
                    expected = "JTJWO524JXIHVPGBDWFLJE7XUIA32ECOZOBLF2QP3V5TQBT3NKZSCG67BQ"
                case .polkadot:
                    expected = "13nN6BGAoJwd7Nw1XxeBCx5YcBXuYnL94Mh7i3xBprqVSsFk"
                case .cardano:
                    expected = "addr1qyr8jjfnypp95eq74aqzn7ss687ehxclgj7mu6gratmg3mul2040vt35dypp042awzsjk5xm3zr3zm5qh7454uwdv08s84ray2"
                case .zcash:
                    expected = "t1YYnByMzdGhQv3W3rnjHMrJs6HH4Y231gy"
                }

                #expect(derivedAddress == expected, "\(chain) failed to match address")
            }
        }
    }

    @Test
    func setupChainsAddsMissingChains() async {
        await #expect(throws: Never.self) {
            let keystore = LocalKeystore.mock()
            let ethWallet = try await keystore.importWallet(
                name: "ETH only",
                type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
                isWalletsEmpty: true,
                source: .import
            )
            let solWallet = try await keystore.importWallet(
                name: "SOL only",
                type: .phrase(words: LocalKeystore.words, chains: [.solana]),
                isWalletsEmpty: false,
                source: .import
            )
            let updated = try keystore.setupChains(
                chains: chains,
                for: [ethWallet, solWallet]
            )

            #expect(updated.count == 2)

            for wallet in updated {
                #expect(wallet.accounts.map(\.chain).asSet() == chains.asSet())
            }
        }
    }

    @Test
    func setupChainsAddNoMissingChains() async {
        await #expect(throws: Never.self) {
            let keystore = LocalKeystore.mock()
            let wallet = try await keystore.importWallet(
                name: "Complete wallet",
                type: .phrase(words: LocalKeystore.words, chains: chains),
                isWalletsEmpty: true,
                source: .import
            )

            let result = try keystore.setupChains(
                chains: chains,
                for: [wallet]
            )

            #expect(result.isEmpty)
        }
    }

    @Test
    func passwordCreatedOnFirstImport() async throws {
        let mockPassword = MockKeystorePassword()
        let keystore = LocalKeystore.mock(keystorePassword: mockPassword)

        #expect(try mockPassword.getPassword().isEmpty)

        let _ = try await keystore.importWallet(
            name: "First Wallet",
            type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
            isWalletsEmpty: true,
            source: .import
        )

        #expect(try mockPassword.getPassword().count == 64)
    }

    @Test
    func concurrentImportAndDelete() async throws {
        let keystore = LocalKeystore.mock(keystorePassword: MockKeystorePassword(memoryPassword: "test-password"))

        let wallets = try await withThrowingTaskGroup(of: Primitives.Wallet.self) { group in
            for index in 0 ..< 5 {
                group.addTask {
                    try await keystore.importWallet(
                        name: "Wallet \(index)",
                        type: .phrase(words: LocalKeystore.words, chains: [.ethereum]),
                        isWalletsEmpty: false,
                        source: .import
                    )
                }
            }

            var wallets: [Primitives.Wallet] = []
            for try await wallet in group {
                wallets.append(wallet)
            }
            return wallets
        }

        let importedWallets = wallets
        #expect(importedWallets.count == 5)

        try await withThrowingTaskGroup(of: Void.self) { group in
            for wallet in importedWallets {
                group.addTask {
                    try await keystore.deleteKey(for: wallet)
                }
            }
            try await group.waitForAll()
        }
    }
}
