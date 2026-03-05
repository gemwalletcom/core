// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import GRDB
import Store
import Keystore
import BannerService
import NotificationService
import DeviceService
import PriceAlertService
import GemAPI
import ChainService
import StakeService
import NodeService
import PriceService
import StreamService
import WalletConnector
import ConnectionsService
import ExplorerService
import NFTService
import BalanceService
import AssetsService
import TransactionsService
import DiscoverAssetsService
import WalletService
import WalletSessionService
import AvatarService
import AppService
import ScanService
import NameService
import PerpetualService
import TransactionStateService
import AddressNameService
import ActivityService
import RewardsService
import EventPresenterService
import ContactService

extension EnvironmentValues {
    @Entry var navigationState: NavigationStateManager = AppResolver.main.navigation
    @Entry var keystore: any Keystore = AppResolver.main.storages.keystore
    @Entry var nodeService: NodeService = AppResolver.main.services.nodeService
    @Entry var priceService: PriceService = AppResolver.main.services.priceService
    @Entry var streamSubscriptionService: StreamSubscriptionService = AppResolver.main.services.streamSubscriptionService
    @Entry var explorerService: ExplorerService = AppResolver.main.services.explorerService
    @Entry var assetsEnabler: any AssetsEnabler = AppResolver.main.services.assetsEnabler
    @Entry var assetDiscoveryService: any AssetDiscoverable = AppResolver.main.services.assetDiscoveryService
    @Entry var walletService: WalletService = AppResolver.main.services.walletService
    @Entry var walletSessionService: any WalletSessionManageable = AppResolver.main.services.walletSessionService
    @Entry var priceAlertService: PriceAlertService = AppResolver.main.services.priceAlertService
    @Entry var deviceService: DeviceService = AppResolver.main.services.deviceService
    @Entry var balanceService: BalanceService = AppResolver.main.services.balanceService
    @Entry var bannerService: BannerService = AppResolver.main.services.bannerService
    @Entry var transactionsService: TransactionsService =  AppResolver.main.services.transactionsService
    @Entry var assetsService: AssetsService = AppResolver.main.services.assetsService
    @Entry var navigationPresenter: NavigationPresenter = AppResolver.main.services.navigationPresenter
    @Entry var stakeService: StakeService = AppResolver.main.services.stakeService
    @Entry var connectionsService: ConnectionsService = AppResolver.main.services.connectionsService
    @Entry var walletConnectorManager: WalletConnectorManager = AppResolver.main.services.walletConnectorManager
    @Entry var chainServiceFactory: ChainServiceFactory = AppResolver.main.services.chainServiceFactory
    @Entry var nftService: NFTService = AppResolver.main.services.nftService
    @Entry var avatarService: AvatarService = AppResolver.main.services.avatarService
    @Entry var releaseService: AppReleaseService = AppResolver.main.services.appReleaseService
    @Entry var scanService: ScanService = AppResolver.main.services.scanService
    @Entry var perpetualService: PerpetualService = AppResolver.main.services.perpetualService
    @Entry var hyperliquidObserverService: any PerpetualObservable<HyperliquidSubscription> = AppResolver.main.services.hyperliquidObserverService
    @Entry var transactionStateService: TransactionStateService = AppResolver.main.services.transactionStateService
    @Entry var nameService: NameService = AppResolver.main.services.nameService
    @Entry var addressNameService: AddressNameService = AppResolver.main.services.addressNameService
    @Entry var activityService: ActivityService = AppResolver.main.services.activityService
    @Entry var eventPresenterService: EventPresenterService = AppResolver.main.services.eventPresenterService
    @Entry var viewModelFactory: ViewModelFactory = AppResolver.main.services.viewModelFactory
    @Entry var rewardsService: RewardsService = AppResolver.main.services.rewardsService
    @Entry var walletSearchService: WalletSearchService = AppResolver.main.services.walletSearchService
    @Entry var assetSearchService: AssetSearchService = AppResolver.main.services.assetSearchService
    @Entry var inAppNotificationService: InAppNotificationService = AppResolver.main.services.inAppNotificationService
    @Entry var contactService: ContactService = AppResolver.main.services.contactService
}
