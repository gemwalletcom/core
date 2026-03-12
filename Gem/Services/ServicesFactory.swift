// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import BannerService
import ChainService
import DeviceService
import PriceAlertService
import StakeService
import NotificationService
import Gemstone
import GemstonePrimitives
import NodeService
import WalletConnector
import ConnectionsService
import Store
import GemAPI
import Keystore
import PriceService
import StreamService
import Preferences
import ExplorerService
import BalanceService
import AssetsService
import TransactionsService
import TransactionStateService
import NFTService
import WalletService
import AvatarService
import WalletSessionService
import ScanService
import SwapService
import NameService
import PerpetualService
import AppService
import AddressNameService
import Blockchain
import NativeProviderService
import ActivityService
import AuthService
import DiscoverAssetsService
import RewardsService
import EventPresenterService
import EarnService
import Transfer
import SwiftHTTPClient
import ContactService
import WebSocketClient


struct ServicesFactory {
    func makeServices(storages: AppResolver.Storages, navigation: NavigationStateManager) -> AppResolver.Services {
        let storeManager = StoreManager(db: storages.db)
        let securePreferences = SecurePreferences()
        let signer = Self.makeRequestSigner(securePreferences: securePreferences)
        let interceptor: (@Sendable (inout URLRequest, GemDeviceAPI) throws -> Void)? = if let signer {
            { request, target in
                try signer.sign(request: &request, walletId: target.walletId ?? "")
            }
        } else {
            nil
        }
        let provider = Provider<GemAPI>()
        let deviceProvider = Provider<GemDeviceAPI>(options: ProviderOptions(baseUrl: nil, requestInterceptor: interceptor))
        let apiService = GemAPIService(provider: provider, deviceProvider: deviceProvider)

        let subscriptionService = Self.makeSubscriptionService(
            apiService: apiService,
            walletStore: storeManager.walletStore
        )
        let deviceService = Self.makeDeviceService(
            apiService: apiService,
            subscriptionService: subscriptionService,
            securePreferences: securePreferences
        )
        let deviceObserverService = Self.makeDeviceObserverService(
            deviceService: deviceService,
            subscriptionService: subscriptionService,
            walletStore: storeManager.walletStore
        )

        let nodeService = NodeService(nodeStore: storeManager.nodeStore)
        let nodeAuthProvider = NodeAuthTokenProvider(securePreferences: securePreferences)
        let nodeProvider = AuthenticatedNodeProvider(nodeProvider: nodeService, requestInterceptor: nodeAuthProvider)
        let nativeProvider = NativeProvider(nodeProvider: nodeProvider)
        let gatewayService = GatewayService(provider: nativeProvider)
        let chainServiceFactory = ChainServiceFactory(nodeProvider: nodeProvider)

        let avatarService = AvatarService(store: storeManager.walletStore)
        let assetsService = Self.makeAssetsService(
            assetStore: storeManager.assetStore,
            balanceStore: storeManager.balanceStore,
            chainFactory: chainServiceFactory
        )

        let walletService = Self.makeWalletService(
            preferences: storages.observablePreferences,
            keystore: storages.keystore,
            walletStore: storeManager.walletStore,
            avatarService: avatarService
        )
        let earnService = EarnService(
            store: storeManager.stakeStore,
            gatewayService: gatewayService
        )
        let balanceService = Self.makeBalanceService(
            balanceStore: storeManager.balanceStore,
            assetsService: assetsService,
            chainFactory: chainServiceFactory
        )
        let stakeService = Self.makeStakeService(
            stakeStore: storeManager.stakeStore,
            addressStore: storeManager.addressStore,
            chainFactory: chainServiceFactory
        )
        let nftService = Self.makeNftService(
            apiService: apiService,
            nftStore: storeManager.nftStore,
            deviceService: deviceService
        )
        let transactionsService = Self.makeTransactionsService(
            apiService: apiService,
            transactionStore: storeManager.transactionStore,
            assetsService: assetsService,
            walletStore: storeManager.walletStore,
            deviceService: deviceService,
            addressStore: storeManager.addressStore
        )
        let transactionStateService = Self.makeTransactionService(
            transactionStore: storeManager.transactionStore,
            nativeProvider: nativeProvider,
            stakeService: stakeService,
            earnService: earnService,
            nftService: nftService,
            chainFactory: chainServiceFactory,
            balanceService: balanceService
        )

        let preferences = storages.observablePreferences.preferences
        let pushNotificationEnablerService = PushNotificationEnablerService(preferences: preferences)
        let bannerSetupService = BannerSetupService(store: storeManager.bannerStore, preferences: preferences)
        let bannerService = Self.makeBannerService(
            bannerStore: storeManager.bannerStore,
            pushNotificationEnablerService: pushNotificationEnablerService
        )
        let navigationPresenter = NavigationPresenter()
        let navigationHandler = NavigationHandler(
            navigationState: navigation,
            presenter: navigationPresenter,
            assetsService: assetsService,
            transactionsService: transactionsService,
            walletService: walletService
        )

        let priceService = PriceService(
            priceStore: storeManager.priceStore,
            fiatRateStore: storeManager.fiatRateStore
        )
        let perpetualService = Self.makePerpetualService(
            perpetualStore: storeManager.perpetualStore,
            assetStore: storeManager.assetStore,
            priceStore: storeManager.priceStore,
            balanceStore: storeManager.balanceStore,
            nodeProvider: nodeProvider,
            preferences: preferences
        )
        let webSocket = Self.makeWebSocket(securePreferences: securePreferences)
        let streamSubscriptionService = StreamSubscriptionService(
            priceService: priceService,
            webSocket: webSocket
        )
        let priceAlertService = Self.makePriceAlertService(
            apiService: apiService,
            priceAlertStore: storeManager.priceAlertStore,
            deviceService: deviceService,
            priceUpdater: streamSubscriptionService,
            preferences: preferences
        )
        let streamEventService = StreamEventService(
            walletStore: storeManager.walletStore,
            notificationStore: storeManager.inAppNotificationStore,
            priceService: priceService,
            priceAlertService: priceAlertService,
            balanceUpdater: balanceService,
            transactionsService: transactionsService,
            nftService: nftService,
            perpetualService: perpetualService,
            preferences: preferences
        )
        let streamObserverService = StreamObserverService(
            subscriptionService: streamSubscriptionService,
            eventService: streamEventService,
            webSocket: webSocket
        )
        let explorerService = ExplorerService.standard
        let swapService = SwapService(nodeProvider: nodeProvider)

        let walletSessionService = WalletSessionService(
            walletStore: storeManager.walletStore,
            preferences: storages.observablePreferences
        )
        let presenter = WalletConnectorPresenter()
        let walletConnectorManager = WalletConnectorManager(presenter: presenter)
        let connectionsService = Self.makeConnectionsService(
            connectionsStore: storeManager.connectionsStore,
            walletSessionService: walletSessionService,
            interactor: walletConnectorManager
        )

        let assetsEnabler = AssetsEnablerService(
            assetsService: assetsService,
            balanceUpdater: balanceService,
            priceUpdater: streamSubscriptionService
        )
        let assetDiscoveryService = AssetDiscoveryService(
            deviceService: deviceService,
            assetsListService: apiService,
            assetService: assetsService,
            assetsEnabler: assetsEnabler
        )
        let walletSetupService = WalletSetupService(balanceService: balanceService)

        let configService = ConfigService(apiService: apiService)
        let releaseService = AppReleaseService(configService: configService)
        let releaseAlertService = ReleaseAlertService(
            appReleaseService: releaseService,
            preferences: preferences
        )
        let rateService = RateService(preferences: preferences)

        let onStartService = Self.makeOnstartService(
            assetListService: apiService,
            assetStore: storeManager.assetStore,
            nodeStore: storeManager.nodeStore,
            preferences: preferences,
            assetsService: assetsService,
            walletService: walletService
        )
        let onstartAsyncService = Self.makeOnstartAsyncService(
            apiService: apiService,
            nodeService: nodeService,
            preferences: preferences,
            assetsService: assetsService,
            bannerSetupService: bannerSetupService,
            configService: configService,
            swappableChainsProvider: swapService
        )
        let onstartWalletService = Self.makeOnstartWalletService(
            preferences: preferences,
            deviceService: deviceService,
            bannerSetupService: bannerSetupService,
            addressStatusService: AddressStatusService(chainServiceFactory: chainServiceFactory),
            pushNotificationEnablerService: pushNotificationEnablerService
        )

        let hyperliquidObserverService = HyperliquidObserverService(
            nodeProvider: PerpetualNodeService(nodeProvider: nodeProvider),
            perpetualService: perpetualService
        )

        let nameService = NameService(provider: apiService)
        let scanService = ScanService(gatewayService: gatewayService)
        let addressNameService = AddressNameService(addressStore: storeManager.addressStore)
        let activityService = ActivityService(store: storeManager.recentActivityStore)
        let authService = AuthService(apiService: apiService, keystore: storages.keystore)
        let rewardsService = RewardsService(apiService: apiService, authService: authService)
        let eventPresenterService = EventPresenterService()
        let walletSearchService = WalletSearchService(
            assetsService: assetsService,
            searchStore: storeManager.searchStore,
            perpetualStore: storeManager.perpetualStore,
            priceStore: storeManager.priceStore,
            preferences: preferences
        )
        let assetSearchService = AssetSearchService(
            assetsService: assetsService,
            searchStore: storeManager.searchStore
        )
        let inAppNotificationService = InAppNotificationService(
            apiService: apiService,
            walletService: walletService,
            store: storeManager.inAppNotificationStore
        )

        let contactService = ContactService(store: storeManager.contactStore, addressStore: storeManager.addressStore)

        let appLifecycleService = AppLifecycleService(
            preferences: preferences,
            connectionsService: connectionsService,
            deviceObserverService: deviceObserverService,
            streamObserverService: streamObserverService,
            streamSubscriptionService: streamSubscriptionService,
            hyperliquidObserverService: hyperliquidObserverService
        )

        let viewModelFactory = ViewModelFactory(
            keystore: storages.keystore,
            chainServiceFactory: chainServiceFactory,
            scanService: scanService,
            swapService: swapService,
            assetsEnabler: assetsEnabler,
            priceUpdater: streamSubscriptionService,
            walletService: walletService,
            stakeService: stakeService,
            earnService: earnService,
            amountService: AmountService(earnDataProvider: earnService),
            nameService: nameService,
            balanceService: balanceService,
            priceService: priceService,
            transactionStateService: transactionStateService,
            addressNameService: addressNameService,
            activityService: activityService,
            eventPresenterService: eventPresenterService,
            fiatService: apiService
        )

        return AppResolver.Services(
            assetsService: assetsService,
            balanceService: balanceService,
            bannerService: bannerService,
            chainServiceFactory: chainServiceFactory,
            connectionsService: connectionsService,
            deviceService: deviceService,
            nodeService: nodeService,
            navigationHandler: navigationHandler,
            navigationPresenter: navigationPresenter,
            priceAlertService: priceAlertService,
            streamObserverService: streamObserverService,
            streamSubscriptionService: streamSubscriptionService,
            priceService: priceService,
            stakeService: stakeService,
            transactionsService: transactionsService,
            transactionStateService: transactionStateService,
            walletService: walletService,
            walletSessionService: walletSessionService,
            assetsEnabler: assetsEnabler,
            assetDiscoveryService: assetDiscoveryService,
            walletSetupService: walletSetupService,
            explorerService: explorerService,
            scanService: scanService,
            nftService: nftService,
            avatarService: avatarService,
            swapService: swapService,
            appReleaseService: releaseService,
            releaseAlertService: releaseAlertService,
            rateService: rateService,
            subscriptionsService: subscriptionService,
            deviceObserverService: deviceObserverService,
            onstartService: onStartService,
            onstartAsyncService: onstartAsyncService,
            onstartWalletService: onstartWalletService,
            walletConnectorManager: walletConnectorManager,
            perpetualService: perpetualService,
            hyperliquidObserverService: hyperliquidObserverService,
            nameService: nameService,
            addressNameService: addressNameService,
            activityService: activityService,
            eventPresenterService: eventPresenterService,
            viewModelFactory: viewModelFactory,
            rewardsService: rewardsService,
            walletSearchService: walletSearchService,
            assetSearchService: assetSearchService,
            appLifecycleService: appLifecycleService,
            inAppNotificationService: inAppNotificationService,
            fiatService: apiService,
            contactService: contactService
        )
    }
}

