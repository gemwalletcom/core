// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import SwiftUI
import Primitives
import PrimitivesTestKit
import WalletService
import StoreTestKit
import WalletServiceTestKit

@testable import ManageWallets
@testable import Store

@MainActor
struct WalletsSceneViewModelTests {
    
    @Test
    func onDeleteConfirmed() async throws {
        let service: WalletService = try .mockWallets()
        let model = WalletsSceneViewModel.mock(walletService: service)
        model.walletsQuery.value = service.wallets

        #expect(model.currentWalletId == .multicoin(address: "0x1"))

        await model.onDeleteConfirmed(wallet: .mock(id: "multicoin_0x1"))

        #expect(model.currentWalletId == .multicoin(address: "0x2"))

        await model.onDeleteConfirmed(wallet: .mock(id: "multicoin_0x2"))

        #expect(model.currentWalletId == .multicoin(address: "0x3"))

        await model.onDeleteConfirmed(wallet: .mock(id: "multicoin_0x3"))
        
        #expect(model.currentWalletId == .none)
    }

    @Test
    func onMove() throws {
        let service: WalletService = try .mockWallets()
        let model = WalletsSceneViewModel.mock(walletService: service)
        model.walletsQuery.value = service.wallets

        model.onMove(from: IndexSet(integer: 0), to: 0)
        #expect(service.sortedWallets.ids == ["multicoin_0x1", "multicoin_0x2", "multicoin_0x3"])

        model.onMove(from: IndexSet(integer: 1), to: 0)
        #expect(service.sortedWallets.ids == ["multicoin_0x2", "multicoin_0x1", "multicoin_0x3"])

        model.onMove(from: IndexSet(integer: 0), to: 3)
        #expect(service.sortedWallets.ids == ["multicoin_0x3", "multicoin_0x2", "multicoin_0x1"])

        model.onMove(from: IndexSet(integer: 2), to: 0)
        #expect(service.sortedWallets.ids == ["multicoin_0x2", "multicoin_0x1", "multicoin_0x3"])
        
    }
}

// MARK: - Mock Extensions

extension WalletsSceneViewModel {
    static func mock(
        navigationPath: Binding<NavigationPath> = .constant(NavigationPath()),
        walletService: WalletService = .mock(),
        isPresentingCreateWalletSheet: Binding<Bool> = .constant(false),
        isPresentingImportWalletSheet: Binding<Bool> = .constant(false)
    ) -> WalletsSceneViewModel {
        WalletsSceneViewModel(
            navigationPath: navigationPath,
            walletService: walletService,
            isPresentingCreateWalletSheet: isPresentingCreateWalletSheet,
            isPresentingImportWalletSheet: isPresentingImportWalletSheet
        )
    }
}

extension WalletService {
    static func mockWallets() throws -> Self {
        let walletStore = WalletStore.mock(db: .mock())
        let wallet1 = Wallet.mock(id: "multicoin_0x1")
        let wallet2 = Wallet.mock(id: "multicoin_0x2")
        let wallet3 = Wallet.mock(id: "multicoin_0x3")
        try walletStore.addWallet(wallet1)
        try walletStore.addWallet(wallet2)
        try walletStore.addWallet(wallet3)

        let service = WalletService.mock(walletStore: walletStore)
        service.setCurrent(for: wallet1.walletId)
        
        return service
    }
    
    var sortedWallets: [Wallet] {
        wallets.sorted { $0.order < $1.order }
    }
}
