// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import BannerService
import ChainService
import DeviceService
import PriceAlertService
import StakeService
import NodeService
import PriceService
import StreamService
import WalletConnector
import ConnectionsService
import ExplorerService
import BalanceService
import AssetsService
import TransactionsService
import TransactionStateService
import DiscoverAssetsService
import WalletService
import WalletSessionService
import AppService
import ScanService
import NFTService
import AvatarService
import SwapService
import NameService
import PerpetualService
import AddressNameService
import ActivityService
import RewardsService
import EventPresenterService
import NotificationService
import GemAPI
import ContactService

extension AppResolver {
    struct Services: Sendable {
        // Environment-level services
        let assetsService: AssetsService
        let balanceService: BalanceService
        let bannerService: BannerService
        let chainServiceFactory: ChainServiceFactory
        let connectionsService: ConnectionsService
        let deviceService: DeviceService
        let nodeService: NodeService
        let navigationHandler: NavigationHandler
        let navigationPresenter: NavigationPresenter
        let priceAlertService: PriceAlertService
        let priceService: PriceService
        let streamObserverService: StreamObserverService
        let streamSubscriptionService: StreamSubscriptionService
        let stakeService: StakeService
        let transactionsService: TransactionsService
        let transactionStateService: TransactionStateService
        let walletService: WalletService
        let walletSessionService: any WalletSessionManageable
        let assetsEnabler: any AssetsEnabler
        let assetDiscoveryService: any AssetDiscoverable
        let walletSetupService: WalletSetupService
        let explorerService: ExplorerService
        let scanService: ScanService
        let nftService: NFTService
        let avatarService: AvatarService
        let swapService: SwapService
        let subscriptionsService: SubscriptionService
        let appReleaseService: AppReleaseService
        let releaseAlertService: ReleaseAlertService
        let rateService: RateService
        let deviceObserverService: DeviceObserverService
        let onstartService: OnstartService
        let onstartAsyncService: OnstartAsyncService
        let onstartWalletService: OnstartWalletService
        let walletConnectorManager: WalletConnectorManager
        let perpetualService: PerpetualService
        let hyperliquidObserverService: any PerpetualObservable<HyperliquidSubscription>
        let nameService: NameService
        let addressNameService: AddressNameService
        let activityService: ActivityService
        let eventPresenterService: EventPresenterService
        let viewModelFactory: ViewModelFactory
        let rewardsService: RewardsService
        let walletSearchService: WalletSearchService
        let assetSearchService: AssetSearchService
        let appLifecycleService: AppLifecycleService
        let inAppNotificationService: InAppNotificationService
        let portfolioService: PortfolioService
        let fiatService: any GemAPIFiatService
        let contactService: ContactService

        init(
            assetsService: AssetsService,
            balanceService: BalanceService,
            bannerService: BannerService,
            chainServiceFactory: ChainServiceFactory,
            connectionsService: ConnectionsService,
            deviceService: DeviceService,
            nodeService: NodeService,
            navigationHandler: NavigationHandler,
            navigationPresenter: NavigationPresenter,
            priceAlertService: PriceAlertService,
            streamObserverService: StreamObserverService,
            streamSubscriptionService: StreamSubscriptionService,
            priceService: PriceService,
            stakeService: StakeService,
            transactionsService: TransactionsService,
            transactionStateService: TransactionStateService,
            walletService: WalletService,
            walletSessionService: any WalletSessionManageable,
            assetsEnabler: any AssetsEnabler,
            assetDiscoveryService: any AssetDiscoverable,
            walletSetupService: WalletSetupService,
            explorerService: ExplorerService,
            scanService: ScanService,
            nftService: NFTService,
            avatarService: AvatarService,
            swapService: SwapService,
            appReleaseService: AppReleaseService,
            releaseAlertService: ReleaseAlertService,
            rateService: RateService,
            subscriptionsService: SubscriptionService,
            deviceObserverService: DeviceObserverService,
            onstartService: OnstartService,
            onstartAsyncService: OnstartAsyncService,
            onstartWalletService: OnstartWalletService,
            walletConnectorManager: WalletConnectorManager,
            perpetualService: PerpetualService,
            hyperliquidObserverService: any PerpetualObservable<HyperliquidSubscription>,
            nameService: NameService,
            addressNameService: AddressNameService,
            activityService: ActivityService,
            eventPresenterService: EventPresenterService,
            viewModelFactory: ViewModelFactory,
            rewardsService: RewardsService,
            walletSearchService: WalletSearchService,
            assetSearchService: AssetSearchService,
            appLifecycleService: AppLifecycleService,
            inAppNotificationService: InAppNotificationService,
            portfolioService: PortfolioService,
            fiatService: any GemAPIFiatService,
            contactService: ContactService
        ) {
            self.assetsService = assetsService
            self.balanceService = balanceService
            self.bannerService = bannerService
            self.chainServiceFactory = chainServiceFactory
            self.connectionsService = connectionsService
            self.deviceService = deviceService
            self.nodeService = nodeService
            self.navigationHandler = navigationHandler
            self.navigationPresenter = navigationPresenter
            self.priceAlertService = priceAlertService
            self.priceService = priceService
            self.streamObserverService = streamObserverService
            self.streamSubscriptionService = streamSubscriptionService
            self.stakeService = stakeService
            self.transactionsService = transactionsService
            self.transactionStateService = transactionStateService
            self.walletService = walletService
            self.walletSessionService = walletSessionService
            self.assetsEnabler = assetsEnabler
            self.assetDiscoveryService = assetDiscoveryService

            self.walletSetupService = walletSetupService
            self.explorerService = explorerService
            self.scanService = scanService
            self.nftService = nftService
            self.avatarService = avatarService
            self.swapService = swapService
            self.appReleaseService = appReleaseService
            self.releaseAlertService = releaseAlertService
            self.rateService = rateService
            self.deviceObserverService = deviceObserverService
            self.subscriptionsService = subscriptionsService
            self.onstartService = onstartService
            self.onstartAsyncService = onstartAsyncService
            self.onstartWalletService = onstartWalletService
            self.walletConnectorManager = walletConnectorManager
            self.perpetualService = perpetualService
            self.hyperliquidObserverService = hyperliquidObserverService
            self.nameService = nameService
            self.addressNameService = addressNameService
            self.activityService = activityService
            self.eventPresenterService = eventPresenterService
            self.viewModelFactory = viewModelFactory
            self.rewardsService = rewardsService
            self.walletSearchService = walletSearchService
            self.assetSearchService = assetSearchService
            self.appLifecycleService = appLifecycleService
            self.inAppNotificationService = inAppNotificationService
            self.portfolioService = portfolioService
            self.fiatService = fiatService
            self.contactService = contactService
        }
    }
}
