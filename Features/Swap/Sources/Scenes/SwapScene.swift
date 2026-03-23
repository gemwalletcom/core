// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Style
import PrimitivesComponents

public struct SwapScene: View {
    @FocusState private var focusedField: Bool

    @State private var model: SwapSceneViewModel

    public init(model: SwapSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            swapFromSectionView
            swapToSectionView
            if model.shouldShowAdditionalInfo {
                additionalInfoSectionView
            }

            if let error = model.swapState.error {
                Section {
                    ListItemErrorView(
                        errorTitle: model.errorTitle,
                        error: error.asAnyError(asset: model.fromAsset?.asset),
                        infoAction: model.errorInfoAction
                    )
                }
            }
        }
        .listSectionSpacing(.compact)
        .safeAreaView {
            bottomActionView
                .confirmationDialog(
                    model.swapDetailsViewModel?.highImpactWarningTitle ?? "",
                    presenting: $model.isPresentingPriceImpactConfirmation,
                    sensoryFeedback: .warning,
                    actions: { _ in
                        Button(
                            model.buttonViewModel.title,
                            role: .destructive,
                            action: model.onSelectSwapConfirmation
                        )
                    },
                    message: {
                        Text(model.isPresentingPriceImpactConfirmation ?? "")
                    }
                )
        }
        .navigationTitle(model.title)
        .onChangeBindQuery(model.fromAssetQuery, action: model.onChangeFromAsset)
        .onChangeBindQuery(model.toAssetQuery, action: model.onChangeToAsset)
        .debouncedTask(id: model.fetchTrigger) {
            await model.fetch()
        }
        .debounce(
            value: model.assetIds,
            initial: true,
            interval: .none,
            action: model.onAssetIdsChange
        )
        .onChange(of: model.amountInputModel.text, model.onChangeFromValue)
        .onChange(of: model.pairSelectorModel, model.onChangePair)
        .onChange(of: model.selectedSwapQuote, model.onChangeSwapQuoute)
        .onTimer(every: 30, id: model.fetchTrigger) {
            await model.fetch()
        }
        .onAppear {
            focusedField = true
        }
    }
}

// MARK: - UI Components

extension SwapScene {
    private var swapFromSectionView: some View {
        Section {
            SwapTokenView(
                model: model.swapTokenModel(type: .pay),
                text: $model.amountInputModel.text,
                onBalanceAction: model.onSelectFromMaxBalance,
                onSelectAssetAction: model.onSelectAssetPay
            )
            .buttonStyle(.borderless)
            .focused($focusedField)
        } header: {
            Text(model.swapFromTitle)
                .listRowInsets(.horizontalMediumInsets)
        } footer: {
            SwapChangeView(
                fromId: $model.pairSelectorModel.fromAssetId,
                toId: $model.pairSelectorModel.toAssetId
            )
                .padding(.top, .small)
                .frame(maxWidth: .infinity)
                .disabled(model.isSwitchAssetButtonDisabled)
                .textCase(nil)
                .listRowSeparator(.hidden)
                .listRowInsets(.horizontalMediumInsets)
        }
    }

    private var swapToSectionView: some View {
        Section {
            SwapTokenView(
                model: model.swapTokenModel(type: .receive(chains: [], assetIds: [])),
                text: $model.toValue,
                showLoading: model.isLoading,
                disabledTextField: true,
                onBalanceAction: {},
                onSelectAssetAction: model.onSelectAssetReceive
            )
            .buttonStyle(.borderless)
        } header: {
            Text(model.swapToTitle)
                .listRowInsets(.horizontalMediumInsets)
        }
    }

    private var additionalInfoSectionView: some View {
        Section {
            if let swapDetailsViewModel = model.swapDetailsViewModel {
                NavigationCustomLink(
                    with: SwapDetailsListView(model: swapDetailsViewModel),
                    action: model.onSelectSwapDetails
                )
            }
        }
    }

    private var swapButton: StateButton {
        StateButton(
            text: model.buttonViewModel.title,
            type: model.buttonViewModel.type,
            image: model.buttonViewModel.icon,
            infoTitle: model.buttonViewModel.infoText,
            action: onSelectActionButton
        )
    }

    private var bottomActionView: some View {
        InputAccessoryView(
            isEditing: focusedField && !model.buttonViewModel.isVisible,
            suggestions: SwapSceneViewModel.inputPercentSuggestions,
            onSelect: {
                focusedField = false
                model.onSelectPercent($0.value)
            },
            onDone: { focusedField = false },
            button: swapButton
        )
    }
}

// MARK: - Actions

extension SwapScene {
    private func onSelectActionButton() {
        focusedField = false
        model.buttonViewModel.action()
    }
}
