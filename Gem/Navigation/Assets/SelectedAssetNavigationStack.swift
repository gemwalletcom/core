// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives
import FiatConnect
import PrimitivesComponents
import Swap
import Transfer

struct SelectedAssetNavigationStack: View  {
    @Environment(\.viewModelFactory) private var viewModelFactory
    @Environment(\.assetsEnabler) private var assetsEnabler
    @Environment(\.activityService) private var activityService

    @State private var navigationPath = NavigationPath()

    private let input: SelectedAssetInput
    private let wallet: Wallet
    private let onComplete: VoidAction

    init(
        input: SelectedAssetInput,
        wallet: Wallet,
        onComplete: VoidAction
    ) {
        self.input = input
        self.wallet = wallet
        self.onComplete = onComplete
    }

    var body: some View {
        NavigationStack(path: $navigationPath) {
            Group {
                switch input.type {
                case .send(let type):
                    RecipientNavigationView(
                        model: viewModelFactory.recipientScene(
                            wallet: wallet,
                            asset: input.asset,
                            type: type,
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
                            wallet: wallet,
                            address: input.address,
                            assetsEnabler: assetsEnabler
                        )
                    )
                case let .buy(_, amount):
                    FiatConnectNavigationView(
                        model: viewModelFactory.fiatScene(
                            assetAddress: input.assetAddress,
                            walletId: wallet.walletId,
                            type: .buy,
                            amount: amount
                        )
                    )
                case let .sell(_, amount):
                    FiatConnectNavigationView(
                        model: viewModelFactory.fiatScene(
                            assetAddress: input.assetAddress,
                            walletId: wallet.walletId,
                            type: .sell,
                            amount: amount
                        )
                    )
                case let .swap(fromAsset, toAsset):
                    SwapNavigationView(
                        model: viewModelFactory.swapScene(
                            input: SwapInput(
                                wallet: wallet,
                                pairSelector: SwapPairSelectorViewModel(
                                    fromAssetId: fromAsset.id,
                                    toAssetId: toAsset?.id ?? SwapPairSelectorViewModel.defaultSwapPair(for: fromAsset).toAssetId
                                )
                            ),
                            onSwap: {
                                navigationPath.append($0)
                            }
                        )
                    )
                case .stake:
                    StakeNavigationView(
                        model: viewModelFactory.stakeScene(
                            wallet: wallet,
                            chain: input.asset.id.chain
                        ),
                        navigationPath: $navigationPath
                    )
                case .earn:
                    #if DEBUG
                    EarnNavigationView(
                        wallet: wallet,
                        asset: input.asset,
                        viewModelFactory: viewModelFactory,
                        navigationPath: $navigationPath
                    )
                    #else
                    EmptyView()
                    #endif
                }
            }
            .toolbarDismissItem(type: .close, placement: .topBarLeading)
            .navigationBarTitleDisplayMode(.inline)
            .navigationDestination(for: TransferData.self) { data in
                ConfirmTransferScene(
                    model: viewModelFactory.confirmTransferScene(
                        wallet: wallet,
                        data: data,
                        onComplete: onComplete
                    )
                )
            }
            .taskOnce {
                updateRecent()
            }
        }
    }
}

// MARK: - Private

extension SelectedAssetNavigationStack {
    private func updateRecent() {
        if let data = input.type.recentActivityData(assetId: input.asset.id) {
            try? activityService.updateRecent(data: data, walletId: wallet.walletId)
        }
    }
}
