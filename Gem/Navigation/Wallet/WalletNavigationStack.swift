// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives
import PrimitivesComponents
import Store
import MarketInsight
import Transactions
import WalletTab
import InfoSheet
import Components
import Assets
import Perpetuals
import Transfer
import StakeService
import PriceAlerts
import AssetsService
import PriceService

struct WalletNavigationStack: View {
    @Environment(\.assetsEnabler) private var assetsEnabler
    @Environment(\.balanceService) private var balanceService
    @Environment(\.navigationState) private var navigationState
    @Environment(\.priceService) private var priceService
    @Environment(\.portfolioService) private var portfolioService
    @Environment(\.priceAlertService) private var priceAlertService
    @Environment(\.assetsService) private var assetsService
    @Environment(\.transactionsService) private var transactionsService
    @Environment(\.bannerService) private var bannerService
    @Environment(\.streamSubscriptionService) private var streamSubscriptionService
    @Environment(\.perpetualService) private var perpetualService
    @Environment(\.hyperliquidObserverService) private var hyperliquidObserverService
    @Environment(\.activityService) private var activityService
    @Environment(\.walletSearchService) private var walletSearchService
    @Environment(\.assetSearchService) private var assetSearchService
    @Environment(\.observablePreferences) private var preferences

    @State private var model: WalletSceneViewModel

    init(model: WalletSceneViewModel) {
        _model = State(initialValue: model)
    }

    private var navigationPath: Binding<NavigationPath> {
        navigationState.wallet.binding
    }

