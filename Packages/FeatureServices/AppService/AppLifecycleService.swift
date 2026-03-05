// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import DeviceService
import StreamService
import PerpetualService
import ConnectionsService
import Primitives
import Preferences

public actor AppLifecycleService: Sendable {
    private let preferences: Preferences
    private let connectionsService: ConnectionsService
    private let deviceObserverService: DeviceObserverService
    private let streamObserverService: StreamObserverService
    private let streamSubscriptionService: StreamSubscriptionService
    private let hyperliquidObserverService: any PerpetualObservable<HyperliquidSubscription>

    private var currentWallet: Wallet?

    public init(
        preferences: Preferences,
        connectionsService: ConnectionsService,
        deviceObserverService: DeviceObserverService,
        streamObserverService: StreamObserverService,
        streamSubscriptionService: StreamSubscriptionService,
        hyperliquidObserverService: any PerpetualObservable<HyperliquidSubscription>
    ) {
        self.preferences = preferences
        self.connectionsService = connectionsService
        self.deviceObserverService = deviceObserverService
        self.streamObserverService = streamObserverService
        self.streamSubscriptionService = streamSubscriptionService
        self.hyperliquidObserverService = hyperliquidObserverService
    }

    public func setup() async {
        async let walletConnect: () = setupWalletConnect()
        async let device: () = setupDeviceObserver()
        async let observers: () = connectObservers()

        _ = await (walletConnect, device, observers)
    }

    public func setupWallet(_ wallet: Wallet) async {
        currentWallet = wallet
        async let assets: () = setupPriceAssets(wallet: wallet)
        async let perpetual: () = connectPerpetual()
        _ = await (assets, perpetual)
    }

    public func updatePerpetualConnection() async {
        await connectPerpetual()
    }

    public func handleScenePhase(_ phase: ScenePhase) async {
        switch phase {
        case .active:
            debugLog("AppLifecycleService: App active — connecting observers")
            await connectObservers()
        case .background:
            debugLog("AppLifecycleService: App background — disconnecting observers")
            await disconnectObservers()
        case .inactive:
            debugLog("AppLifecycleService: App inactive")
        @unknown default:
            break
        }
    }
}

// MARK: - Private

extension AppLifecycleService {
    private func setupWalletConnect() async {
        do {
            try await connectionsService.setup()
        } catch {
            debugLog("AppLifecycleService setupWalletConnect error: \(error)")
        }
    }

    private func setupDeviceObserver() async {
        do {
            try await deviceObserverService.startSubscriptionsObserver()
        } catch {
            debugLog("AppLifecycleService setupDeviceObserver error: \(error)")
        }
    }

    private func setupPriceAssets(wallet: Wallet) async {
        do {
            try await streamSubscriptionService.setupAssets(walletId: wallet.walletId)
        } catch {
            debugLog("AppLifecycleService setupPriceAssets error: \(error)")
        }
    }

    private func connectObservers() async {
        async let price: () = streamObserverService.connect()
        async let perpetual: () = connectPerpetual()
        async let authToken: () = deviceObserverService.startAuthTokenRefresh()
        _ = await (price, perpetual, authToken)
    }

    private func connectPerpetual() async {
        if let wallet = currentWallet, wallet.isMultiCoins, preferences.isPerpetualEnabled {
            await hyperliquidObserverService.setup(for: wallet)
        } else {
            await hyperliquidObserverService.disconnect()
        }
    }

    private func disconnectObservers() async {
        async let price: () = streamObserverService.disconnect()
        async let perpetual: () = hyperliquidObserverService.disconnect()
        async let authToken: () = deviceObserverService.stopAuthTokenRefresh()
        _ = await (price, perpetual, authToken)
    }
}
