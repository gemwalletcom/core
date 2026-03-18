// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Preferences
import Primitives
import Store

public struct SubscriptionService: Sendable {
    private let subscriptionProvider: any GemAPISubscriptionService
    private let preferences: Preferences
    private let walletStore: WalletStore

    public init(
        subscriptionProvider: any GemAPISubscriptionService,
        walletStore: WalletStore,
        preferences: Preferences = .standard
    ) {
        self.subscriptionProvider = subscriptionProvider
        self.walletStore = walletStore
        self.preferences = preferences
    }

    public func invalidateSubscriptions() {
        preferences.invalidateSubscriptions()
    }

    public func update() async throws {
        let local = try walletStore.getWallets().map { wallet in
            WalletSubscription(
                walletId: wallet.id,
                source: wallet.source,
                subscriptions: wallet.addressChains
            )
        }
        let remote = try await subscriptionProvider.getSubscriptions()
        let changes = Self.calculateChanges(local: local, remote: remote)

        guard changes.hasChanges else {
            preferences.subscriptionsVersionHasChange = false
            return
        }

        if !changes.toAdd.isEmpty {
            try await subscriptionProvider.addSubscriptions(subscriptions: changes.toAdd)
        }

        if !changes.toDelete.isEmpty {
            try await subscriptionProvider.deleteSubscriptions(subscriptions: changes.toDelete)
        }

        preferences.subscriptionsVersionHasChange = false
    }

    static func calculateChanges(
        local: [WalletSubscription],
        remote: [WalletSubscriptionChains]
    ) -> SubscriptionChanges {
        let remoteByWallet = Dictionary(uniqueKeysWithValues: remote.map { ($0.walletId, $0) })
        let localByWallet = Dictionary(uniqueKeysWithValues: local.map { ($0.walletId, $0) })

        let toAdd = local.compactMap { wallet -> WalletSubscription? in
            let remoteChains = Set(remoteByWallet[wallet.walletId]?.chains ?? [])
            let newSubscriptions = wallet.subscriptions.compactMap { addressChains -> AddressChains? in
                let newChains = addressChains.chains.filter { !remoteChains.contains($0) }
                guard !newChains.isEmpty else { return nil }
                return AddressChains(address: addressChains.address, chains: newChains)
            }
            guard !newSubscriptions.isEmpty else { return nil }
            return WalletSubscription(walletId: wallet.walletId, source: wallet.source, subscriptions: newSubscriptions)
        }

        let toDelete = remote.compactMap { remoteWallet -> WalletSubscriptionChains? in
            guard let localWallet = localByWallet[remoteWallet.walletId] else {
                return remoteWallet
            }
            let localChains = Set(localWallet.subscriptions.flatMap(\.chains))
            let chainsToDelete = remoteWallet.chains.filter { !localChains.contains($0) }
            guard !chainsToDelete.isEmpty else { return nil }
            return WalletSubscriptionChains(walletId: remoteWallet.walletId, chains: chainsToDelete.sorted())
        }

        return SubscriptionChanges(toAdd: toAdd, toDelete: toDelete)
    }
}
