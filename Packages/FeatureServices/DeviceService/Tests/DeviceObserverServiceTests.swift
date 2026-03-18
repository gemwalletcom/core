// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import DeviceServiceTestKit
import Preferences
import PreferencesTestKit
import StoreTestKit

@testable import DeviceService

struct DeviceObserverServiceTests {

    @Test
    func handleSubscriptionsChangeInvalidatesAndUpdatesDevice() async throws {
        let preferences = Preferences.mock()
        preferences.subscriptionsVersionHasChange = false

        let deviceService = DeviceServiceMock()
        let observerService = DeviceObserverService(
            deviceService: deviceService,
            subscriptionsService: .mock(preferences: preferences),
            subscriptionsObserver: .mock()
        )

        try await observerService.handleSubscriptionsChange()

        #expect(preferences.subscriptionsVersionHasChange)
        #expect(await deviceService.updateCalls == 1)
    }
}
