// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Primitives
import Onboarding
import ManageWallets
import Style

struct WalletsNavigationStack: View {
    @Environment(\.walletService) private var walletService
    @Environment(\.avatarService) private var avatarService
    @Environment(\.nameService) private var nameService
    @Environment(\.dismiss) private var dismiss

    @State private var navigationPath = NavigationPath()

    @State private var isPresentingCreateWalletSheet = false
    @State private var isPresentingImportWalletSheet = false

    var body: some View {
        NavigationStack(path: $navigationPath) {
            WalletsScene(
                model: WalletsSceneViewModel(
                    navigationPath: $navigationPath,
                    walletService: walletService,
                    isPresentingCreateWalletSheet: $isPresentingCreateWalletSheet,
                    isPresentingImportWalletSheet: $isPresentingImportWalletSheet
                )
            )
            .navigationDestination(for: Scenes.WalletDetail.self) {
                WalletDetailScene(
                    model: WalletDetailViewModel(
                        navigationPath: $navigationPath,
                        wallet: $0.wallet,
                        walletService: walletService
                    )
                )
            }
            .navigationDestination(for: Scenes.WalletSelectImage.self) {
                WalletImageScene(model: WalletImageViewModel(
                    wallet: $0.wallet,
                    avatarService: avatarService
                ))
            }
            .sheet(isPresented: $isPresentingCreateWalletSheet) {
                CreateWalletNavigationStack(
                    model: CreateWalletModel(
                        walletService: walletService,
                        avatarService: avatarService,
                        onComplete: { dismiss() }
                    )
                )
            }
            .sheet(isPresented: $isPresentingImportWalletSheet) {
                ImportWalletNavigationStack(
                    model: ImportWalletViewModel(
                        walletService: walletService,
                        avatarService: avatarService,
                        nameService: nameService,
                        onComplete: { dismiss() }
                    )
                )
            }
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("", systemImage: SystemImage.xmark) {
                        dismiss()
                    }
                }
            }
            .navigationBarTitleDisplayMode(.inline)
        }
    }
}
