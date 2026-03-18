// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import AssetsService
import BalanceService
import GemAPI
import Primitives
import Preferences

public struct AssetDiscoveryService: AssetDiscoverable {
    private let assetsListService: any GemAPIAssetsListService
    private let assetService: AssetsService
    private let assetsEnabler: any AssetsEnabler

    public init(
        assetsListService: any GemAPIAssetsListService,
        assetService: AssetsService,
        assetsEnabler: any AssetsEnabler
    ) {
        self.assetsListService = assetsListService
        self.assetService = assetService
        self.assetsEnabler = assetsEnabler
    }

    public func discoverAssets(wallet: Wallet) async throws {
        let preferences = WalletPreferences(walletId: wallet.walletId)
        
        let assetIds = try await assetsListService.getDeviceAssets(walletId: wallet.id, fromTimestamp: preferences.assetsTimestamp)

        if assetIds.isNotEmpty {
            try await assetService.prefetchAssets(assetIds: assetIds)
            try await assetsEnabler.enableAssets(wallet: wallet, assetIds: assetIds, enabled: true)
        }

        preferences.completeInitialLoadAssets = true
        preferences.assetsTimestamp = Int(Date.now.timeIntervalSince1970)
    }
}
