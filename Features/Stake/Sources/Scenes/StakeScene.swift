// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Store
import Primitives
import Components
import Localization
import InfoSheet
import PrimitivesComponents

public struct StakeScene: View {
    private let model: StakeSceneViewModel

    public init(model: StakeSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        List {
            stakeInfoSection
            if model.showManage {
                stakeSection
            }
            if model.showTronResources {
                resourcesSection
            }
            delegationsSection
        }
        .listSectionSpacing(.compact)
        .refreshable {
            await model.fetch()
        }
        .navigationTitle(model.title)
        .taskOnce {
            Task {
                await model.fetch()
            }
        }
    }
}

// MARK: - UI Components

extension StakeScene {
    private var stakeSection: some View {
        Section(Localized.Common.manage) {
            if model.showStake {
                NavigationLink(value: model.stakeDestination) {
                    ListItemView(title: model.stakeTitle)
                }
                .enabled(model.isStakeEnabled)
            }
            
            if model.showFreeze {
                NavigationLink(value: model.freezeDestination) {
                    ListItemView(title: model.freezeTitle)
                }
            }

            if model.showUnfreeze {
                NavigationLink(value: model.unfreezeDestination) {
                    ListItemView(title: model.unfreezeTitle)
                }
            }

            if model.canClaimRewards {
                NavigationLink(value: model.claimRewardsDestination) {
                    ListItemView(
                        title: model.claimRewardsTitle,
                        subtitle: model.claimRewardsText
                    )
                }
            }
        }
    }

    private var delegationsSection: some View {
        Section(model.delegationsSectionTitle) {
            switch model.delegationsState {
            case .noData:
                EmptyContentView(model: model.emptyContentModel)
                    .cleanListRow()
            case .loading:
                ListItemLoadingView()
                    .id(UUID())
            case .data(let delegations):
                ForEach(delegations) { delegation in
                    NavigationLink(value: model.navigationDestination(for: delegation)) {
                        DelegationView(delegation: delegation)
                    }
                }
                .listRowInsets(.assetListRowInsets)
            case .error(let error):
                ListItemErrorView(errorTitle: Localized.Errors.errorOccured, error: error)
            }
        }
    }

    private var stakeInfoSection: some View {
        Section(model.assetTitle) {
            ListItemView(
                title: model.stakeAprModel.title,
                subtitle: model.stakeAprModel.subtitle,
                infoAction: model.onAprInfo
            )
            ListItemView(
                title: model.lockTimeTitle,
                subtitle: model.lockTimeValue,
                infoAction: model.onLockTimeInfo
            )
            if let minAmountValue = model.minAmountValue {
                ListItemView(title: model.minAmountTitle, subtitle: minAmountValue)
            }
        }
    }

    private var resourcesSection: some View {
        Section(model.resourcesTitle) {
            ListItemView(
                title: model.energyTitle,
                subtitle: model.energyText
            )

            ListItemView(
                title: model.bandwidthTitle,
                subtitle: model.bandwidthText
            )
        }
        
    }
}
