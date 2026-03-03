// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Testing

struct WalletIdTests {
    @Test
    func id() {
        #expect(WalletId.multicoin(address: "0x123").id == "multicoin_0x123")
        #expect(WalletId.single(chain: .ethereum, address: "0x456").id == "single_ethereum_0x456")
    }

    @Test
    func walletTypeAndChain() {
        #expect(WalletId.multicoin(address: "0x123").walletType == .multicoin)
        #expect(WalletId.multicoin(address: "0x123").chain == nil)
        #expect(WalletId.single(chain: .ethereum, address: "0x456").chain == .ethereum)
    }

    @Test
    func fromId() throws {
        #expect(try WalletId.from(id: "multicoin_0x123") == .multicoin(address: "0x123"))
        #expect(try WalletId.from(id: "single_ethereum_0x456") == .single(chain: .ethereum, address: "0x456"))
        #expect(throws: Error.self) { try WalletId.from(id: "invalid") }
    }

    @Test
    func fromIdAndBackToId() throws {
        let types: [WalletId] = [
            .multicoin(address: "0xabc"),
            .single(chain: .ethereum, address: "0x123"),
            .privateKey(chain: .bitcoin, address: "bc1test"),
            .view(chain: .solana, address: "soladdr"),
        ]
        for type in types {
            #expect(try WalletId.from(id: type.id) == type)
        }
    }

    @Test
    func codableRoundTrip() throws {
        let walletId = WalletId.multicoin(address: "0xabc")
        let data = try JSONEncoder().encode(walletId)
        let decoded = try JSONDecoder().decode(WalletId.self, from: data)
        #expect(decoded == walletId)
    }

    @Test
    func decodeBareString() throws {
        let json = Data("\"multicoin_0xabc\"".utf8)
        let decoded = try JSONDecoder().decode(WalletId.self, from: json)
        #expect(decoded == .multicoin(address: "0xabc"))
    }
}
