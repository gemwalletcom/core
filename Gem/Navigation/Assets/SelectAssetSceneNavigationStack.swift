// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives
import Components
import Style
import FiatConnect
import PrimitivesComponents
import Assets
import Transfer
import Recents

struct SelectAssetSceneNavigationStack: View {
    @Environment(\.viewModelFactory) private var viewModelFactory
    @Environment(\.assetsEnabler) private var assetsEnabler
    @Environment(\.activityService) private var activityService

    @State private var isPresentingFilteringView: Bool = false

    @State private var model: SelectAssetViewModel
    @State private var navigationPath = NavigationPath()
    @Binding private var isPresentingSelectAssetType: SelectAssetType?

    init(
        model: SelectAssetViewModel,
        isPresentingSelectType: Binding<SelectAssetType?>
    ) {
        _model = State(wrappedValue: model)
        _isPresentingSelectAssetType = isPresentingSelectType
    }

    var body: some View {
        NavigationStack(path: $navigationPath) {
            SelectAssetScene(
                model: model
            )
            .onChange(of: model.assetSelection, onChangeAssetSelection)
            .toolbar {
                ToolbarDismissItem(
                    type: .close,
                    placement: .topBarLeading
                )
                if model.showFilter {
                    ToolbarItem(placement: .navigationBarTrailing) {
                        FilterButton(
                            isActive: model.filterModel.isAnyFilterSpecified,
                            action: onSelectFilter
                        )
                    }
                }
                if model.showAddToken {
                    ToolbarItem(placement: .navigationBarTrailing) {
                        Button {
                            model.isPresentingAddToken = true
                        } label: {
                            Images.System.plus
                        }
                    }
                }
            }
            .navigationDestination(for: SelectAssetInput.self) { input in
                Group {
                    switch input.type {
                    case .send:
                        RecipientNavigationView(
                            model: viewModelFactory.recipientScene(
                                wallet: model.wallet,
                                asset: input.asset,
                                type: .asset(input.asset),
                                onRecipientDataAction: {
                                    navigationPath.append($0)
                                },
                                onTransferAction: {
                                    navigationPath.append($0)
                                }
                            )
                        )
                    case .receive:
                        ReceiveScene(
                            model: ReceiveViewModel(
                                assetModel: AssetViewModel(asset: input.asset),
                                wallet: model.wallet,
                                address: input.assetAddress.address,
                                assetsEnabler: assetsEnabler
                            )
                        )
                    case .buy:
                        FiatConnectNavigationView(
                            model: viewModelFactory.fiatScene(
                                assetAddress: input.assetAddress,
                                walletId: model.wallet.walletId
                            )
                        )
                    case .deposit:
                        AmountNavigationView(
                            model: viewModelFactory.amountScene(
                                input: AmountInput(
                                    type: .deposit(
                                        recipient: RecipientData(
                                            recipient: .hyperliquid,
                                            amount: .none
                                        )
                                    ),
                                    asset: input.asset
                                ),
                                wallet: model.wallet,
                                onTransferAction: {
                                    navigationPath.append($0)
                                }
                            )
                        )
                    case .withdraw:
                        let withdrawRecipient = Recipient(
                            name: model.wallet.name,
                            address: input.assetAddress.address,
                            memo: nil
                        )
                        AmountNavigationView(
                            model: viewModelFactory.amountScene(
                                input: AmountInput(
                                    type: .withdraw(
                                        recipient: RecipientData(
                                            recipient: withdrawRecipient,
                                            amount: .none
                                        )
                                    ),
                                    asset: input.asset
                                ),
                                wallet: model.wallet,
                                onTransferAction: {
                                    navigationPath.append($0)
                                }
                            )
                        )
                    case .manage, .priceAlert, .swap:
                        EmptyView()
                    }
                }
            }
            .navigationBarTitleDisplayMode(.inline)
            .navigationDestination(for: TransferData.self) { data in
                ConfirmTransferScene(
                    model: viewModelFactory.confirmTransferScene(
                        wallet: model.wallet,
                        data: data,
                        onComplete: {
                            isPresentingSelectAssetType = nil
                        }
                    )
                )
            }
        }
        .sheet(isPresented: $model.isPresentingAddToken) {
            AddTokenNavigationStack(
                wallet: model.wallet,
                isPresenting: $model.isPresentingAddToken
            )
        }
        .sheet(isPresented: $isPresentingFilteringView) {
            NavigationStack {
                AssetsFilterScene(model: $model.filterModel)
            }
            .presentationDetentsForCurrentDeviceSize(expandable: true)
            .presentationDragIndicator(.visible)
            .presentationBackground(Colors.grayBackground)
        }
        .sheet(isPresented: $model.isPresentingRecents) {
            RecentsScene(
                model: RecentsSceneViewModel(
                    walletId: model.wallet.walletId,
                    types: model.recentsQuery.request.types,
                    filters: model.recentsQuery.request.filters,
                    activityService: activityService,
                    onSelect: model.onSelectRecent
                )
            )
        }
    }
}

// MARK: - Actions

extension SelectAssetSceneNavigationStack {
    private func onSelectFilter() {
        isPresentingFilteringView.toggle()
    }

    private func onChangeAssetSelection(_: AssetSelectionType?, new: AssetSelectionType?) {
        if let new {
            model.assetSelection = nil
            switch new {
            case .regular(let input):
                model.updateRecent(assetId: input.asset.id)
                navigationPath.append(input)
            case .recent(let input):
                navigationPath.append(input)
            }
        }
    }
}
