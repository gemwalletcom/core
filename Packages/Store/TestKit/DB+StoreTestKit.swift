// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store
import Primitives
import PrimitivesTestKit

public extension DB {
    static func mock() -> DB {
        DB(fileName: "\(UUID().uuidString).sqlite")
    }

    static func mockWithChains(_ chains: [Chain] = [.bitcoin]) -> DB {
        let db = Self.mock()
        let assetStore = AssetStore(db: db)
        try? assetStore.add(assets: chains.map { .mock(asset: .mock(id: $0.assetId)) })
        return db
    }

    static func mockAssets(assets: [AssetBasic] = .mock()) -> DB {
        let db = Self.mock()
        let assetStore = AssetStore(db: db)
        let balanceStore = BalanceStore(db: db)
        let walletStore = WalletStore(db: db)

        let existingChainIds = assets.filter { $0.asset.type == .native }.map(\.asset.chain).asSet()
        let allChains = assets.map(\.asset.chain).asSet()
        let missingChains = allChains.subtracting(existingChainIds)
        let chainAssets: [AssetBasic] = missingChains.map { .mock(asset: .mock(id: $0.assetId)) }

        try? assetStore.add(assets: assets + chainAssets)
        try? walletStore.addWallet(.mock(accounts: assets.map { Account.mock(chain: $0.asset.chain) }))
        try? balanceStore.addBalance(assets.map { AddBalance(assetId: $0.asset.id, isEnabled: true) }, for: .mock())
        try? balanceStore.updateBalances(.mock(assets: assets), for: .mock())
        
        return db
    }
}
