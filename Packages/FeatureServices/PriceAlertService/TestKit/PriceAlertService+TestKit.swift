// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PriceAlertService
import StoreTestKit
import Store
import DeviceService
import DeviceServiceTestKit
import GemAPITestKit
import GemAPI
import PriceService
import PriceServiceTestKit
import Preferences
import PreferencesTestKit

public extension PriceAlertService {
    static func mock(
        store: PriceAlertStore = .mock(),
        apiService: any GemAPIPriceAlertService = GemAPIPriceAlertServiceMock(),
        deviceService: any DeviceServiceable = DeviceServiceMock(),
        priceUpdater: any PriceUpdater = .mock(),
        preferences: Preferences = .mock()
    ) -> PriceAlertService {
        PriceAlertService(
            store: store,
            apiService: apiService,
            deviceService: deviceService,
            priceUpdater: priceUpdater,
            preferences: preferences
        )
    }
}
