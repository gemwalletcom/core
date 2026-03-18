// Copyright (c). Gem Wallet. All rights reserved.

import DeviceService
import Store
import StoreTestKit

public extension DeviceObserverService {
    static func mock(
        deviceService: any DeviceServiceable = DeviceServiceMock(),
        subscriptionsService: SubscriptionService = .mock(),
        subscriptionsObserver: SubscriptionsObserver = .mock()
    ) -> DeviceObserverService {
        DeviceObserverService(
            deviceService: deviceService,
            subscriptionsService: subscriptionsService,
            subscriptionsObserver: subscriptionsObserver
        )
    }
}
