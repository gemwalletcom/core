// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Preferences
import Primitives
import Store
import WebSocketClient
import PriceService
import PriceAlertService
import BalanceService
import TransactionsService
import NFTService
import PerpetualService

public actor StreamObserverService: Sendable {
    private let walletStore: WalletStore
    private let notificationStore: InAppNotificationStore
    private let priceService: PriceService
    private let priceAlertService: PriceAlertService
    private let balanceUpdater: any BalanceUpdater
    private let transactionsService: TransactionsService
    private let nftService: NFTService
    private let perpetualService: any PerpetualServiceable
    private let subscriptionService: StreamSubscriptionService

    private let preferences: Preferences
    private let decoder = JSONDateDecoder.standard

    private let webSocket: any WebSocketConnectable
    private var observeTask: Task<Void, Never>?

    public init(
        walletStore: WalletStore,
        notificationStore: InAppNotificationStore,
        priceService: PriceService,
        priceAlertService: PriceAlertService,
        balanceUpdater: any BalanceUpdater,
        transactionsService: TransactionsService,
        nftService: NFTService,
        perpetualService: any PerpetualServiceable,
        subscriptionService: StreamSubscriptionService,
        preferences: Preferences,
        webSocket: any WebSocketConnectable
    ) {
        self.walletStore = walletStore
        self.notificationStore = notificationStore
        self.priceService = priceService
        self.priceAlertService = priceAlertService
        self.balanceUpdater = balanceUpdater
        self.transactionsService = transactionsService
        self.nftService = nftService
        self.perpetualService = perpetualService
        self.subscriptionService = subscriptionService
        self.preferences = preferences
        self.webSocket = webSocket
    }

    deinit {
        observeTask?.cancel()
    }

    // MARK: - Public API

    public func connect() {
        guard observeTask == nil else { return }

        observeTask = Task { [weak self] in
            guard let self else { return }
            await self.observeConnection()
        }
    }

    public func disconnect() async {
        guard observeTask != nil else { return }

        observeTask?.cancel()
        observeTask = nil

        await webSocket.disconnect()
    }

    // MARK: - Private

    private func observeConnection() async {
        for await event in await webSocket.connect() {
            guard !Task.isCancelled else { break }

            switch event {
            case .connected: await subscriptionService.resubscribe()
            case .message(let data): await handleMessage(data)
            case .disconnected: break
            }
        }
    }

    private func handleMessage(_ data: Data) async {
        do {
            switch try decoder.decode(StreamEvent.self, from: data) {
            case .prices(let payload):
                try handlePrices(payload)
            case .balances(let updates):
                Task { await self.perform { try await self.handleBalanceUpdates(updates) } }
            case .transactions(let update):
                Task { await self.perform { try await self.transactionsService.updateAll(walletId: update.walletId) } }
            case .nft(let update):
                Task { await self.perform { try await self.handleNftUpdate(update) } }
            case .perpetual(let update):
                Task { await self.perform { try await self.handlePerpetualUpdate(update) } }
            case .inAppNotification(let update):
                try notificationStore.addNotifications([update.notification])
            case .priceAlerts:
                Task { await self.perform { try await self.priceAlertService.update() } }
            }
        } catch {
            debugLog("stream observer: handleMessage error: \(error)")
        }
    }
}

// MARK: - Event Handlers

extension StreamObserverService {
    private func perform(_ body: () async throws -> Void) async {
        do {
            try await body()
        } catch {
            debugLog("stream observer error: \(error)")
        }
    }

    private func handlePrices(_ payload: WebSocketPricePayload) throws {
        debugLog("stream observer: prices: \(payload.prices.count), rates: \(payload.rates.count)")
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
