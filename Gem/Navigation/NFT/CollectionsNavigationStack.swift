// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Primitives
import NFT
import Components
import Localization
import Style
import Assets
import AssetsService

struct CollectionsNavigationStack: View {
    @Environment(\.navigationState) private var navigationState
    @Environment(\.assetsEnabler) private var assetsEnabler
    @Environment(\.avatarService) private var avatarService
    @Environment(\.priceAlertService) private var priceAlertService
    @Environment(\.activityService) private var activityService
    @Environment(\.nftService) private var nftService
    @Environment(\.assetSearchService) private var assetSearchService

    @State private var model: CollectionsViewModel

    @Binding private var isPresentingSelectedAssetInput: SelectedAssetInput?

    private var navigationPath: Binding<NavigationPath> {
        navigationState.collections.binding
    }

    init(
        model: CollectionsViewModel,
        isPresentingSelectedAssetInput: Binding<SelectedAssetInput?>
    ) {
        _model = State(initialValue: model)
        _isPresentingSelectedAssetInput = isPresentingSelectedAssetInput
    }

    var body: some View {
        NavigationStack(path: navigationPath) {
            CollectionsScene(model: model)
                .onChange(
                    of: model.currentWallet,
                    initial: true,
                    model.onChangeWallet
                )
                .navigationDestination(for: Scenes.Collection.self) { scene in
                    CollectionsScene(
                        model: CollectionViewModel(
                            wallet: model.wallet,
                            collectionId: scene.id,
                            collectionName: scene.name
                        )
                    )
                }
                .navigationDestination(for: Scenes.UnverifiedCollections.self) { _ in
                    CollectionsScene(
                        model: UnverifiedCollectionsViewModel(wallet: model.wallet)
                    )
                }
                .navigationDestination(for: Scenes.Collectible.self) {
                    CollectibleScene(
                        model: CollectibleViewModel(
                            wallet: model.wallet,
                            assetData: $0.assetData,
                            avatarService: avatarService,
                            nftService: nftService,
                            isPresentingSelectedAssetInput: $isPresentingSelectedAssetInput
                        )
                    )
                }
                .sheet(item: $model.isPresentingReceiveSelectAssetType) {
                    SelectAssetSceneNavigationStack(
                        model: SelectAssetViewModel(
                            wallet: model.wallet,
                            selectType: $0,
                            searchService: assetSearchService,
                            assetsEnabler: assetsEnabler,
                            priceAlertService: priceAlertService,
                            activityService: activityService
                        )
                    )
                }
                .toolbar {
                    ToolbarItem(placement: .navigationBarTrailing) {
                        Button(action: model.onSelectReceive) {
                            Images.System.plus
                        }
                    }
                }
        }
    }
}
