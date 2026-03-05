import Testing
import Primitives
import Foundation
import WalletConnectSign
import Store
import PrimitivesTestKit
import StoreTestKit
import PreferencesTestKit
import WalletSessionServiceTestKit
import WalletSessionService
import WalletConnectorService
import struct Gemstone.SignMessage

@testable import WalletConnector

struct WalletConnectorSignerTests {
    @Test
    func getWalletsRequiredChains() throws {
        let ethOnlyWallet = Wallet.mock(id: "multicoin_0x1", accounts: [.mock(chain: .ethereum)])
        let ethPolygonWallet = Wallet.mock(id: "multicoin_0x2", accounts: [.mock(chain: .ethereum), .mock(chain: .polygon)])
        let solanaWallet = Wallet.mock(id: "multicoin_0x3", accounts: [.mock(chain: .solana)])

        let signer = try WalletConnectorSigner.mock(wallets: [ethOnlyWallet, ethPolygonWallet, solanaWallet])

        let matchingWallets = try signer.getWallets(for: .requiredChains())
        let noMatchWallets = try signer.getWallets(for: .requiredChainsNoMatch())
        
        #expect(matchingWallets.count == 1)
        #expect(matchingWallets.first?.walletId == ethPolygonWallet.walletId)
        #expect(noMatchWallets.isEmpty)
    }

    @Test
    func getWalletsOptionalChains() throws {
        let regularWallet = Wallet.mock(id: "multicoin_0x1", accounts: [.mock(chain: .ethereum)])
        let viewOnlyWallet = Wallet.mock(id: "view_ethereum_0x2", type: .view, accounts: [.mock(chain: .ethereum)])
        let bitcoinWallet = Wallet.mock(id: "multicoin_0x3", accounts: [.mock(chain: .bitcoin)])
        
        let signer = try WalletConnectorSigner.mock(wallets: [regularWallet, viewOnlyWallet, bitcoinWallet])

        let matchingWallets = try signer.getWallets(for: .multiOptionalNamespaces())
        let emptyOptionalWallets = try signer.getWallets(for: .emptyOptionalChains())
        
        #expect(matchingWallets.count == 1)
        #expect(matchingWallets.first?.walletId == regularWallet.walletId)
        #expect(emptyOptionalWallets.count == 2)
    }

    @Test
    func getWalletsMultiOptionalNamespaces() throws {
        let solWallet = Wallet.mock(id: "multicoin_0x1", accounts: [.mock(chain: .solana)])
        let solEthWallet = Wallet.mock(id: "multicoin_0x2", accounts: [.mock(chain: .solana), .mock(chain: .ethereum)])
        let solEthBnbWallet = Wallet.mock(id: "multicoin_0x3", accounts: [.mock(chain: .solana), .mock(chain: .ethereum), .mock(chain: .smartChain)])
        
        let signer = try WalletConnectorSigner.mock(wallets: [solWallet, solEthWallet, solEthBnbWallet])

        let wallets = try signer.getWallets(for: .multiOptionalNamespaces())
        
        #expect(wallets.count == 2)
        #expect(wallets.contains(where: { $0.walletId == solEthWallet.walletId }))
        #expect(wallets.contains(where: { $0.walletId == solEthBnbWallet.walletId }))
    }

    @Test
    func getWalletsMixedRequiredOptional() throws {
        let ethOnlyWallet = Wallet.mock(id: "multicoin_0x1", accounts: [.mock(chain: .ethereum)])
        let ethPolygonWallet = Wallet.mock(id: "multicoin_0x2", accounts: [.mock(chain: .ethereum), .mock(chain: .polygon)])
        let ethPolygonSolanaWallet = Wallet.mock(id: "multicoin_0x3", accounts: [.mock(chain: .ethereum), .mock(chain: .polygon), .mock(chain: .solana)])
        
        let signer = try WalletConnectorSigner.mock(wallets: [ethOnlyWallet, ethPolygonWallet, ethPolygonSolanaWallet])

        let wallets = try signer.getWallets(for: .mixedRequiredOptional())
        
        #expect(wallets.count == 1)
        #expect(wallets.first?.walletId == ethPolygonSolanaWallet.walletId)
    }

    @Test
    func getWalletsNonEIP155Optional() throws {
        let ethWallet = Wallet.mock(id: "multicoin_0x1", accounts: [.mock(chain: .ethereum)])
        let bitcoinWallet = Wallet.mock(id: "multicoin_0x2", accounts: [.mock(chain: .bitcoin)])
        let cosmosWallet = Wallet.mock(id: "multicoin_0x3", accounts: [.mock(chain: .cosmos)])

        let signer = try WalletConnectorSigner.mock(wallets: [ethWallet, bitcoinWallet, cosmosWallet])

        let wallets = try signer.getWallets(for: .nonEIP155Optional())

        #expect(wallets.count == 1)
        #expect(wallets.first?.walletId == cosmosWallet.walletId)
    }