// MARK: - Private Static

extension ServicesFactory {
    private static func makeRequestSigner(securePreferences: SecurePreferences) -> DeviceRequestSigner? {
        do {
            let keyPair = try DeviceService.getOrCreateKeyPair(securePreferences: securePreferences)
            return try DeviceRequestSigner(privateKey: keyPair.privateKey)
        } catch {
            debugLog("makeRequestSigner error: \(error)")
            return nil
        }
    }

    private static func makeSubscriptionService(
        apiService: GemAPIService,
        walletStore: WalletStore
    ) -> SubscriptionService {
        SubscriptionService(
            subscriptionProvider: apiService,
            walletStore: walletStore
        )
    }

    private static func makeDeviceService(
        apiService: GemAPIService,
        subscriptionService: SubscriptionService,
        securePreferences: SecurePreferences
    ) -> DeviceService {
        DeviceService(
            deviceProvider: apiService,
            subscriptionsService: subscriptionService,
            securePreferences: securePreferences
        )
    }

    private static func makeDeviceObserverService(
        deviceService: any DeviceServiceable,
        subscriptionService: SubscriptionService,
        walletStore: WalletStore
    ) -> DeviceObserverService {
        DeviceObserverService(
            deviceService: deviceService,
            subscriptionsService: subscriptionService,
            subscriptionsObserver: walletStore.observer()
        )
    }

