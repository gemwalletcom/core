// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Store
import Primitives
import PrimitivesTestKit
import StoreTestKit

struct ConnectionsStoreTests {

    @Test
    func getConnectionReturnsBoundWallet() throws {
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)
        let connectionsStore = ConnectionsStore(db: db)

        let walletA = Wallet.mock(id: "multicoin_0xa", accounts: [.mock(chain: .ethereum)])
        let walletB = Wallet.mock(id: "multicoin_0xb", accounts: [.mock(chain: .ethereum)])
        try walletStore.addWallet(walletA)
        try walletStore.addWallet(walletB)

        try connectionsStore.addConnection(.mock(session: .mock(sessionId: "session-a"), wallet: walletA))
        try connectionsStore.addConnection(.mock(session: .mock(sessionId: "session-b"), wallet: walletB))

        #expect(try connectionsStore.getConnection(id: "session-a").wallet.id == walletA.id)
        #expect(try connectionsStore.getConnection(id: "session-b").wallet.id == walletB.id)
    }

    @Test
    func getConnectionThrowsForNonexistentSession() throws {
        #expect(throws: Error.self) {
            _ = try ConnectionsStore.mock().getConnection(id: "nonexistent")
        }
    }
}