    @Test
    func validateChainPresent() async throws {
        let db = DB.mock()
        let walletStore = WalletStore(db: db)
        let connectionsStore = ConnectionsStore(db: db)

        let wallet = Wallet.mock(id: "multicoin_0x1", accounts: [.mock(chain: .ethereum)])
        try walletStore.addWallet(wallet)

        let signer = WalletConnectorSigner.mock(
            connectionsStore: connectionsStore,
            walletSessionService: WalletSessionService.mock(store: walletStore)
        )

        let sessionId = "session-chain-test"
        try signer.addConnection(connection: WalletConnection(
            session: .mock(id: sessionId, sessionId: sessionId, chains: [.ethereum]),
            wallet: wallet
        ))

        let message = SignMessage(chain: "ethereum", signType: .eip191, data: Data())
        await #expect(throws: WalletConnectorServiceError.unresolvedChainId(Chain.polygon.rawValue)) {
            try await signer.signMessage(sessionId: sessionId, chain: .polygon, message: message)
        }
    }

    @Test
    func validateChainEmptyChains() async throws {
        let db = DB.mock()
        let walletStore = WalletStore(db: db)
        let connectionsStore = ConnectionsStore(db: db)

        let wallet = Wallet.mock(id: "multicoin_0x1", accounts: [.mock(chain: .ethereum)])
        try walletStore.addWallet(wallet)

        let signer = WalletConnectorSigner.mock(
            connectionsStore: connectionsStore,
            walletSessionService: WalletSessionService.mock(store: walletStore)
        )

        let sessionId = "session-empty-chains"
        try signer.addConnection(connection: WalletConnection(
            session: .mock(id: sessionId, sessionId: sessionId, chains: []),
            wallet: wallet
        ))

        let message = SignMessage(chain: "ethereum", signType: .eip191, data: Data())
        await #expect(throws: WalletConnectorServiceError.unresolvedChainId(Chain.ethereum.rawValue)) {
            try await signer.signMessage(sessionId: sessionId, chain: .ethereum, message: message)
        }
    }

    @Test
    func sessionBindsToConnectedWallet() throws {
        let db = DB.mock()
        let walletStore = WalletStore(db: db)
        let connectionsStore = ConnectionsStore(db: db)

        let walletA = Wallet.mock(id: "multicoin_0xa", name: "Wallet A", accounts: [.mock(chain: .ethereum)])
        let walletB = Wallet.mock(id: "multicoin_0xb", name: "Wallet B", accounts: [.mock(chain: .ethereum)])

        try walletStore.addWallet(walletA)
        try walletStore.addWallet(walletB)

        let signer = WalletConnectorSigner.mock(
            connectionsStore: connectionsStore,
            walletSessionService: WalletSessionService.mock(store: walletStore)
        )

        let sessionAId = "session-for-wallet-a"
        let sessionBId = "session-for-wallet-b"

        try signer.addConnection(connection: WalletConnection(
            session: .mock(id: sessionAId, sessionId: sessionAId, chains: [.ethereum]),
            wallet: walletA
        ))
        try signer.addConnection(connection: WalletConnection(
            session: .mock(id: sessionBId, sessionId: sessionBId, chains: [.ethereum]),
            wallet: walletB
        ))

        let connectionA = try connectionsStore.getConnection(id: sessionAId)
        let connectionB = try connectionsStore.getConnection(id: sessionBId)

        #expect(connectionA.wallet.id == walletA.id)
        #expect(connectionB.wallet.id == walletB.id)
    }
}

extension WalletConnectorSigner {
    static func mock(
        connectionsStore: ConnectionsStore = .mock(),
        walletSessionService: any WalletSessionManageable = WalletSessionService.mock(
            store: .mock(),
            preferences: .mock()
        )
    ) -> WalletConnectorSigner {
        WalletConnectorSigner(
            connectionsStore: connectionsStore,
            walletSessionService: walletSessionService,
            walletConnectorInteractor: WalletConnectorManager(presenter: WalletConnectorPresenter())
        )
    }
    
    static func mock(wallets: [Wallet]) throws -> WalletConnectorSigner {
        let db = DB.mock()
        let walletStore = WalletStore(db: db)
        for wallet in wallets {
            try walletStore.addWallet(wallet)
        }
        
        return WalletConnectorSigner.mock(
            connectionsStore: ConnectionsStore(db: db),
            walletSessionService: WalletSessionService.mock(store: walletStore))
    }
}

extension Session.Proposal {
    static func requiredChains() throws -> Session.Proposal {
        try Bundle.decode(from: "RequiredChainsProposal", withExtension: "json", in: .module)
    }
    
    static func requiredChainsNoMatch() throws -> Session.Proposal {
        try Bundle.decode(from: "RequiredChainsNoMatchProposal", withExtension: "json", in: .module)
    }

    static func emptyOptionalChains() throws -> Session.Proposal {
        try Bundle.decode(from: "EmptyOptionalChainsProposal", withExtension: "json", in: .module)
    }
    
    static func multiOptionalNamespaces() throws -> Session.Proposal {
        try Bundle.decode(from: "MultiOptionalNamespacesProposal", withExtension: "json", in: .module)
    }
    
    static func mixedRequiredOptional() throws -> Session.Proposal {
        try Bundle.decode(from: "MixedRequiredOptionalProposal", withExtension: "json", in: .module)
    }
    
    static func nonEIP155Optional() throws -> Session.Proposal {
        try Bundle.decode(from: "NonEIP155OptionalProposal", withExtension: "json", in: .module)
    }
}
