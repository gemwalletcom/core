// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Store
import Preferences
import Primitives
import GemstonePrimitives

public struct ImportAssetsService: Sendable {
    let assetListService: any GemAPIAssetsListService
    let assetsService: AssetsService
    let assetStore: AssetStore
    let preferences: Preferences
    
    public init(
        assetListService: any GemAPIAssetsListService,
        assetsService: AssetsService,
        assetStore: AssetStore,
        preferences: Preferences
    ) {
        self.assetListService = assetListService
        self.assetsService = assetsService
        self.assetStore = assetStore
        self.preferences = preferences
    }
    
    // sync
    public func migrate() throws {
        let releaseVersionNumber = Bundle.main.buildVersionNumber
        
        #if targetEnvironment(simulator)
        #else
        guard preferences.localAssetsVersion < releaseVersionNumber else {
            return
        }
        preferences.localAssetsVersion = releaseVersionNumber
        #endif
        
        let chains = AssetConfiguration.allChains
        let defaultAssets = chains.map { $0.defaultAssets }.flatMap { $0 }
        let assetIds = chains.map { $0.id } + defaultAssets.ids
        
        let localAssetsVersion = try assetStore.getAssets(for: assetIds).map { $0.id }
        
        if localAssetsVersion.count != assetIds.count {
            let assets = chains.map {
                let chain = $0.asset.chain
                let score = AssetScore.defaultScore(chain: $0.asset.chain)
                let isStakable = GemstoneConfig.shared.getChainConfig(chain: chain.rawValue).isStakeSupported
                let isSwapable = GemstoneConfig.shared.getChainConfig(chain: chain.rawValue).isSwapSupported
                let isBuyable = score.rank >= 40
                
                return AssetBasic(
                    asset: $0.asset,
                    properties: AssetProperties(
                        isEnabled: true,
                        isBuyable: isBuyable,
                        isSellable: false,
                        isSwapable: isSwapable,
                        isStakeable: isStakable,
                        stakingApr: .none,
                        isEarnable: false,
                        earnApr: nil,
                        hasImage: true
                    ),
                    score: score,
                    price: nil
                )
            }
            try assetStore.add(assets: assets)
            
            try assetStore.insert(assets: defaultAssets.map {
                AssetBasic(
                    asset: $0,
                    properties: AssetProperties(
                        isEnabled: true,
                        isBuyable: false,
                        isSellable: false,
                        isSwapable: false,
                        isStakeable: false,
                        stakingApr: .none,
                        isEarnable: false,
                        earnApr: nil,
                        hasImage: false
                    ),
                    score: AssetScore(rank: 16),
                    price: nil
                )
            })
        }
        
        try assetStore.setAssetIsStakeable(for: chains.filter { $0.isStakeSupported }.map { $0.id }, value: true)
    }

    public func updateFiatAssets() async throws {
        async let getBuyAssets = try assetListService.getBuyableFiatAssets()
        async let getSellAssets = try assetListService.getSellableFiatAssets()

        let (buyAssets, sellAssets) = try await (getBuyAssets, getSellAssets)

        let assetIds = (buyAssets.assetIds + sellAssets.assetIds).compactMap { try? AssetId(id: $0) }

        async let prefetchResult = try assetsService.prefetchAssets(assetIds: assetIds)
        async let setBuyableResult = try assetStore.setAssetIsBuyable(for: buyAssets.assetIds, value: true)
        async let setSellableResult = try assetStore.setAssetIsSellable(for: sellAssets.assetIds, value: true)

        _ = try await (prefetchResult, setBuyableResult, setSellableResult)

        preferences.fiatOnRampAssetsVersion = Int(buyAssets.version)
        preferences.fiatOffRampAssetsVersion = Int(sellAssets.version)
    }
    
    public func updateSwapAssets() async throws {
        let assets = try await assetListService.getSwapAssets()

        try await assetsService.prefetchAssets(assetIds: assets.assetIds.compactMap { try? AssetId(id: $0) })
        try assetStore.setAssetIsSwappable(for: assets.assetIds, value: true)
    
        preferences.swapAssetsVersion = Int(assets.version)
    }
}
