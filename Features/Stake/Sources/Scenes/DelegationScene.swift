// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import PrimitivesComponents

public struct DelegationScene: View {
    private let model: DelegationSceneViewModel

    public init(model: DelegationSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        List {
            Section { } header: {
                WalletHeaderView(
                    model: model.model,
                    isPrivacyEnabled: .constant(false),
                    balanceActionType: .none,
                    onHeaderAction: nil,
                    onInfoAction: nil
                )
                .padding(.top, .small)
            }
            .cleanListRow()

            Section {
                if let url = model.providerUrl {
                    SafariNavigationLink(url: url) {
                        ListItemView(field: model.providerField)
                    }
                } else {
                    ListItemView(field: model.providerField)
                }

                if model.aprModel.showApr {
                    ListItemView(title: model.aprModel.title, subtitle: model.aprModel.subtitle)
                }

                ListItemView(title: model.stateTitle, subtitle: model.stateModel.title, subtitleStyle: model.stateModel.textStyle)

                if let completionDateField = model.completionDateField {
                    ListItemView(field: completionDateField)
                }
            }

            if let rewardsText = model.model.rewardsText {
                Section {
                    ListItemView(
                        title: model.rewardsTitle,
                        titleStyle: model.model.titleStyle,
                        subtitle: rewardsText,
                        subtitleStyle: model.model.subtitleStyle,
                        subtitleExtra: model.model.rewardsFiatValueText,
                        subtitleStyleExtra: model.model.subtitleExtraStyle,
                        imageStyle: model.assetImageStyle
                    )
                }
            }

            if model.showManage {
                Section(model.manageTitle) {
                    ForEach(model.availableActions) { action in
                        NavigationCustomLink(with: ListItemView(title: model.actionTitle(action))) {
                            model.onSelectAction(action)
                        }
                    }
                }
            }
        }
        .navigationTitle(model.title)
        .listSectionSpacing(.compact)
    }
}
