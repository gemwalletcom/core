// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SwiftUI
import Transfer
import Stake
import InfoSheet

struct StakeNavigationView: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    @State private var model: StakeSceneViewModel
    @Binding private var navigationPath: NavigationPath

    public init(
        model: StakeSceneViewModel,
        navigationPath: Binding<NavigationPath>
    ) {
        _model = State(initialValue: model)
        _navigationPath = navigationPath
    }

    var body: some View {
        StakeScene(
            model: model
        )
        .bindQuery(model.delegationsQuery, model.assetQuery, model.validatorsQuery)
        .ifLet(model.stakeInfoUrl, content: { view, url in
            view.toolbarInfoButton(url: url)
        })
        .sheet(item: $model.isPresentingInfoSheet) {
            InfoSheetScene(type: $0)
        }
        .navigationDestination(for: AmountInput.self) { input in
            AmountNavigationView(
                model: viewModelFactory.amountScene(
                    input: input,
                    wallet: model.wallet,
                    onTransferAction: {
                        navigationPath.append($0)
                    }
                )
            )
        }
        .navigationDestination(for: Delegation.self) { delegation in
            DelegationScene(
                model: viewModelFactory.delegationScene(
                    wallet: model.wallet,
                    delegation: delegation,
                    asset: delegation.base.assetId.chain.asset,
                    validators: model.validators,
                    onAmountInputAction: {
                        navigationPath.append($0)
                    },
                    onTransferAction: {
                        navigationPath.append($0)
                    }
                )
            )
        }
    }
}
