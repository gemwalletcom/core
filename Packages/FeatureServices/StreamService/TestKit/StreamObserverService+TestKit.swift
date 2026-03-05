// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store
import StoreTestKit
import StreamService
import PriceService
import PriceServiceTestKit
import PriceAlertService
import PriceAlertServiceTestKit
import BalanceService
import BalanceServiceTestKit
import TransactionsService
import TransactionsServiceTestKit
import NFTService
import NFTServiceTestKit
import PerpetualService
import PerpetualServiceTestKit
import Preferences
import PreferencesTestKit
import WebSocketClient
import WebSocketClientTestKit

public extension StreamObserverService {
    static func mock(
        walletStore: WalletStore = .mock(),
        notificationStore: InAppNotificationStore = .mock(),
        priceService: PriceService = .mock(),
        priceAlertService: PriceAlertService = .mock(),
        balanceUpdater: any BalanceUpdater = .mock(),
        transactionsService: TransactionsService = .mock(),
        nftService: NFTService = .mock(),
        perpetualService: any HyperliquidPerpetualServiceable = PerpetualServiceMock(),
        subscriptionService: StreamSubscriptionService = .mock(),
        preferences: Preferences = .mock(),
        webSocket: any WebSocketConnectable = WebSocketConnectionMock()
    ) -> StreamObserverService {
        StreamObserverService(
            walletStore: walletStore,
            notificationStore: notificationStore,
            priceService: priceService,
            priceAlertService: priceAlertService,
            balanceUpdater: balanceUpdater,
            transactionsService: transactionsService,
            nftService: nftService,
            perpetualService: perpetualService,
            subscriptionService: subscriptionService,
            preferences: preferences,
            webSocket: webSocket
        )
    }
}
