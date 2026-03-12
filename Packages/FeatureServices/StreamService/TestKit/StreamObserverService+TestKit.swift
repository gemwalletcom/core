// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import StreamService
import WebSocketClient
import WebSocketClientTestKit

public extension StreamObserverService {
    static func mock(
        subscriptionService: StreamSubscriptionService = .mock(),
        eventService: StreamEventService = .mock(),
        webSocket: any WebSocketConnectable = WebSocketConnectionMock()
    ) -> StreamObserverService {
        StreamObserverService(
            subscriptionService: subscriptionService,
            eventService: eventService,
            webSocket: webSocket
        )
    }
}
