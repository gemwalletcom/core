// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Foundation
import Store
import StoreTestKit
import PrimitivesTestKit
import Primitives

struct RecentActivityRequestTests {

    @Test
    func fetchRecentAssets() throws {
        let db = DB.mockAssets()
        let store = RecentActivityStore(db: db)
        let btc = AssetId(chain: .bitcoin)
        let bnb = AssetId(chain: .smartChain)
        let now = Date()

        try store.add(assetId: btc, toAssetId: .none, walletId: WalletId.mock(), type: .search, createdAt: now.addingTimeInterval(-2))
        try store.add(assetId: bnb, toAssetId: .none, walletId: WalletId.mock(), type: .search, createdAt: now.addingTimeInterval(-1))
        try store.add(assetId: btc, toAssetId: .none, walletId: WalletId.mock(), type: .transfer, createdAt: now)

        try db.dbQueue.read { db in
            let result = try RecentActivityRequest(walletId: WalletId.mock(), limit: 10).fetch(db)

            #expect(result.count == 2)
            #expect(result.first?.asset.id == btc)
            #expect(result.last?.asset.id == bnb)
        }
    }

    @Test
    func fetchRecentAssetsWithFilters() throws {
        let db = DB.mockAssets()
        let store = RecentActivityStore(db: db)
        let assetStore = AssetStore(db: db)
        let btc = AssetId(chain: .bitcoin)
        let bnb = AssetId(chain: .smartChain)
        let eth = AssetId(chain: .ethereum)
        let walletId = WalletId.mock()

        try store.add(assetId: btc, toAssetId: .none, walletId: walletId, type: .search, createdAt: Date())
        try store.add(assetId: bnb, toAssetId: .none, walletId: walletId, type: .search, createdAt: Date())
        try store.add(assetId: eth, toAssetId: .none, walletId: walletId, type: .search, createdAt: Date())
        try assetStore.setAssetIsBuyable(for: [btc.identifier], value: false)

        try db.dbQueue.read { db in
            let noFilter = try RecentActivityRequest(walletId: walletId, limit: 10).fetch(db)
            let hasBalance = try RecentActivityRequest(walletId: walletId, limit: 10, filters: [.hasBalance]).fetch(db)
            let buyable = try RecentActivityRequest(walletId: walletId, limit: 10, filters: [.buyable]).fetch(db)
            let chains = try RecentActivityRequest(walletId: walletId, limit: 10, filters: [.chains([Chain.ethereum.rawValue])]).fetch(db)

            #expect(noFilter.count == 3)
            #expect(hasBalance.count == 2)
            #expect(hasBalance.map(\.asset.id).contains(btc) == false)
            #expect(buyable.count == 2)
            #expect(buyable.map(\.asset.id).contains(btc) == false)
            #expect(chains.count == 1)
            #expect(chains.first?.asset.id == eth)
        }
    }
}