    var body: some View {
        NavigationStack(path: navigationPath) {
            ZStack {
                WalletScene(model: model)
                    .opacity(model.isPresentingSearch ? 0 : 1)

                if model.isPresentingSearch {
                    WalletSearchScene(
                        model: WalletSearchSceneViewModel(
                            wallet: model.wallet,
                            searchService: walletSearchService,
                            activityService: activityService,
                            assetsEnabler: assetsEnabler,
                            balanceService: balanceService,
                            perpetualService: perpetualService,
                            onDismissSearch: model.onToggleSearch,
                            onSelectAssetAction: onSelectAsset,
                            onAddToken: model.onSelectAddCustomToken
                        )
                    )
                    .transition(.opacity)
                }
            }
            .onChange(of: model.currentWallet, model.onChangeWallet)
            .onChange(of: navigationState.walletTabReselected, model.onWalletTabReselected)
            .bindQuery(model.assetsQuery, model.bannersQuery, model.totalFiatQuery)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                if !model.isPresentingSearch {
                    ToolbarItem(placement: .principal) {
                        WalletBarView(
                            model: model.walletBarModel,
                            action: model.onSelectWalletBar
                        )
                        .liquidGlass()
                    }
                    ToolbarItem(placement: .navigationBarTrailing) {
                        Button(action: model.onToggleSearch) {
                            model.searchImage
                        }
                    }
                }
            }
            .navigationDestination(for: Scenes.Asset.self) {
                AssetNavigationView(
                    model: AssetSceneViewModel(
                        assetsEnabler: assetsEnabler,
                        balanceService: balanceService,
                        assetsService: assetsService,
                        transactionsService: transactionsService,
                        priceUpdater: streamSubscriptionService,
                        priceAlertService: priceAlertService,
                        bannerService: bannerService,
                        input: AssetSceneInput(
                            wallet: model.wallet,
                            asset: $0.asset
                        ),
                        isPresentingSelectedAssetInput: model.isPresentingSelectedAssetInput
                    )
                )
            }
            .navigationDestination(for: Scenes.Transaction.self) {
                TransactionNavigationView(
                    model: TransactionSceneViewModel(
                        transaction: $0.transaction,
                        walletId: model.wallet.walletId
                    )
                )
            }
            .navigationDestination(for: Scenes.Price.self) {
                ChartScene(
                    model: ChartSceneViewModel(
                        priceService: priceService,
                        assetModel: AssetViewModel(asset: $0.asset),
                        priceAlertService: priceAlertService,
                        walletId: model.wallet.walletId,
                        onSetPriceAlert: model.presentPriceAlert
                    )
                )
            }
            .navigationDestination(for: Scenes.Perpetuals.self) { _ in
                PerpetualsNavigationView(
                    wallet: model.wallet,
                    perpetualService: perpetualService,
                    observerService: hyperliquidObserverService,
                    activityService: activityService,
                    onSelectAssetType: { model.isPresentingSheet = .selectAsset($0) },
                    onSelectAsset: { navigationState.wallet.append(Scenes.Perpetual($0)) }
                )
            }
            .navigationDestination(for: Scenes.AssetsResults.self) { destination in
                AssetsResultsScene(
                    model: AssetsResultsSceneViewModel(
                        wallet: model.wallet,
                        assetsEnabler: assetsEnabler,
                        balanceService: balanceService,
                        preferences: preferences.preferences,
                        request: WalletSearchRequest(
                            walletId: model.wallet.walletId,
                            searchBy: destination.searchQuery,
                            tag: destination.tag,
                            limit: AssetsResultsSceneViewModel.defaultLimit
                        ),
                        onSelectAsset: onSelectAsset
                    )
                )
            }
            .navigationDestination(for: Scenes.Perpetual.self) {
                PerpetualNavigationView(
                    asset: $0.asset,
                    wallet: model.wallet,
                    perpetualService: perpetualService,
                    transactionsService: transactionsService,
                    observerService: hyperliquidObserverService,
                    isPresentingSheet: $model.isPresentingSheet
                )
            }
            .navigationDestination(for: Scenes.AssetPriceAlert.self) {
                AssetPriceAlertsScene(
                    model: AssetPriceAlertsViewModel(
                        priceAlertService: priceAlertService,
                        walletId: model.wallet.walletId,
                        asset: $0.asset
                    )
                )
            }
            .sheet(item: $model.isPresentingSheet) { sheet in
                switch sheet {
                case .wallets:
                    WalletsNavigationStack()
                case .selectAsset(let type):
                    SelectAssetSceneNavigationStack(
                        model: SelectAssetViewModel(
                            wallet: model.wallet,
                            selectType: type,
                            searchService: assetSearchService,
                            assetsEnabler: assetsEnabler,
                            priceAlertService: priceAlertService,
                            activityService: activityService
                        )
                    )
                case .infoSheet(let type):
                    InfoSheetScene(type: type)
                case .transferData(let data):
                    ConfirmTransferNavigationStack(
                        wallet: model.wallet,
                        transferData: data,
                        onComplete: model.onTransferComplete
                    )
                case .perpetualRecipientData(let data):
                    PerpetualPositionNavigationStack(
                        perpetualRecipientData: data,
                        wallet: model.wallet,
                        onComplete: { model.isPresentingSheet = nil }
                    )
                case .setPriceAlert(let asset):
                    SetPriceAlertNavigationStack(
                        model: SetPriceAlertViewModel(
                            walletId: model.wallet.walletId,
                            asset: asset,
                            priceAlertService: priceAlertService
                        ) { model.onSetPriceAlertComplete(message: $0) }
                    )
                case .addAsset:
                    AddAssetNavigationStack(wallet: model.wallet)
                case .portfolio:
                    WalletPortfolioScene(
                        model: WalletPortfolioSceneViewModel(
                            wallet: model.wallet,
                            portfolioService: portfolioService,
                            priceService: priceService,
                            currencyCode: preferences.preferences.currency
                        )
                    )
                }
            }
            .safariSheet(url: $model.isPresentingUrl)
        }
        .toast(message: $model.isPresentingToastMessage)
    }

    private func onSelectAsset(asset: Asset) {
        if asset.type == .perpetual {
            navigationState.wallet.append(Scenes.Perpetual(asset))
        } else {
            navigationState.wallet.append(Scenes.Asset(asset: asset))
        }
    }
}
