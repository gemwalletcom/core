// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import GemAPI
import Store
import WalletService
import Preferences

public struct InAppNotificationService: Sendable {
    private let apiService: GemAPINotificationService
    private let walletService: WalletService
    private let store: InAppNotificationStore

    public init(
        apiService: GemAPINotificationService = GemAPIService.shared,
        walletService: WalletService,
        store: InAppNotificationStore
    ) {
        self.apiService = apiService
        self.walletService = walletService
        self.store = store
    }

    public func update(walletId: WalletId) async throws {
        let preferences = WalletPreferences(walletId: walletId)
        let newTimestamp = Int(Date.now.timeIntervalSince1970)

        let notifications = try await apiService.getNotifications(
            fromTimestamp: preferences.notificationsTimestamp
        )
        try store.addNotifications(notifications)

        preferences.notificationsTimestamp = newTimestamp
    }

    public func markNotificationsRead() async throws {
        try await apiService.markNotificationsRead()
    }
}
