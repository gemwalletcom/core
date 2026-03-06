// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Localization
import Style
import PrimitivesComponents

public struct AutocloseScene: View {
    public enum Field: Int, Hashable {
        case takeProfit
        case stopLoss
    }
    @FocusState private var focusedField: Field?
    @State private var model: AutocloseSceneViewModel

    public init(model: AutocloseSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            Section {
                ListAssetItemView(model: model.positionItemViewModel)
            }

            Section {
                if let entryPriceField = model.entryPriceField {
                    ListItemView(field: entryPriceField)
                }
                ListItemView(field: model.marketPriceField)
            }

            AutocloseInputSection(
                inputModel: $model.input.takeProfit,
                sectionModel: model.takeProfitModel,
                field: Field.takeProfit,
                focusedField: $focusedField
            )

            AutocloseInputSection(
                inputModel: $model.input.stopLoss,
                sectionModel: model.stopLossModel,
                field: Field.stopLoss,
                focusedField: $focusedField
            )
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .safeAreaView {
            InputAccessoryView(
                isEditing: model.input.focused?.text.isEmpty == true,
                suggestions: model.takeProfitModel.percentSuggestions,
                onSelect: { model.onSelectPercent($0.value) },
                onDone: { focusedField = nil },
                button: StateButton(
                    text: Localized.Transfer.confirm,
                    type: model.confirmButtonType,
                    action: model.onSelectConfirm
                )
            )
        }
        .navigationTitle(model.title)
        .onChange(of: focusedField, model.onChangeFocusField)
    }
}
