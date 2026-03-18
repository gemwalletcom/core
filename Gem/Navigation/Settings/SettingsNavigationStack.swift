// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives
import Store
import PrimitivesComponents
import PriceAlerts
import WalletConnector
import Preferences
import MarketInsight
import Settings
import PriceService
import RewardsService
import InAppNotifications
import NotificationService
import ContactService
import Contacts
import WalletSessionService

struct SettingsNavigationStack: View {
    @Environment(\.navigationState) private var navigationState
    @Environment(\.deviceService) private var deviceService
    @Environment(\.transactionsService) private var transactionsService
    @Environment(\.assetsService) private var assetsService
    @Environment(\.stakeService) private var stakeService
    @Environment(\.bannerService) private var bannerService
    @Environment(\.connectionsService) private var connectionsService
    @Environment(\.assetsEnabler) private var assetsEnabler
    @Environment(\.walletService) private var walletService
    @Environment(\.walletSessionService) private var walletSessionService
    @Environment(\.priceAlertService) private var priceAlertService
    @Environment(\.priceService) private var priceService
    @Environment(\.nodeService) private var nodeService
    @Environment(\.chainServiceFactory) private var chainServiceFactory
    @Environment(\.observablePreferences) private var observablePreferences
    @Environment(\.releaseService) private var releaseService
    @Environment(\.perpetualService) private var perpetualService
    @Environment(\.walletConnectorManager) private var walletConnectorManager
    @Environment(\.rewardsService) private var rewardsService
    @Environment(\.inAppNotificationService) private var inAppNotificationService
    @Environment(\.contactService) private var contactService
    @Environment(\.nameService) private var nameService

    @State private var isPresentingWallets = false
    @State private var currencyModel: CurrencySceneViewModel

    let walletId: WalletId
    @Binding var isPresentingSupport: Bool

    init(
        walletId: WalletId,
        preferences: Preferences = .standard,
        priceService: PriceService,
        isPresentingSupport: Binding<Bool>
    ) {
        self.walletId = walletId
        _isPresentingSupport = isPresentingSupport
        _currencyModel = State(
            initialValue: CurrencySceneViewModel(
                currencyStorage: preferences,
                priceService: priceService
            )
        )
    }

    private var navigationPath: Binding<NavigationPath> {
        navigationState.settings.binding
    }

    var body: some View {
        NavigationStack(path: navigationPath) {
            SettingsScene(
                model: SettingsViewModel(
                    walletId: walletId,
                    walletSessionService: walletSessionService,
                    observablePrefereces: observablePreferences
                ),
                isPresentingWallets: $isPresentingWallets,
                isPresentingSupport: $isPresentingSupport,
                deviceId: (try? SecurePreferences.standard.getDeviceId()) ?? ""
            )
            .navigationBarTitleDisplayMode(.inline)
            .navigationDestination(for: Scenes.Security.self) { _ in
                SecurityScene(model: SecurityViewModel())
            }
            .navigationDestination(for: Scenes.Notifications.self) { _ in
                NotificationsScene(
                    model: NotificationsViewModel(
                        deviceService: deviceService,
                        bannerService: bannerService
                    )
                )
            }
            .navigationDestination(for: Scenes.PriceAlerts.self) { _ in
                PriceAlertsNavigationView(
                    model: PriceAlertsSceneViewModel(priceAlertService: priceAlertService)
                )
            }
            .navigationDestination(for: Scenes.AssetPriceAlert.self) {
                AssetPriceAlertsScene(
                    model: AssetPriceAlertsViewModel(
                        priceAlertService: priceAlertService,
                        walletId: walletId,
                        asset: $0.asset
                    )
                )
            }
            .navigationDestination(for: Scenes.Price.self) { scene in
                ChartScene(
                    model: ChartSceneViewModel(
                        priceService: priceService,
                        assetModel: AssetViewModel(asset: scene.asset),
                        priceAlertService: priceAlertService,
                        walletId: walletId,
                        onSetPriceAlert: { _ in }
                    )
                )
            }
            .navigationDestination(for: Scenes.Chains.self) { _ in
                ChainListSettingsScene()
            }
            .navigationDestination(for: Scenes.AboutUs.self) { _ in
                AboutUsScene(
                    model: AboutUsViewModel(
                        preferences: observablePreferences,
                        releaseService: releaseService
                    )
                )
            }
            .navigationDestination(for: Scenes.WalletConnect.self) { _ in
                ConnectionsScene(
                    model: ConnectionsViewModel(
                        service: connectionsService,
                        walletConnectorPresenter: walletConnectorManager.presenter
                    )
                )
            }
            .navigationDestination(for: Scenes.Developer.self) { _ in
                DeveloperScene(model: DeveloperViewModel(
                    walletId: walletId,
                    transactionsService: transactionsService,
                    assetService: assetsService,
                    stakeService: stakeService,
                    bannerService: bannerService,
                    priceService: priceService,
                    perpetualService: perpetualService
                ))
            }
            .navigationDestination(for: Scenes.InAppNotifications.self) { _ in
                if let wallet = walletService.currentWallet {
                    InAppNotificationsScene(
                        model: InAppNotificationsViewModel(
                            wallet: wallet,
                            notificationService: inAppNotificationService
                        )
                    )
                }
            }
            .navigationDestination(for: Scenes.Currency.self) { _ in
                CurrencyScene(model: currencyModel)
            }
            .navigationDestination(for: Scenes.Preferences.self) { _ in
                PreferencesScene(model: PreferencesViewModel(currencyModel: currencyModel))
            }
            .navigationDestination(for: Scenes.AppIcon.self) { _ in
                AppIconScene(model: AppIconSceneViewModel())
            }
            .navigationDestination(for: Scenes.Referral.self) { scene in
                let wallets = walletService.wallets.filter { $0.type == .multicoin }
                if let wallet = wallets.first(where: { $0.id == walletService.currentWallet?.id }) ?? wallets.first {
                    RewardsScene(
                        model: RewardsViewModel(
                            rewardsService: rewardsService,
                            assetsEnabler: assetsEnabler,
                            wallet: wallet,
                            wallets: wallets,
                            activateCode: scene.code,
                            giftCode: scene.giftCode
                        )
                    )
                }
            }
            .navigationDestination(for: Scenes.ChainSettings.self) {
                ChainSettingsScene(
                    model: ChainSettingsSceneViewModel(
                        nodeService: nodeService,
                        chainServiceFactory: chainServiceFactory,
                        chain: $0.chain
                    )
                )
            }
            .navigationDestination(for: Scenes.Contacts.self) { _ in
                ContactsNavigationView(
                    model: ContactsViewModel(service: contactService, nameService: nameService),
                    navigationPath: navigationPath
                )
            }
            .sheet(isPresented: $isPresentingWallets) {
                WalletsNavigationStack()
            }
        }
        .onChange(of: currencyModel.selectedCurrencyValue) { _, _ in
            navigationState.settings.removeLast()
        }
    }
}

extension Preferences: @retroactive CurrencyStorable {}
