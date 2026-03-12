// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Store
import PriceService
import PriceAlertService
import BalanceService
import TransactionsService
import NFTService
import PerpetualService
import Preferences

public struct StreamEventService: Sendable {
    private let walletStore: WalletStore
    private let notificationStore: InAppNotificationStore
    private let priceService: PriceService
    private let priceAlertService: PriceAlertService
    private let balanceUpdater: any BalanceUpdater
    private let transactionsService: TransactionsService
    private let nftService: NFTService
    private let perpetualService: any HyperliquidPerpetualServiceable
    private let preferences: Preferences

    public init(
        walletStore: WalletStore,
        notificationStore: InAppNotificationStore,
        priceService: PriceService,
        priceAlertService: PriceAlertService,
        balanceUpdater: any BalanceUpdater,
        transactionsService: TransactionsService,
        nftService: NFTService,
        perpetualService: any HyperliquidPerpetualServiceable,
        preferences: Preferences
    ) {
        self.walletStore = walletStore
        self.notificationStore = notificationStore
        self.priceService = priceService
        self.priceAlertService = priceAlertService
        self.balanceUpdater = balanceUpdater
        self.transactionsService = transactionsService
        self.nftService = nftService
        self.perpetualService = perpetualService
        self.preferences = preferences
    }

    public func handle(_ event: StreamEvent) async {
        switch event {
        case .prices(let payload):
            await perform { try handlePrices(payload) }
        case .balances(let updates):
            Task { await self.perform { try await self.handleBalanceUpdates(updates) } }
        case .transactions(let update):
            Task { await self.perform { try await self.transactionsService.updateAll(walletId: update.walletId) } }
        case .nft(let update):
            Task { await self.perform { try await self.handleNftUpdate(update) } }
        case .perpetual(let update):
            Task { await self.perform { try await self.handlePerpetualUpdate(update) } }
        case .inAppNotification(let update):
            await perform { try notificationStore.addNotifications([update.notification]) }
        case .priceAlerts:
            Task { await self.perform { try await self.priceAlertService.update() } }
        }
    }
}

// MARK: - Private

extension StreamEventService {
    private func perform(_ body: () async throws -> Void) async {
        do {
            try await body()
        } catch {
            debugLog("stream event handler error: \(error)")
        }
    }

    private func handlePrices(_ payload: WebSocketPricePayload) throws {
        debugLog("stream event handler: prices: \(payload.prices.count), rates: \(payload.rates.count)")
        try priceService.addRates(payload.rates)
        try priceService.updatePrices(payload.prices, currency: preferences.currency)
    }

    private func handleBalanceUpdates(_ updates: [StreamBalanceUpdate]) async throws {
        for (walletId, walletUpdates) in Dictionary(grouping: updates, by: \.walletId) {
            guard let wallet = try walletStore.getWallet(id: walletId) else { continue }
            await balanceUpdater.updateBalance(for: wallet, assetIds: walletUpdates.map(\.assetId))
        }
    }

    private func handleNftUpdate(_ update: StreamNftUpdate) async throws {
        guard let wallet = try walletStore.getWallet(id: update.walletId) else { return }
        try await nftService.updateAssets(wallet: wallet)
    }

    private func handlePerpetualUpdate(_ update: StreamPerpetualUpdate) async throws {
        guard let wallet = try walletStore.getWallet(id: update.walletId), let account = wallet.hyperliquidAccount else { return }
        try await perpetualService.fetchPositions(walletId: update.walletId, address: account.address)
    }
}
