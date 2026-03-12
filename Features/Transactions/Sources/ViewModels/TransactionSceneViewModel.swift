// Copyright (c). Gem Wallet. All rights reserved.

import Components
import ExplorerService
import Foundation
import InfoSheet
import Preferences
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class TransactionSceneViewModel {
    private let preferences: Preferences
    private let explorerService: ExplorerService

    public let query: ObservableQuery<TransactionRequest>
    var transactionExtended: TransactionExtended { query.value }
    var isPresentingTransactionSheet: TransactionSheetType?

    public init(
        transaction: TransactionExtended,
        walletId: WalletId,
        preferences: Preferences = Preferences.standard,
        explorerService: ExplorerService = ExplorerService.standard
    ) {
        self.preferences = preferences
        self.explorerService = explorerService
        self.query = ObservableQuery(TransactionRequest(walletId: walletId, transactionId: transaction.id), initialValue: transaction)
    }

    var title: String { model.titleTextValue.text }
    var explorerURL: URL { explorerViewModel.url }
}

// MAKR: - ListSectionProvideable

extension TransactionSceneViewModel: ListSectionProvideable {
    public var sections: [ListSection<TransactionItem>] {
        [
            ListSection(type: .header, [.header]),
            ListSection(type: .swapAction, [.swapButton]),
            ListSection(type: .details, [.date, .status, .participant, .memo, .network, .pnl, .price, .provider, .fee]),
            ListSection(type: .explorer, [.explorerLink])
        ]
    }

    public func itemModel(for item: TransactionItem) -> any ItemModelProvidable<TransactionItemModel> {
        switch item {
        case .header: headerViewModel
        case .swapButton: TransactionSwapButtonViewModel(metadata: model.transaction.transaction.metadata?.decode(TransactionSwapMetadata.self), state: model.transaction.transaction.state)
        case .date: TransactionDateViewModel(date: model.transaction.transaction.createdAt)
        case .status: TransactionStatusViewModel(state: model.transaction.transaction.state, onInfoAction: onSelectStatusInfo)
        case .participant: TransactionParticipantViewModel(transactionViewModel: model)
        case .memo: TransactionMemoViewModel(transaction: model.transaction.transaction)
        case .network: TransactionNetworkViewModel(chain: model.transaction.asset.chain)
        case .pnl: TransactionPnlViewModel(metadata: model.transaction.transaction.metadata?.decode(TransactionPerpetualMetadata.self))
        case .price: TransactionPriceViewModel(metadata: model.transaction.transaction.metadata?.decode(TransactionPerpetualMetadata.self))
        case .provider: TransactionProviderViewModel(metadata: model.transaction.transaction.metadata?.decode(TransactionSwapMetadata.self))
        case .fee: TransactionNetworkFeeViewModel(feeDisplay: model.infoModel.feeDisplay, onInfoAction: onSelectFee)
        case .explorerLink: TransactionExplorerViewModel(transactionViewModel: model, explorerService: explorerService)
        }
    }
}

// MARK: - Actions

extension TransactionSceneViewModel {
    func onSelectTransactionHeader() {
        if let headerLink = headerViewModel.headerLink {
            UIApplication.shared.open(headerLink)
        }
    }

    func onSelectShare() {
        isPresentingTransactionSheet = .share
    }

    func onSelectFeeDetails() {
        isPresentingTransactionSheet = .feeDetails
    }

    private func onSelectFee() {
        let chain = model.transaction.transaction.assetId.chain
        isPresentingTransactionSheet = .info(.networkFee(chain))
    }

    private func onSelectStatusInfo() {
        let assetImage = model.assetImage
        isPresentingTransactionSheet = .info(.transactionState(
            imageURL: assetImage.imageURL,
            placeholder: assetImage.placeholder,
            state: model.transaction.transaction.state
        ))
    }
}

// MARK: - Private

extension TransactionSceneViewModel {
    private var model: TransactionViewModel {
        TransactionViewModel(
            explorerService: ExplorerService.standard,
            transaction: transactionExtended,
            currency: preferences.currency
        )
    }
    
    private var headerViewModel: TransactionHeaderViewModel {
        TransactionHeaderViewModel(
            transaction: model.transaction,
            infoModel: model.infoModel
        )
    }

    private var explorerViewModel: TransactionExplorerViewModel {
        TransactionExplorerViewModel(
            transactionViewModel: model,
            explorerService: explorerService
        )
    }

    var feeDetailsViewModel: NetworkFeeSceneViewModel {
        NetworkFeeSceneViewModel(
            chain: model.transaction.transaction.assetId.chain,
            priority: .normal,
            value: model.infoModel.feeDisplay?.amount.text,
            fiatValue: model.infoModel.feeDisplay?.fiat?.text
        )
    }
}
