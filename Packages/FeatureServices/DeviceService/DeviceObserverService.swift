// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store

public actor DeviceObserverService {
    private let deviceService: any DeviceServiceable
    private let subscriptionsService: SubscriptionService
    private let subscriptionsObserver: SubscriptionsObserver

    private var nodeAuthTokenUpdateTask: Task<Void, Never>?

    public init(
        deviceService: any DeviceServiceable,
        subscriptionsService: SubscriptionService,
        subscriptionsObserver: SubscriptionsObserver
    ) {
        self.deviceService = deviceService
        self.subscriptionsService = subscriptionsService
        self.subscriptionsObserver = subscriptionsObserver
    }

    public func startSubscriptionsObserver() async throws {
        for try await _ in subscriptionsObserver.observe().dropFirst() {
            try await handleSubscriptionsChange()
        }
    }

    public func startNodeAuthTokenUpdates() {
        guard nodeAuthTokenUpdateTask == nil else { return }

        nodeAuthTokenUpdateTask = Task { [deviceService] in
            try? await deviceService.updateNodeAuthTokenIfNeeded()
            while !Task.isCancelled {
                try? await Task.sleep(for: DeviceService.nodeAuthTokenUpdateInterval)
                try? await deviceService.updateNodeAuthTokenIfNeeded()
            }
        }
    }

    public func stopNodeAuthTokenUpdates() {
        nodeAuthTokenUpdateTask?.cancel()
        nodeAuthTokenUpdateTask = nil
    }

    func handleSubscriptionsChange() async throws {
        subscriptionsService.invalidateSubscriptions()
        try await deviceService.update()
    }
}