    private static func makeWalletService(
        preferences: ObservablePreferences,
        keystore: any Keystore,
        walletStore: WalletStore,
        avatarService: AvatarService
    ) -> WalletService {
        WalletService(
            keystore: keystore,
            walletStore: walletStore,
            preferences: preferences,
            avatarService: avatarService
        )
    }

    private static func makeBalanceService(
        balanceStore: BalanceStore,
        assetsService: AssetsService,
        chainFactory: ChainServiceFactory
    ) -> BalanceService {
        BalanceService(
            balanceStore: balanceStore,
            assetsService: assetsService,
            chainServiceFactory: chainFactory
        )
    }

    private static func makeStakeService(
        stakeStore: StakeStore,
        addressStore: AddressStore,
        chainFactory: ChainServiceFactory
    ) -> StakeService {
        StakeService(
            store: stakeStore,
            addressStore: addressStore,
            chainServiceFactory: chainFactory
        )
    }

    private static func makeAssetsService(
        assetStore: AssetStore,
        balanceStore: BalanceStore,
        chainFactory: ChainServiceFactory
    ) -> AssetsService {
        AssetsService(
            assetStore: assetStore,
            balanceStore: balanceStore,
            chainServiceFactory: chainFactory
        )
    }

    private static func makeTransactionsService(
        apiService: GemAPIService,
        transactionStore: TransactionStore,
        assetsService: AssetsService,
        walletStore: WalletStore,
        deviceService: any DeviceServiceable,
        addressStore: AddressStore
    ) -> TransactionsService {
        TransactionsService(
            provider: apiService,
            transactionStore: transactionStore,
            assetsService: assetsService,
            walletStore: walletStore,
            deviceService: deviceService,
            addressStore: addressStore
        )
    }

