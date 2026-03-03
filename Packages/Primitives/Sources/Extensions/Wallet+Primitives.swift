// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension Wallet: Identifiable {}

public extension Wallet {
    var canSign: Bool {
        !isViewOnly
    }

    var isViewOnly: Bool {
        return type == .view
    }

    var isMultiCoins: Bool {
        return type == .multicoin
    }

    var walletId: WalletId {
        try! WalletId.from(id: id)
    }

    var addressChains: [AddressChains] {
        Dictionary(grouping: accounts, by: \.address)
            .map { AddressChains(address: $0.key, chains: Set($0.value.map(\.chain)).sorted()) }
    }

    var hasTokenSupport: Bool {
        accounts.map { $0.chain }.asSet().intersection(AssetConfiguration.supportedChainsWithTokens).isNotEmpty
    }

    func account(for chain: Chain) throws -> Account {
        guard let account = accounts.filter({ $0.chain == chain }).first else {
            throw AnyError("account not found for chain: \(chain.rawValue)")
        }
        return account
    }

    var hyperliquidAccount: Account? {
        accounts.first {
            $0.chain == .arbitrum || $0.chain == .hyperCore || $0.chain == .hyperliquid
        }
    }
}

// factory
public extension Wallet {
    static func makeView(name: String, chain: Chain, address: String) -> Wallet {
        let id = WalletId.make(walletType: .view, chain: chain, address: address).id
        return Wallet(
            id: id,
            externalId: nil,
            name: name,
            index: 0,
            type: .view,
            accounts: [
                Account(
                    chain: chain,
                    address: address,
                    derivationPath: "",
                    extendedPublicKey: ""
                ),
            ],
            order: 0,
            isPinned: false,
            imageUrl: nil,
            source: .import
        )
    }
}
