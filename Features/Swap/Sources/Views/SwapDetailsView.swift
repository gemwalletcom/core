// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Components
import PrimitivesComponents
import InfoSheet
import Localization
import Primitives
import Style

public struct SwapDetailsView: View {
    @Environment(\.dismiss) private var dismiss
    @Bindable private var model: SwapDetailsViewModel

    public init(model: Bindable<SwapDetailsViewModel>) {
        _model = model
    }

    public var body: some View {
        VStack {
            switch model.state {
            case .data: listView
            case .error(let error): List { ListItemErrorView(errorTitle: Localized.Errors.errorOccured, error: error.asAnyError(asset: model.fromAsset)) }
            case .loading: LoadingView()
            case .noData: List { ListItemErrorView(errorTitle: nil, error: AnyError(Localized.Errors.errorOccured)) }
            }
        }
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Button("", systemImage: SystemImage.checkmark, action: { dismiss() })
            }
        }
        .navigationTitle(Localized.Common.details)
        .navigationBarTitleDisplayMode(.inline)
        .listSectionSpacing(.compact)
        .contentMargins([.top], .extraSmall, for: .scrollContent)
        .sheet(item: $model.isPresentingInfoSheet) {
            InfoSheetScene(type: $0)
        }
        .sheet(isPresented: $model.isPresentingSwapProviderSelectionSheet) {
            SelectableListNavigationStack(
                model: model.swapProvidersViewModel,
                onFinishSelection: model.onFinishSwapProviderSelection,
                listContent: { SimpleListItemView(model: $0) }
            )
        }
    }

    private var listView: some View {
        List {
            Section {
                let view = SimpleListItemView(model: model.selectedProviderItem)
                if model.allowSelectProvider {
                    NavigationCustomLink(
                        with: view
                    ) {
                        model.onSelectProvidersSelection()
                    }
                } else {
                    view
                }
            } header: {
                Text(Localized.Common.provider)
                    .listRowInsets(.horizontalMediumInsets)
            }
            
            Section {
                if let rateText = model.rateText {
                    ListItemRotateView(
                        title: model.rateTitle,
                        subtitle: rateText,
                        action: model.switchRateDirection
                    )
                }

                if let swapEstimationField = model.swapEstimationField {
                    ListItemView(field: swapEstimationField)
                }

                PriceImpactView(
                    model: model.priceImpactModel,
                    infoAction: model.onSelectPriceImpactInfoSheet
                )

                ListItemView(field: model.minReceiveField)

                ListItemView(
                    field: model.slippageField,
                    infoAction: model.onSelectSlippageInfoSheet
                )
            }
        }
    }
}

