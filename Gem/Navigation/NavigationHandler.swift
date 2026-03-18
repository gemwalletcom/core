// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Primitives
import AssetsService
import TransactionsService
import WalletService

@Observable
final class NavigationHandler: Sendable {
    private let navigationState: NavigationStateManager
    private let presenter: NavigationPresenter

    private let assetsService: AssetsService
    private let transactionsService: TransactionsService
    private let walletService: WalletService

    @MainActor var wallet: Wallet?

    init(
        navigationState: NavigationStateManager,
        presenter: NavigationPresenter,
        assetsService: AssetsService,
        transactionsService: TransactionsService,
        walletService: WalletService
    ) {
        self.navigationState = navigationState
        self.presenter = presenter
        self.assetsService = assetsService
        self.transactionsService = transactionsService
        self.walletService = walletService
    }

    @MainActor
    func handlePush(_ userInfo: [AnyHashable: Any]) async {
        do {
            let notification = try PushNotification(from: userInfo)
            try await handle(notification)
        } catch {
            debugLog("NavigationHandler push error: \(error)")
        }
    }

    @MainActor
    func handle(_ action: URLAction) async {
        do {
            try await handleURLAction(action)
        } catch {
            debugLog("NavigationHandler URLAction error: \(error)")
        }
    }
}

// MARK: - URLAction

@MainActor
extension NavigationHandler {
    private func handleURLAction(_ action: URLAction) async throws {
        switch action {
        case .walletConnect:
            return

        case .asset(let assetId):
            try await navigateToAsset(assetId)

        case .swap(let fromId, let toId):
            try await presentSwap(from: fromId, to: toId)
            return

        case .perpetuals:
            navigationState.wallet.append(Scenes.Perpetuals())

        case .rewards(let code):
            navigationState.settings.append(Scenes.Referral(code: code))

        case .gift(let code):
            navigationState.settings.append(Scenes.Referral(code: nil, giftCode: code))

        case .buy(let assetId, let amount):
            try await presentBuy(assetId: assetId, amount: amount)
            return

        case .sell(let assetId, let amount):
            try await presentSell(assetId: assetId, amount: amount)
            return

        case .setPriceAlert(let assetId, let price):
            try await presentSetPriceAlert(assetId: assetId, price: price)
            return
        }

        selectTab(for: action.selectTab)
    }
}

// MARK: - PushNotification

@MainActor
extension NavigationHandler {
    private func handle(_ notification: PushNotification) async throws {
        switch notification {
        case .asset(let assetId):
            try await navigateToAsset(assetId)
        case .transaction(let walletId, let assetId, let transaction):
            try await navigateToTransaction(walletId: walletId, assetId: assetId, transaction: transaction)
        case .priceAlert(let assetId):
            try await navigateToAsset(assetId)
        case .buyAsset(let assetId, let amount):
            try await presentBuy(assetId: assetId, amount: amount)
        case .swapAsset(let fromId, let toId):
            try await presentSwap(from: fromId, to: toId)
        case .support:
            presenter.isPresentingSupport.wrappedValue = true
        case .rewards:
            navigationState.settings.append(Scenes.Referral(code: nil))
        case .stake: break
            //TODO: Select wallet and open stake screen of an asset
        case .test, .unknown: break
        }

        selectTab(for: notification.selectTab)
    }
}

// MARK: - Private

@MainActor
extension NavigationHandler {
    private func selectTab(for tab: TabItem?) {
        guard let tab else { return }
        navigationState.selectedTab = tab
    }

    private func navigateToAsset(_ assetId: AssetId) async throws {
        let asset = try await assetsService.getOrFetchAsset(for: assetId)
        navigationState.wallet.append(Scenes.Asset(asset: asset))
    }

    private func navigateToTransaction(walletId: WalletId, assetId: AssetId, transaction: Primitives.Transaction) async throws {
        guard let _ = try? walletService.getWallet(walletId: walletId) else {
            return
        }

        let asset = try await assetsService.getOrFetchAsset(for: assetId)
        try transactionsService.addTransaction(walletId: walletId, transaction: transaction)
        let transaction = try transactionsService.getTransaction(walletId: walletId, transactionId: transaction.id.identifier)

        if walletService.currentWalletId != walletId {
            walletService.setCurrent(for: walletId)
            await Task.yield()
        }

        switch asset.type {
        case .perpetual:
            navigationState.wallet.setPath([Scenes.Perpetuals(), Scenes.Perpetual(asset), Scenes.Transaction(transaction: transaction)])
        default:
            navigationState.wallet.setPath([Scenes.Asset(asset: asset), Scenes.Transaction(transaction: transaction)])
        }

        navigationState.selectedTab = .wallet
    }

    private func presentSwap(from fromId: AssetId, to toId: AssetId?) async throws {
        let fromAsset = try await assetsService.getOrFetchAsset(for: fromId)
        let toAsset: Asset? = if let toId { try await assetsService.getOrFetchAsset(for: toId) } else { nil }
        try presentAssetInput(type: .swap(fromAsset, toAsset), for: fromAsset)
    }

    private func presentBuy(assetId: AssetId, amount: Int?) async throws {
        let asset = try await assetsService.getOrFetchAsset(for: assetId)
        try presentAssetInput(type: .buy(asset, amount: amount), for: asset)
    }

    private func presentSell(assetId: AssetId, amount: Int?) async throws {
        let asset = try await assetsService.getOrFetchAsset(for: assetId)
        try presentAssetInput(type: .sell(asset, amount: amount), for: asset)
    }

    private func presentSetPriceAlert(assetId: AssetId, price: Double?) async throws {
        let asset = try await assetsService.getOrFetchAsset(for: assetId)
        presenter.isPresentingPriceAlert.wrappedValue = SetPriceAlertInput(asset: asset, price: price)
    }

    private func presentAssetInput(type: SelectedAssetType, for asset: Asset) throws {
        guard let wallet else { return }
        let account = try wallet.account(for: asset.chain)
        presenter.isPresentingAssetInput.wrappedValue = SelectedAssetInput(
            type: type,
            assetAddress: AssetAddress(asset: account.chain.asset, address: account.address)
        )
    }

    func resetNavigation() {
        navigationState.clearAll()
        navigationState.selectedTab = .wallet
    }
}

// MARK: - TabItem Selection

private extension URLAction {
    var selectTab: TabItem? {
        switch self {
        case .asset, .perpetuals: .wallet
        case .swap, .buy, .sell, .setPriceAlert, .walletConnect: nil
        case .rewards, .gift: .settings
        }
    }
}

private extension PushNotification {
    var selectTab: TabItem? {
        switch self {
        case .transaction, .asset, .priceAlert, .stake: .wallet
        case .buyAsset, .swapAsset: nil
        case .support, .rewards: .settings
        case .test, .unknown: nil
        }
    }
}
