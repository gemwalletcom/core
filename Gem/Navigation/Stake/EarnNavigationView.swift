// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Primitives
import Stake
import Transfer

struct EarnNavigationView: View {
    @Environment(\.viewModelFactory) private var viewModelFactory
    @State private var model: EarnSceneViewModel
    @Binding var navigationPath: NavigationPath

    let wallet: Wallet
    let asset: Asset

    init(
        wallet: Wallet,
        asset: Asset,
        viewModelFactory: ViewModelFactory,
        navigationPath: Binding<NavigationPath>
    ) {
        _model = State(initialValue: viewModelFactory.earnScene(wallet: wallet, asset: asset))
        self.wallet = wallet
        self.asset = asset
        _navigationPath = navigationPath
    }

    var body: some View {
        EarnScene(model: model)
            .bindQuery(model.assetQuery, model.positionsQuery, model.providersQuery)
            .navigationDestination(for: AmountInput.self) { input in
                AmountNavigationView(
                    model: viewModelFactory.amountScene(
                        input: input,
                        wallet: wallet,
                        onTransferAction: {
                            navigationPath.append($0)
                        }
                    )
                )
            }
            .navigationDestination(for: Delegation.self) { delegation in
                DelegationScene(
                    model: viewModelFactory.delegationScene(
                        wallet: wallet,
                        delegation: delegation,
                        asset: asset,
                        validators: [],
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
