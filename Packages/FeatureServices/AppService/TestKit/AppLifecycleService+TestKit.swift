// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Preferences
import PreferencesTestKit
import DeviceService
import DeviceServiceTestKit
import StreamService
import StreamServiceTestKit
import PerpetualService
import PerpetualServiceTestKit
import ConnectionsService
import ConnectionsServiceTestKit
@testable import AppService

public extension AppLifecycleService {
    static func mock(
        preferences: Preferences = .mock(),
        connectionsService: ConnectionsService = .mock(),
        deviceObserverService: DeviceObserverService = .mock(),
        streamObserverService: StreamObserverService = .mock(),
        streamSubscriptionService: StreamSubscriptionService = .mock(),
        hyperliquidObserverService: PerpetualObserverMock = PerpetualObserverMock()
    ) -> AppLifecycleService {
        AppLifecycleService(
            preferences: preferences,
            connectionsService: connectionsService,
            deviceObserverService: deviceObserverService,
            streamObserverService: streamObserverService,
            streamSubscriptionService: streamSubscriptionService,
            hyperliquidObserverService: hyperliquidObserverService
        )
    }
}