    private static func makeTransactionService(
        transactionStore: TransactionStore,
        nativeProvider: NativeProvider,
        stakeService: StakeService,
        earnService: EarnService,
        nftService: NFTService,
        chainFactory: ChainServiceFactory,
        balanceService: BalanceService
    ) -> TransactionStateService {
        TransactionStateService(
            transactionStore: transactionStore,
            swapper: GemSwapper(rpcProvider: nativeProvider),
            stakeService: stakeService,
            earnService: earnService,
            nftService: nftService,
            chainServiceFactory: chainFactory,
            balanceUpdater: balanceService
        )
    }

    private static func makeBannerService(
        bannerStore: BannerStore,
        pushNotificationEnablerService: PushNotificationEnablerService
    ) -> BannerService {
        BannerService(
            store: bannerStore,
            pushNotificationService: pushNotificationEnablerService
        )
    }

    private static func makePriceAlertService(
        apiService: GemAPIService,
        priceAlertStore: PriceAlertStore,
        deviceService: DeviceService,
        priceUpdater: any PriceUpdater,
        preferences: Preferences
    ) -> PriceAlertService {
        PriceAlertService(
            store: priceAlertStore,
            apiService: apiService,
            deviceService: deviceService,
            priceUpdater: priceUpdater,
            preferences: preferences
        )
    }

