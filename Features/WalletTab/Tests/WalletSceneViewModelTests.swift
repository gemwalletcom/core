// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Primitives
import PrimitivesTestKit
import BannerServiceTestKit
import WalletTabTestKit

@testable import WalletTab
@testable import Store

@MainActor
struct WalletSceneViewModelTests {
    @Test
    func isLoading() {
        let model = WalletSceneViewModel.mock()
        #expect(model.isLoadingAssets == false)

        model.shouldStartLoadingAssets()
        #expect(model.isLoadingAssets)

        model.fetch()
        #expect(!model.isLoadingAssets == false)
    }

    @Test
    func priorityBannerReturnsHighestPriority() {
        let model = WalletSceneViewModel.mock()
        model.bannersQuery.value = [
            .mock(event: .stake, state: .active),
            .mock(event: .enableNotifications, state: .cancelled),
            .mock(event: .accountActivation, state: .alwaysActive)
        ]

        #expect(model.walletBannersModel.allBanners.first?.state == .alwaysActive)
    }

    @Test
    func onChangeWalletUpdatesImageUrl() {
        let wallet = Wallet.mock(id: "multicoin_0x1", imageUrl: nil)
        let model = WalletSceneViewModel.mock(wallet: wallet)

        #expect(model.wallet.imageUrl == nil)

        let updatedWallet = Wallet.mock(id: "multicoin_0x1", imageUrl: "avatar.png")
        model.onChangeWallet(wallet, updatedWallet)

        #expect(model.wallet.imageUrl == "avatar.png")
    }

    @Test
    func onChangeWalletSwitchesToDifferentWallet() {
        let wallet = Wallet.mock(id: "multicoin_0x1", name: "Wallet 1")
        let model = WalletSceneViewModel.mock(wallet: wallet)

        #expect(model.wallet.id == "multicoin_0x1")

        let newWallet = Wallet.mock(id: "multicoin_0x2", name: "Wallet 2")
        model.onChangeWallet(wallet, newWallet)

        #expect(model.wallet.id == "multicoin_0x2")
    }
}
