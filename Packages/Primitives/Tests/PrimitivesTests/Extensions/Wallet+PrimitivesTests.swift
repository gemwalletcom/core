// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Testing

struct Wallet_PrimitivesTests {
    @Test
    func canSign() {
        #expect(Wallet.mock(type: .multicoin).canSign == true)
        #expect(Wallet.mock(type: .view).canSign == false)
    }

    @Test
    func addressChains() {
        let wallet = Wallet.mock(accounts: [
            .mock(chain: .ethereum, address: "0x1"),
            .mock(chain: .polygon, address: "0x1"),
            .mock(chain: .bitcoin, address: "bc1")
        ])
        let result = wallet.addressChains.sorted { $0.address < $1.address }

        #expect(result.count == 2)
        #expect(result[0].address == "0x1")
        #expect(Set(result[0].chains) == Set([.ethereum, .polygon]))
        #expect(result[1] == AddressChains(address: "bc1", chains: [.bitcoin]))
    }

    @Test
    func walletIdFromType() throws {
        #expect(throws: Error.self) {
            try WalletId.from(type: .multicoin, accounts: [.mock(chain: .bitcoin, address: "0x123")])
        }
        #expect(try WalletId.from(type: .multicoin, accounts: [.mock(chain: .ethereum, address: "0x123")]) == .multicoin(address: "0x123"))
        #expect(try WalletId.from(type: .single, accounts: [.mock(chain: .ethereum, address: "0x456")]) == .single(chain: .ethereum, address: "0x456"))
        #expect(try WalletId.from(type: .privateKey, accounts: [.mock(chain: .bitcoin, address: "bc1abc")]) == .privateKey(chain: .bitcoin, address: "bc1abc"))
        #expect(try WalletId.from(type: .view, accounts: [.mock(chain: .ethereum, address: "0x789")]) == .view(chain: .ethereum, address: "0x789"))
    }
}
