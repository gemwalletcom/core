// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Primitives
import Store
import Components
import Localization
import PrimitivesComponents
import AssetsService
import BalanceService
import Preferences
import PriceAlertService
import ActivityService
import Style

@Observable
@MainActor
public final class SelectAssetViewModel {
    let preferences: Preferences
    let selectType: SelectAssetType
    let searchService: AssetSearchService
    let assetsEnabler: any AssetsEnabler
    let priceAlertService: PriceAlertService
    let activityService: ActivityService

    public let wallet: Wallet

    var state: StateViewType<[AssetBasic]> = .noData
    var searchModel: AssetSearchViewModel

    public let assetsQuery: ObservableQuery<AssetsRequest>
    public let recentsQuery: ObservableQuery<RecentActivityRequest>
    var assets: [AssetData] { assetsQuery.value }
    var recents: [RecentAsset] { recentsQuery.value }

    var isSearching: Bool = false
    var isDismissSearch: Bool = false
    var isPresentingCopyToast: Bool = false
    var copyTypeViewModel: CopyTypeViewModel?

    public var isPresentingAddToken: Bool = false
    public var isPresentingRecents: Bool = false
    public var assetSelection: AssetSelectionType?

    public var filterModel: AssetsFilterViewModel
    public var onSelectAssetAction: AssetAction

    public init(
        preferences: Preferences = Preferences.standard,
        wallet: Wallet,
        selectType: SelectAssetType,
        searchService: AssetSearchService,
        assetsEnabler: any AssetsEnabler,
        priceAlertService: PriceAlertService,
        activityService: ActivityService,
        selectAssetAction: AssetAction = .none
    ) {
        self.preferences = preferences
        self.wallet = wallet
        self.selectType = selectType
        self.searchService = searchService
        self.assetsEnabler = assetsEnabler
        self.priceAlertService = priceAlertService
        self.activityService = activityService
        self.onSelectAssetAction = selectAssetAction

        let filter = AssetsFilterViewModel(
            type: selectType,
            model: ChainsFilterViewModel(
                chains: wallet.chains
            )
        )
        self.filterModel = filter
        self.searchModel = AssetSearchViewModel(selectType: selectType)

        self.assetsQuery = ObservableQuery(AssetsRequest(walletId: wallet.walletId, filters: filter.filters), initialValue: [])
        self.recentsQuery = ObservableQuery(
            RecentActivityRequest(
                walletId: wallet.walletId,
                limit: 10,
                types: RecentActivityType.allCases.filter { $0 != .perpetual
                },
                filters: filter.defaultFilters),
            initialValue: []
        )
    }

    var title: String {
        switch selectType {
        case .send: Localized.Wallet.send
        case .receive(let type):
            switch type {
            case .asset: Localized.Wallet.receive
            case .collection: Localized.Wallet.receiveCollection
            }
        case .buy: Localized.Wallet.buy
        case .swap(let type):
            switch type {
            case .pay: Localized.Swap.youPay
            case .receive: Localized.Swap.youReceive
            }
        case .manage: Localized.Wallet.manageTokenList
        case .priceAlert: Localized.Assets.selectAsset
        case .deposit: Localized.Wallet.deposit
        case .withdraw: Localized.Wallet.withdraw
        }
    }

    var sections: AssetsSections {
        AssetsSections.from(assets)
    }

    var enablePopularSection: Bool {
        [.buy, .priceAlert].contains(selectType)
    }

    var showPopularSection: Bool {
        enablePopularSection && sections.popular.isNotEmpty
    }

    var showPinnedSection: Bool {
        sections.pinned.isNotEmpty
    }

    var showAssetsSection: Bool {
        sections.assets.isNotEmpty
    }

    var popularImage: Image { Images.System.starFill }
    var popularTitle: String { Localized.Common.popular }

    var pinnedImage: Image { Images.System.pin }
    var pinnedTitle: String { Localized.Common.pinned }

    var assetsTitle: String {
        switch selectType {
        case .send, .buy, .swap, .manage, .priceAlert, .deposit, .withdraw, .receive(.asset):
            Localized.Assets.title
        case .receive(.collection):
            Localized.Settings.Networks.title
        }
    }

    public var showAddToken: Bool {
        selectType == .manage && wallet.hasTokenSupport && !filterModel.chainsFilter.isEmpty
    }

    public var showFilter: Bool {
        switch selectType {
        case .receive(let type):
            switch type {
            case .asset:
                wallet.isMultiCoins && !filterModel.chainsFilter.isEmpty
            case .collection: false
            }
        case .buy, .manage, .priceAlert, .send, .swap:
            wallet.isMultiCoins && !filterModel.chainsFilter.isEmpty
        case .deposit, .withdraw: false
        }
    }

    var isNetworkSearchEnabled: Bool {
        switch selectType {
        case .manage, .receive, .buy, .priceAlert: return true
        case let .swap(type):
            switch type {
            case .pay: return false
            case .receive: return true
            }
        case .send, .deposit, .withdraw: return false
        }
    }

    var showTags: Bool {
        !isSearching && searchModel.searchableQuery.isEmpty
    }

    var showLoading: Bool {
        state.isLoading && showEmpty
    }

    var showEmpty: Bool {
        sections.pinned.isEmpty && sections.assets.isEmpty
    }

    var showRecents: Bool {
        switch selectType {
        case .send, .receive, .buy, .swap: searchModel.searchableQuery.isEmpty && recents.isNotEmpty
        case .manage, .priceAlert, .deposit, .withdraw: false
        }
    }

