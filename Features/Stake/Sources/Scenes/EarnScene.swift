// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Primitives
import Localization
import PrimitivesComponents

public struct EarnScene: View {
    private let model: EarnSceneViewModel

    public init(model: EarnSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        List {
            switch model.providersState {
            case .noData:
                Section {
                    ListItemView(title: Localized.Errors.noDataAvailable)
                }
            case .loading:
                ListItemLoadingView()
                    .id(UUID())
            case .data:
                Section(model.assetTitle) {
                    ListItemView(
                        title: model.aprModel.title,
                        subtitle: model.aprModel.subtitle
                    )
                }
            case .error(let error):
                ListItemErrorView(errorTitle: Localized.Errors.errorOccured, error: error)
            }

            if model.showDeposit {
                Section(Localized.Common.manage) {
                    NavigationLink(value: model.depositDestination) {
                        ListItemView(title: Localized.Wallet.deposit)
                    }
                }
            }

            Section(model.positionsSectionTitle) {
                if model.hasPositions {
                    ForEach(model.positionModels) { delegation in
                        NavigationLink(value: delegation.delegation) {
                            DelegationView(delegation: delegation)
                        }
                    }
                    .listRowInsets(.assetListRowInsets)
                } else if model.showEmptyState {
                    EmptyContentView(model: model.emptyContentModel)
                        .cleanListRow()
                }
            }
        }
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
        .refreshable {
            await model.fetch()
        }
        .taskOnce {
            Task {
                await model.fetch()
            }
        }
    }
}
