// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Style
import Localization
import InfoSheet
import PrimitivesComponents
import FiatConnect
import Swap
import Preferences
import Primitives

public struct ConfirmTransferScene: View {
    @Environment(\.fiatService) private var fiatService
    @State private var model: ConfirmTransferSceneViewModel

    public init(model: ConfirmTransferSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        ListSectionView(
            provider: model,
            content: content(for:)
        )
        .contentMargins([.top], .small, for: .scrollContent)
        .listSectionSpacing(.compact)
        .safeAreaButton {
            StateButton(model.confirmButtonModel)
        }
        .frame(maxWidth: .infinity)
        .debounce(
            value: model.feeModel.priority,
            interval: .none,
            action: model.onChangeFeePriority
        )
        .taskOnce { model.fetch() }
        .navigationTitle(model.title)
        // TODO: - move to navigation view
        .navigationBarTitleDisplayMode(.inline)
        .activityIndicator(isLoading: model.confirmingState.isLoading, message: model.progressMessage)
        .sheet(item: $model.isPresentingSheet) {
            switch $0 {
            case .info(let type):
                InfoSheetScene(type: type)
            case .url(let url):
                SFSafariView(url: url)
            case .networkFeeSelector:
                NetworkFeeSheet(model: model.feeModel)
            case .payloadDetails:
                NavigationStack {
                    SimulationPayloadDetailsScene(
                        primaryFields: model.primaryPayloadFields,
                        secondaryFields: model.secondaryPayloadFields,
                        fieldViewModel: model.payloadFieldViewModel(for:),
                        contextMenuItems: model.contextMenuItems(for:)
                    )
                    .presentationDetents([.large])
                    .presentationBackground(Colors.grayBackground)
                }
            case .fiatConnect(let assetAddress, let walletId):
                NavigationStack {
                    FiatConnectNavigationView(
                        model: FiatSceneViewModel(
                            fiatService: fiatService,
                            assetAddress: assetAddress,
                            walletId: walletId
                        )
                    )
                    .navigationBarTitleDisplayMode(.inline)
                    .toolbarDismissItem(type: .close, placement: .topBarLeading)
                }
            case .swapDetails:
                if case let .swapDetails(model) = model.detailsViewModel.itemModel {
                    NavigationStack {
                        SwapDetailsView(model: Bindable(model))
                            .presentationDetentsForCurrentDeviceSize(expandable: true)
                            .presentationBackground(Colors.grayBackground)
                    }
                }
            case .perpetualDetails(let model):
                NavigationStack {
                    PerpetualDetailsView(model: model)
                        .presentationDetentsForCurrentDeviceSize(expandable: true)
                        .presentationBackground(Colors.grayBackground)
                }
            }
        }
        .alertSheet($model.isPresentingAlertMessage)
    }
}

// MARK: - UI Components

extension ConfirmTransferScene {

    @ViewBuilder
    private func content(for itemModel: ConfirmTransferItemModel) -> some View {
        switch itemModel {
        case let .header(model):
            TransactionHeaderListItemView(
                headerType: model.headerType,
                showClearHeader: model.showClearHeader
            )
        case let .app(model):
            ListItemImageView(model: model)
                .contextMenu(
                    .url(title: self.model.websiteTitle, onOpen: self.model.onSelectOpenWebsiteURL)
                )
        case let .sender(model):
            ListItemImageView(model: model)
                .contextMenu([
                    .copy(value: self.model.senderAddress),
                    .url(title: self.model.senderExplorerText, onOpen: self.model.onSelectOpenSenderAddressURL)
                ])
        case let .recipient(model):
            AddressListItemView(model: model)
        case let .network(model):
            ListItemImageView(model: model)
        case let .memo(model):
            ListItemView(model: model)
                .contextMenu( model.subtitle.map ({ [.copy(value: $0)] }) ?? [] )
        case .swapDetails(let model):
            NavigationCustomLink(
                with: SwapDetailsListView(model: model),
                action: { self.model.onSelectSwapDetails() }
            )
        case .perpetualDetails(let model):
            NavigationCustomLink(
                with: ListItemView(model: model.listItemModel),
                action: { self.model.onSelectPerpetualDetails(model) }
            )
        case .perpetualModifyPosition(let model):
            ListItemView(model: model.listItemModel)
        case let .networkFee(model, selectable):
            if selectable {
                NavigationCustomLink(
                    with: ListItemView(model: model),
                    action: self.model.onSelectFeePicker
                )
            } else {
                ListItemView(model: model)
            }
        case let .warnings(warnings):
            SimulationWarningsContent(warnings: warnings)
        case let .payload(fields):
            Group {
                SimulationPayloadFieldsContent(
                    fields: fields,
                    fieldViewModel: self.model.payloadFieldViewModel(for:),
                    contextMenuItems: self.model.contextMenuItems(for:)
                )

                if self.model.hasPayloadDetails {
                    NavigationCustomLink(
                        with: ListItemView(title: Localized.Common.details),
                        action: self.model.onSelectPayloadDetails
                    )
                }
            }
        case let .error(title, error, onInfoAction):
            ListItemErrorView(
                errorTitle: title,
                error: error,
                infoAction: onInfoAction
            )
        case .empty:
            EmptyView()
        }
    }
}