    private static func makeConnectionsService(
        connectionsStore: ConnectionsStore,
        walletSessionService: WalletSessionService,
        interactor: any WalletConnectorInteractable
    ) -> ConnectionsService {
        ConnectionsService(
            store: connectionsStore,
            signer: WalletConnectorSigner(
                connectionsStore: connectionsStore,
                walletSessionService: walletSessionService,
                walletConnectorInteractor: interactor
            )
        )
    }

    private static func makeOnstartService(
        assetListService: any GemAPIAssetsListService,
        assetStore: AssetStore,
        nodeStore: NodeStore,
        preferences: Preferences,
        assetsService: AssetsService,
        walletService: WalletService
    ) -> OnstartService {
        OnstartService(
            assetListService: assetListService,
            assetsService: assetsService,
            assetStore: assetStore,
            nodeStore: nodeStore,
            preferences: preferences,
            walletService: walletService
        )
    }

    private static func makeOnstartAsyncService(
        apiService: GemAPIService,
        nodeService: NodeService,
        preferences: Preferences,
        assetsService: AssetsService,
        bannerSetupService: BannerSetupService,
        configService: ConfigService,
        swappableChainsProvider: any SwappableChainsProvider
    ) -> OnstartAsyncService {
        let importAssetsService = ImportAssetsService(
            assetListService: apiService,
            assetsService: assetsService,
            assetStore: assetsService.assetStore,
            preferences: preferences
        )

        return OnstartAsyncService(
            runners: [
                ConfigUpdateRunner(configService: configService),
                BannerSetupRunner(bannerSetupService: bannerSetupService),
                NodeImportRunner(nodeService: nodeService),
                AssetsUpdateRunner(
                    configService: configService,
                    importAssetsService: importAssetsService,
                    assetsService: assetsService,
                    swappableChainsProvider: swappableChainsProvider,
                    preferences: preferences
                ),
            ]
        )
    }

    private static func makeOnstartWalletService(
        preferences: Preferences,
        deviceService: DeviceService,
        bannerSetupService: BannerSetupService,
        addressStatusService: AddressStatusService,
        pushNotificationEnablerService: PushNotificationEnablerService
    ) -> OnstartWalletService {
        OnstartWalletService(
            preferences: preferences,
            deviceService: deviceService,
            bannerSetupService: bannerSetupService,
            addressStatusService: addressStatusService,
            pushNotificationEnablerService: pushNotificationEnablerService
        )
    }
    
    private static func makeNftService(
        apiService: GemAPIService,
        nftStore: NFTStore,
        deviceService: any DeviceServiceable
    ) -> NFTService {
        NFTService(
            apiService: apiService,
            nftStore: nftStore,
            deviceService: deviceService
        )
    }
    
    private static func makePerpetualService(
        perpetualStore: PerpetualStore,
        assetStore: AssetStore,
        priceStore: PriceStore,
        balanceStore: BalanceStore,
        nodeProvider: any NodeURLFetchable,
        preferences: Preferences
    ) -> PerpetualService {
        PerpetualService(
            store: perpetualStore,
            assetStore: assetStore,
            priceStore: priceStore,
            balanceStore: balanceStore,
            provider: PerpetualProviderFactory(nodeProvider: nodeProvider).createProvider(),
            preferences: preferences
        )
    }

    private static func makeWebSocket(securePreferences: SecurePreferences) -> any WebSocketConnectable {
        let requestProvider = AuthenticatedRequestProvider(securePreferences: securePreferences)
        let configuration = WebSocketConfiguration(requestProvider: requestProvider)
        return WebSocketConnection(configuration: configuration)
    }

}
