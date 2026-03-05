// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Store

extension View {
    func inject(resolver: AppResolver) -> some View {
        self
            .inject(storages: resolver.storages)
            .inject(services: resolver.services)
            .inject(navigation: resolver.navigation)
    }

    private func inject(services: AppResolver.Services) -> some View {
        self
            .environment(\.nodeService, services.nodeService)
            .environment(\.walletService, services.walletService)
            .environment(\.walletSessionService, services.walletSessionService)
            .environment(\.assetsEnabler, services.assetsEnabler)
            .environment(\.assetDiscoveryService, services.assetDiscoveryService)
            .environment(\.deviceService, services.deviceService)
            .environment(\.transactionsService, services.transactionsService)
            .environment(\.assetsService, services.assetsService)
            .environment(\.stakeService, services.stakeService)
            .environment(\.bannerService, services.bannerService)
            .environment(\.balanceService, services.balanceService)
            .environment(\.priceAlertService, services.priceAlertService)
            .environment(\.navigationPresenter, services.navigationPresenter)
            .environment(\.chainServiceFactory, services.chainServiceFactory)
            .environment(\.priceService, services.priceService)
            .environment(\.streamSubscriptionService, services.streamSubscriptionService)
            .environment(\.explorerService, services.explorerService)
            .environment(\.scanService, services.scanService)
            .environment(\.connectionsService, services.connectionsService)
            .environment(\.nftService, services.nftService)
            .environment(\.avatarService, services.avatarService)
            .environment(\.releaseService, services.appReleaseService)
            .environment(\.viewModelFactory, services.viewModelFactory)
            .environment(\.inAppNotificationService, services.inAppNotificationService)
            .environment(\.fiatService, services.fiatService)
    }
    
    private func inject(storages: AppResolver.Storages) -> some View {
        self
            .databaseQueue(storages.db.dbQueue)
            .environment(\.keystore, storages.keystore)
            .environment(\.observablePreferences, storages.observablePreferences)
    }

    private func inject(navigation: NavigationStateManager) -> some View {
        self
            .environment(\.navigationState, navigation)
    }
}
