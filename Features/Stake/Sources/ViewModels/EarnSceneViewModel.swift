// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Foundation
import Localization
import Primitives
import Store
import Style
import EarnService
import PrimitivesComponents

@MainActor
@Observable
public final class EarnSceneViewModel {
    private let earnService: EarnService
    private var viewState: StateViewType<Bool> = .loading

    public let wallet: Wallet
    public let asset: Asset
    private let currencyCode: String

    public let assetQuery: ObservableQuery<AssetRequest>
    public let positionsQuery: ObservableQuery<DelegationsRequest>
    public let providersQuery: ObservableQuery<ValidatorsRequest>

    public var assetData: AssetData { assetQuery.value }
    public var positions: [Delegation] { positionsQuery.value }
    public var providers: [DelegationValidator] { providersQuery.value }

    public init(
        wallet: Wallet,
        asset: Asset,
        currencyCode: String,
        earnService: EarnService
    ) {
        self.wallet = wallet
        self.asset = asset
        self.currencyCode = currencyCode
        self.earnService = earnService
        self.assetQuery = ObservableQuery(AssetRequest(walletId: wallet.walletId, assetId: asset.id), initialValue: .with(asset: asset))
        self.positionsQuery = ObservableQuery(
            DelegationsRequest(walletId: wallet.walletId, assetId: asset.id, providerType: .earn),
            initialValue: []
        )
        self.providersQuery = ObservableQuery(
            ValidatorsRequest(chain: asset.id.chain, providerType: .earn),
            initialValue: []
        )
    }

    var title: String { Localized.Common.earn }
    var assetTitle: String { AssetViewModel(asset: asset).title }


    private var apr: Double? {
        providers.first.map(\.apr).flatMap { $0 > 0 ? $0 : nil }
            ?? assetData.metadata.earnApr
    }

    var aprModel: AprViewModel {
        AprViewModel(apr: apr ?? .zero)
    }

    var showDeposit: Bool {
        wallet.canSign && providers.isNotEmpty
    }

    var depositDestination: AmountInput? {
        guard let provider = providers.first else { return nil }
        return AmountInput(
            type: .earn(.deposit(provider)),
            asset: asset
        )
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .earn(symbol: asset.symbol))
    }

    var positionModels: [DelegationViewModel] {
        positions
            .filter { (BigInt($0.base.balance) ?? .zero) > 0 }
            .map { DelegationViewModel(delegation: $0, asset: asset, currencyCode: currencyCode) }
    }

    var hasPositions: Bool {
        positionModels.isNotEmpty
    }

    var showEmptyState: Bool {
        !hasPositions && !viewState.isLoading
    }

    var positionsSectionTitle: String {
        hasPositions ? Localized.Perpetual.positions : .empty
    }

    var providersState: StateViewType<Bool> {
        switch viewState {
        case .noData: .noData
        case .loading: providers.isEmpty ? .loading : .data(true)
        case .data: providers.isEmpty ? .noData : .data(true)
        case .error(let error): .error(error)
        }
    }
}

// MARK: - Actions

extension EarnSceneViewModel {
    func fetch() async {
        viewState = .loading
        do {
            let address = try wallet.account(for: asset.id.chain).address
            try await earnService.update(
                walletId: wallet.walletId,
                assetId: asset.id,
                address: address
            )
            viewState = .data(true)
        } catch {
            viewState = .error(error)
        }
    }
}
