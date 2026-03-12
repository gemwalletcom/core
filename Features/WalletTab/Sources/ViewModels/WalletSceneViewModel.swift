// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Style
import Primitives
import BalanceService
import BannerService
import DiscoverAssetsService
import Preferences
import Store
import Localization
import PrimitivesComponents
import InfoSheet
import Components
import WalletService
import Formatters

@Observable
@MainActor
public final class WalletSceneViewModel: Sendable {
    private let assetDiscoveryService: any AssetDiscoverable
    private let balanceService: BalanceService
    private let bannerService: BannerService
    private let walletService: WalletService

    let observablePreferences: ObservablePreferences

    public var wallet: Wallet

    // db queries
    public let totalFiatQuery: ObservableQuery<TotalValueRequest>
    public let assetsQuery: ObservableQuery<AssetsRequest>
    public let bannersQuery: ObservableQuery<BannersRequest>

    // db observed values
    public var totalFiatValue: TotalFiatValue { totalFiatQuery.value }
    public var assets: [AssetData] { assetsQuery.value }
    public var banners: [Banner] { bannersQuery.value }

    // TODO: - separate presenting sheet state logic to separate type
    public var isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>
    public var isPresentingWallets = false
    public var isPresentingSelectAssetType: SelectAssetType?
    public var isPresentingInfoSheet: InfoSheetType?
    public var isPresentingUrl: URL? = nil
    public var isPresentingTransferData: TransferData?
    public var isPresentingPerpetualRecipientData: PerpetualRecipientData?
    public var isPresentingSetPriceAlert: Asset?
    public var isPresentingToastMessage: ToastMessage?
    public var isPresentingSearch = false
    public var isPresentingAddToken: Bool = false
    public var isPresentingPortfolio = false

    public var isLoadingAssets: Bool = false

    public init(
        assetDiscoveryService: any AssetDiscoverable,
        balanceService: BalanceService,
        bannerService: BannerService,
        walletService: WalletService,
        observablePreferences: ObservablePreferences,
        wallet: Wallet,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>
    ) {
        self.wallet = wallet
        self.assetDiscoveryService = assetDiscoveryService
        self.balanceService = balanceService
        self.bannerService = bannerService
        self.walletService = walletService
        self.observablePreferences = observablePreferences

        self.totalFiatQuery = ObservableQuery(TotalValueRequest(walletId: wallet.walletId, balanceType: .wallet), initialValue: .zero)
        self.assetsQuery = ObservableQuery(AssetsRequest(walletId: wallet.walletId, filters: [.enabledBalance]), initialValue: [])
        self.bannersQuery = ObservableQuery(BannersRequest(walletId: wallet.walletId, assetId: .none, chain: .none, events: [.accountBlockedMultiSignature, .onboarding]), initialValue: [])
        self.isPresentingSelectedAssetInput = isPresentingSelectedAssetInput
    }

    public var currentWallet: Wallet? { walletService.currentWallet }

    var manageTokenTitle: String { Localized.Wallet.manageTokenList }
    var perpetualsTitle: String { Localized.Perpetuals.title }

    public var searchImage: Image { Images.System.search }
    public var manageImage: Image { Images.Actions.manage }

    var showPinnedSection: Bool {
        !sections.pinned.isEmpty
    }

    var showPerpetuals: Bool {
        observablePreferences.isPerpetualEnabled && wallet.isMultiCoins
    }

    var currencyCode: String {
        observablePreferences.preferences.currency
    }

    var sections: AssetsSections {
        AssetsSections.from(assets)
    }

    public var walletBarModel: WalletBarViewViewModel {
        let walletModel = WalletViewModel(wallet: wallet)
        return WalletBarViewViewModel(
            name: walletModel.name,
            image: walletModel.avatarImage
        )
    }

    var walletHeaderModel: WalletHeaderViewModel {
        WalletHeaderViewModel(
            walletType: wallet.type,
            totalValue: totalFiatValue,
            currencyCode: currencyCode,
            bannerEventsViewModel: HeaderBannerEventViewModel(events: banners.map(\.event))
        )
    }

    var walletBannersModel: WalletSceneBannersViewModel {
        WalletSceneBannersViewModel(
            banners: banners,
            totalFiatValue: totalFiatValue.value
        )
    }

}

