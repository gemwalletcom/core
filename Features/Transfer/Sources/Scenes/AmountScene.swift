// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import struct Stake.ValidatorViewModel
import struct Stake.ValidatorView

struct AmountScene: View {
    @FocusState private var focusedField: Bool

    private var model: AmountSceneViewModel

    init(model: AmountSceneViewModel) {
        self.model = model
    }

    var body: some View {
        @Bindable var model = model
        List {
            CurrencyInputValidationView(
                model: $model.amountInputModel,
                config: model.inputConfig,
                infoAction: model.infoAction(for:)
            )
            .padding(.top, .medium)
            .listGroupRowStyle()
            .disabled(model.isInputDisabled)
            .focused($focusedField)

            if model.isBalanceViewEnabled {
                Section {
                    AssetBalanceView(
                        image: model.assetImage,
                        title: model.assetName,
                        balance: model.balanceText,
                        secondary: {
                            Button(model.maxTitle, action: onSelectMaxButton)
                                .buttonStyle(.listEmpty(paddingHorizontal: .medium, paddingVertical: .small))
                                .fixedSize()
                        }
                    )
                }
            }

            if let infoText = model.infoText {
                Section {
                    Button(action: model.onSelectReservedFeesInfo) {
                        HStack {
                            Images.System.info
                                .foregroundStyle(Colors.gray)
                                .frame(width: .list.image, height: .list.image)
                            Text(infoText)
                                .textStyle(.calloutSecondary)
                        }
                    }
                }
            }

            switch model.provider {
            case let .stake(stake):
                Section(stake.validatorSelection.title) {
                    if stake.validatorSelection.isEnabled {
                        NavigationLink(value: stake.validatorSelection.selected) {
                            ValidatorView(model: ValidatorViewModel(validator: stake.validatorSelection.selected))
                        }
                    } else {
                        ValidatorView(model: ValidatorViewModel(validator: stake.validatorSelection.selected))
                    }
                }

            case let .freeze(freeze):
                @Bindable var resourceSelection = freeze.resourceSelection
                Section {
                    Picker("", selection: $resourceSelection.selected) {
                        ForEach(resourceSelection.options) { resource in
                            Text(ResourceViewModel(resource: resource).title)
                                .tag(resource)
                        }
                    }
                    .pickerStyle(.segmented)
                    .frame(width: 200)
                    .onChange(of: resourceSelection.selected, model.onChangeResource)
                }
                .cleanListRow()

            case let .perpetual(perpetual):
                if let leverageSelection = perpetual.leverageSelection {
                    Section {
                        NavigationCustomLink(
                            with: ListItemView(
                                title: leverageSelection.title,
                                subtitle: leverageSelection.selected.displayText,
                                subtitleStyle: perpetual.leverageTextStyle
                            ),
                            action: model.onSelectLeverage
                        )
                    }
                }

                if perpetual.isAutocloseEnabled {
                    Section {
                        NavigationCustomLink(
                            with: ListItemView(
                                title: perpetual.autocloseTitle,
                                subtitle: perpetual.autocloseText.subtitle,
                                subtitleExtra: perpetual.autocloseText.subtitleExtra
                            ),
                            action: model.onSelectAutoclose
                        )
                    }
                }

            case let .earn(earn):
                Section(earn.providerTitle) {
                    ValidatorView(model: ValidatorViewModel(validator: earn.provider))
                }

            case .transfer:
                EmptyView()
            }
        }
        .safeAreaButton {
            StateButton(
                text: model.continueTitle,
                type: .primary(model.actionButtonState),
                action: onSelectNextButton
            )
        }
        .contentMargins([.top], .zero, for: .scrollContent)
        .listSectionSpacing(.custom(.medium))
        .frame(maxWidth: .infinity)
        .navigationTitle(model.title)
        .onAppear {
            model.onAppear()
            if model.shouldFocusOnAppear {
                focusedField = true
            }
        }
    }

    private func onSelectMaxButton() {
        focusedField = false
        model.onSelectMaxButton()
    }

    private func onSelectNextButton() {
        focusedField = false
        model.onSelectNextButton()
    }
}
