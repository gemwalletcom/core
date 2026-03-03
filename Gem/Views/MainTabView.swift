// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives
import Store
import Localization
import Style
import NFT
import TransactionsService
import WalletTab
import Transactions
import Assets
import PriceAlerts
import Components

struct MainTabView: View {
    @Environment(\.assetDiscoveryService) private var assetDiscoveryService
    @Environment(\.balanceService) private var balanceService
    @Environment(\.bannerService) private var bannerService
    @Environment(\.navigationState) private var navigationState
    @Environment(\.navigationPresenter) private var presenter
    @Environment(\.nftService) private var nftService
    @Environment(\.priceService) private var priceService
    @Environment(\.observablePreferences) private var observablePreferences
    @Environment(\.walletService) private var walletService
    @Environment(\.assetsService) private var assetsService
    @Environment(\.priceAlertService) private var priceAlertService
    @Environment(\.transactionsService) private var transactionsService

    @State private var model: MainTabViewModel

    private var tabViewSelection: Binding<TabItem> {
        Binding(
            get: { navigationState.selectedTab },
            set: { onSelect(tab: $0) }
        )
    }

    init(model: MainTabViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        TabView(selection: tabViewSelection) {
            WalletNavigationStack(
                model: WalletSceneViewModel(
                    assetDiscoveryService: assetDiscoveryService,
                    balanceService: balanceService,
                    bannerService: bannerService,
                    walletService: walletService,
                    observablePreferences: observablePreferences,
                    wallet: model.wallet,
                    isPresentingSelectedAssetInput: presenter.isPresentingAssetInput
                )
            )
            .tabItem {
                tabItem(Localized.Wallet.title, Images.Tabs.wallet)
            }
            .tag(TabItem.wallet)
            
            if model.isMarketEnabled {
                MarketsNavigationStack()
                .tabItem {
                    tabItem("Markets", Images.Tabs.markets)
                }
                .tag(TabItem.markets)
            }
            
            if model.isCollectionsEnabled {
                CollectionsNavigationStack(
                    model: CollectionsViewModel(
                        nftService: nftService,
                        walletService: walletService,
                        wallet: model.wallet
                    ),
                    isPresentingSelectedAssetInput: presenter.isPresentingAssetInput
                )
                .tabItem {
                    tabItem(Localized.Nft.collections, Images.Tabs.collections)
                }
                .tag(TabItem.collections)
            }
            
            TransactionsNavigationStack(
                model: TransactionsViewModel(
                    transactionsService: transactionsService,
                    walletService: walletService,
                    wallet: model.wallet,
                    type: .all
                )
            )
            .tabItem {
                tabItem(Localized.Activity.title, Images.Tabs.activity)
            }
            .badge(model.transactions)
            .tag(TabItem.activity)

            SettingsNavigationStack(
                walletId: model.wallet.walletId,
                priceService: priceService,
                isPresentingSupport: presenter.isPresentingSupport
            )
            .tabItem {
                tabItem(Localized.Settings.title, Images.Tabs.settings)
            }
            .tag(TabItem.settings)
        }
        .sheet(item: presenter.isPresentingAssetInput) { input in
            SelectedAssetNavigationStack(
                input: input,
                wallet: model.wallet,
                onComplete: { onComplete(type: input.type) }
            )
        }
        .sheet(item: presenter.isPresentingPriceAlert) { input in
            SetPriceAlertNavigationStack(
                model: SetPriceAlertViewModel(
                    walletId: model.wallet.walletId,
                    asset: input.asset,
                    priceAlertService: priceAlertService,
                    price: input.price,
                    onComplete: onSetPriceAlertComplete
                )
            )
        }
        .toast(message: $model.isPresentingToastMessage)
        .bindQuery(model.transactionsQuery)
        .onChange(of: walletService.currentWallet, model.onChangeWallet)
    }
}

// MARK: - UI Components

extension MainTabView {
    @ViewBuilder
    private func tabItem(_ title: String, _ image: Image) -> Label<Text, Image> {
        Label(
            title: { Text(title) },
            icon: { image }
        )
    }
}

// MARK: - Actions

extension MainTabView {
    private func onSelect(tab: TabItem) {
        navigationState.select(tab: tab)
    }

    private func onSetPriceAlertComplete(message: String) {
        presenter.isPresentingPriceAlert.wrappedValue = nil
        model.isPresentingToastMessage = .priceAlert(message: message)
    }

    private func onComplete(type: SelectedAssetType) {
        switch type {
        case .receive, .stake, .earn, .buy, .sell:
            presenter.isPresentingAssetInput.wrappedValue = nil
        case let .send(type):
            switch type {
            case .nft:
                if navigationState.selectedTab == .collections {
                    navigationState.collections.reset()
                    navigationState.activity.reset()
                    navigationState.selectedTab = .activity
                }
            case .asset:
                break
            }
            presenter.isPresentingAssetInput.wrappedValue = nil
        case let .swap(fromAsset, _):
            Task {
                let asset = try await assetsService.getOrFetchAsset(for: fromAsset.id)

                switch navigationState.selectedTab {
                case .wallet:
                    navigationState.wallet.setPath([Scenes.Asset(asset: asset)])
                case .activity:
                    navigationState.wallet.setPath([Scenes.Asset(asset: asset)])
                    navigationState.selectedTab = .wallet
                case .markets, .settings, .collections:
                    break
                }
                presenter.isPresentingAssetInput.wrappedValue = nil
            }
        }
    }
}

