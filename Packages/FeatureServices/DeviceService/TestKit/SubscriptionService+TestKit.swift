// Copyright (c). Gem Wallet. All rights reserved.

import DeviceService
import GemAPI
import GemAPITestKit
import Preferences
import Store
import StoreTestKit

public extension SubscriptionService {
    static func mock(
        subscriptionProvider: any GemAPISubscriptionService = GemAPISubscriptionServiceMock(),
        walletStore: WalletStore = .mock(),
        preferences: Preferences = .standard
    ) -> SubscriptionService {
        SubscriptionService(
            subscriptionProvider: subscriptionProvider,
            walletStore: walletStore,
            preferences: preferences
        )
    }
}