// MARK: - Business Logic

extension WalletSceneViewModel {
    func fetch() {
        Task {
            shouldStartLoadingAssets()
            await fetch(wallet: wallet, assetIds: assets.map { $0.asset.id })
            isLoadingAssets = false
        }
    }
    
    public func onSelectWalletBar() {
        isPresentingWallets.toggle()
    }

    public func onSelectManage() {
        isPresentingSelectAssetType = .manage
    }

    public func onToggleSearch() {
        isPresentingSearch.toggle()
    }

    public func onSelectAddCustomToken() {
        isPresentingAddToken = true
    }

    func onSelectPortfolio() {
        isPresentingPortfolio = true
    }

    func onHeaderAction(type: HeaderButtonType) {
        let selectType: SelectAssetType = switch type {
        case .buy: .buy
        case .send: .send
        case .receive: .receive(.asset)
        case .sell, .swap, .more, .stake, .deposit, .withdraw:
            fatalError()
        }
        isPresentingSelectAssetType = selectType
    }

    func onCloseBanner(banner: Banner) {
        bannerService.onClose(banner)
    }

    func onSelectWatchWalletInfo() {
        isPresentingInfoSheet = .watchWallet
    }

    func onBanner(action: BannerAction) {
        switch action.type {
        case .event, .closeBanner:
            Task {
                try await handleBanner(action: action)
            }
        case .button(let bannerButton):
            switch bannerButton {
            case .buy: isPresentingSelectAssetType = .buy
            case .receive: isPresentingSelectAssetType = .receive(.asset)
            }
        }
        isPresentingUrl = action.url
    }

    func onHideAsset(_ assetId: AssetId) {
        do {
            try balanceService.hideAsset(walletId: wallet.walletId, assetId: assetId)
        } catch {
            debugLog("WalletSceneViewModel hide Asset error: \(error)")
        }
    }

    func onPinAsset(_ asset: Asset, value: Bool) {
        do {
            try balanceService.setPinned(value, walletId: wallet.walletId, assetId: asset.id)
            isPresentingToastMessage = .pin(asset.name, pinned: value)
        } catch {
            debugLog("WalletSceneViewModel pin asset error: \(error)")
        }
    }

    func onCopyAddress(_ message: String) {
        isPresentingToastMessage = .copy(message)
    }

    public func onChangeWallet(_ oldWallet: Wallet?, _ newWallet: Wallet?) {
        guard let newWallet else { return }

        if wallet.walletId != newWallet.walletId {
            refresh(for: newWallet)
        } else if wallet != newWallet {
            wallet = newWallet
        }
    }

    public func onWalletTabReselected(_: Bool, _: Bool) {
         isPresentingSearch = false
    }
    
    func shouldStartLoadingAssets() {
        let preferences = WalletPreferences(walletId: wallet.walletId)
        isLoadingAssets = !preferences.completeInitialLoadAssets && preferences.assetsTimestamp == .zero
    }
    
    public func onTransferComplete() {
        isPresentingTransferData = nil
    }
    
    public func onSetPriceAlertComplete(message: String) {
        isPresentingSetPriceAlert = nil
        isPresentingToastMessage = .priceAlert(message: message)
    }
}

// MARK: - Private

extension WalletSceneViewModel {
    private func fetch(wallet: Wallet, assetIds: [AssetId]) async {
        async let balance: () = balanceService.updateBalance(for: wallet, assetIds: assetIds)
        async let discovery: () = discoverAssets(wallet: wallet)
        _ = await (balance, discovery)
    }

    private func discoverAssets(wallet: Wallet) async {
        do {
            try await assetDiscoveryService.discoverAssets(wallet: wallet)
        } catch {
            debugLog("WalletSceneViewModel discoverAssets error: \(error)")
        }
    }

    private func refresh(for newWallet: Wallet) {
        wallet = newWallet
        totalFiatQuery.request.walletId = newWallet.walletId
        assetsQuery.request.walletId = newWallet.walletId
        bannersQuery.request.walletId = newWallet.walletId

        fetch()
    }

    private func handleBanner(action: BannerAction) async throws {
        try await bannerService.handleAction(action)
    }
}