    var recentModels: [AssetViewModel] {
        recents.map { AssetViewModel(asset: $0.asset) }
    }

    var currencyCode: String {
        preferences.currency
    }
}

// MARK: - Business Logic

extension SelectAssetViewModel {
    public func updateRecent(assetId: AssetId) {
        guard let data = selectType.recentActivityData(assetId: assetId) else { return }
        do {
            try activityService.updateRecent(data: data, walletId: wallet.walletId)
        } catch {
            debugLog("Failed to update recent activity: \(error)")
        }
    }

    func selectAsset(asset: Asset) {
        switch selectType {
        case .priceAlert:
            Task {
                await setPriceAlert(assetId: asset.id, enabled: true)
            }
        case .manage, .send, .receive, .buy, .swap, .deposit, .withdraw: break
        }
        onSelectAssetAction?(asset)
    }

    func search(query: String) async {
        let query = query.trim()
        if query.isEmpty {
            return
        }
        await searchAssets(
            query: query,
            priorityAssetsQuery: searchModel.priorityAssetsQuery,
            tag: nil
        )
    }

    func handleAction(assetId: AssetId, enabled: Bool) async {
        switch selectType {
        case .manage:
            do {
                try await assetsEnabler.enableAssets(wallet: wallet, assetIds: [assetId], enabled: enabled)
            } catch {
                debugLog("SelectAssetViewModel handleAction error: \(error)")
            }
        case .send, .receive, .buy, .swap, .priceAlert, .deposit, .withdraw: break
        }
    }

    func setSelected(tag: AssetTagSelection) {
        isDismissSearch.toggle()
        searchModel.tagsViewModel.selectedTag = tag
        searchModel.focus = .tags
        updateRequest()
        Task {
            await searchAssets(
                query: .empty,
                priorityAssetsQuery: searchModel.priorityAssetsQuery,
                tag: searchModel.tagsViewModel.selectedTag.tag
            )
        }
    }

    func updateRequest() {
        assetsQuery.request.searchBy = searchModel.priorityAssetsQuery.or(.empty)
        state = isNetworkSearchEnabled ? .loading : .noData
    }

    func onChangeFocus(_: Bool, isSearchable: Bool) {
        if isSearchable {
            searchModel.focus = .search
            searchModel.tagsViewModel.selectedTag = .all
            updateRequest()
        }
    }

    func onChangeFilterModel(_: AssetsFilterViewModel, model: AssetsFilterViewModel) {
        assetsQuery.request.filters = model.filters
    }
}

// MARK: - Actions

extension SelectAssetViewModel {
    func onAssetAction(action: ListAssetItemAction, assetData: AssetData) {
        let asset = assetData.asset
        switch action {
        case .switcher(let enabled):
            Task {
                await handleAction(assetId: asset.id, enabled: enabled)
            }
        case .copy:
            let address = assetData.account.address
            copyTypeViewModel = CopyTypeViewModel(
                type: .address(asset, address: address),
                copyValue: address
            )
            isPresentingCopyToast = true
            Task {
                await handleAction(assetId: asset.id, enabled: true)
            }
        }
    }

    func onSelectRecents() {
        isPresentingRecents = true
    }

    func onSelectAsset(_ assetData: AssetData) {
        assetSelection = .regular(SelectAssetInput(type: selectType, assetAddress: assetData.assetAddress))
    }

    public func onSelectRecent(_ asset: Asset) {
        switch selectType {
        case .send, .receive, .buy:
            assetSelection = .recent(SelectAssetInput(type: selectType, assetAddress: assetAddress(for: asset)))
        case .swap:
            selectAsset(asset: asset)
        case .manage, .priceAlert, .deposit, .withdraw:
            break
        }
        isPresentingRecents = false
    }

    func onSelectAddCustomToken() {
        isPresentingAddToken.toggle()
    }
}

// MARK: - Private

extension SelectAssetViewModel {

    private func assetAddress(for asset: Asset) -> AssetAddress {
        let address: String = {
            do {
                return try wallet.account(for: asset.chain).address
            } catch {
                debugLog(error.localizedDescription)
                return ""
            }
        }()
        return AssetAddress(asset: asset, address: address)
    }
    
    private func searchAssets(
        query: String,
        priorityAssetsQuery: String?,
        tag: AssetTag?
    ) async {
        do {
            let assets = try await searchService.searchAssets(
                wallet: wallet,
                query: query,
                priorityAssetsQuery: priorityAssetsQuery,
                tag: tag
            )
            state = .data(assets)
        } catch {
            handle(error: error)
        }
    }

    private func setPriceAlert(assetId: AssetId, enabled: Bool) async {
        do {
            let currency = Preferences.standard.currency
            if enabled {
                try await priceAlertService.add(priceAlert: .default(for: assetId, currency: currency))
            } else {
                try await priceAlertService.delete(priceAlerts: [.default(for: assetId, currency: currency)])
            }
        } catch {
            handle(error: error)
        }
    }

    private func handle(error: any Error) {
        state.setError(error)
        debugLog("SelectAssetScene scene error: \(error)")
    }
}

// MARK: - Models extensions

extension SelectAssetType {
    var listType: AssetListType {
        switch self {
        case .send,
                .buy,
                .swap,
                .deposit,
                .withdraw: .view
        case .receive(let type): .copy(type)
        case .manage: .manage
        case .priceAlert: .price
        }
    }
}
