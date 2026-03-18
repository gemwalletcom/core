// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public actor GemAPISubscriptionServiceMock: GemAPISubscriptionService {
    private let delay: Duration?
    private let subscriptions: [WalletSubscriptionChains]
    private let getSubscriptionsError: Error?

    public private(set) var getSubscriptionsCalls = 0

    public init(
        delay: Duration? = nil,
        subscriptions: [WalletSubscriptionChains] = [],
        getSubscriptionsError: Error? = nil
    ) {
        self.delay = delay
        self.subscriptions = subscriptions
        self.getSubscriptionsError = getSubscriptionsError
    }

    public func getSubscriptions() async throws -> [WalletSubscriptionChains] {
        getSubscriptionsCalls += 1
        if let delay {
            try await Task.sleep(for: delay)
        }
        if let getSubscriptionsError {
            throw getSubscriptionsError
        }
        return subscriptions
    }

    public func addSubscriptions(subscriptions: [WalletSubscription]) async throws {}

    public func deleteSubscriptions(subscriptions: [WalletSubscriptionChains]) async throws {}
}
