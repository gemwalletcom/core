// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.Hyperliquid
import struct Gemstone.GemPerpetualBalance
import struct Gemstone.GemPerpetualPosition
import struct Gemstone.GemHyperliquidOpenOrder
import struct Gemstone.GemChartCandleUpdate
import Primitives
import WebSocketClient

public actor HyperliquidObserverService: PerpetualObservable {

    private let perpetualService: HyperliquidPerpetualServiceable
    private let webSocket: any WebSocketConnectable
    private let hyperliquid = Hyperliquid()
    private let encoder = JSONEncoder()

    private var observeTask: Task<Void, Never>?
    private var currentWallet: Wallet?
    private var activeSubscriptions: Set<HyperliquidSubscription> = []

    public let chartService: any ChartStreamable = ChartObserverService()

    public init(
        nodeProvider: any NodeURLFetchable,
        perpetualService: HyperliquidPerpetualServiceable
    ) {
        self.webSocket = WebSocketConnection(url: nodeProvider.node(for: .hyperCore))
        self.perpetualService = perpetualService
    }

    deinit {
        observeTask?.cancel()
    }

    // MARK: - Public API

    public func setup(for wallet: Wallet) async {
        await connect(for: wallet)
    }

    public func disconnect() async {
        guard observeTask != nil else { return }

        observeTask?.cancel()
        observeTask = nil
        currentWallet = nil

        await webSocket.disconnect()
    }

    public func subscribe(_ subscription: HyperliquidSubscription) async throws {
        activeSubscriptions.insert(subscription)
        try await send(HyperliquidRequest(method: .subscribe, subscription: subscription))
    }

    public func unsubscribe(_ subscription: HyperliquidSubscription) async throws {
        activeSubscriptions.remove(subscription)
        try await send(HyperliquidRequest(method: .unsubscribe, subscription: subscription))
    }

    public func update(for wallet: Wallet) async {
        guard let address = wallet.hyperliquidAccount?.address else { return }
        do {
            try await perpetualService.fetchPositions(walletId: wallet.walletId, address: address)
        } catch {
            debugLog("HyperliquidObserver: update failed: \(error)")
        }
    }

    // MARK: - Private

    private func connect(for wallet: Wallet) async {
        guard currentWallet?.id != wallet.id else { return }

        await disconnect()
        currentWallet = wallet
        await update(for: wallet)

        guard observeTask == nil else { return }

        observeTask = Task { [weak self] in
            guard let self else { return }
            await observeConnection()
        }
    }

    private func observeConnection() async {
        for await event in await webSocket.connect() {
            guard !Task.isCancelled else { break }

            switch event {
            case .connected:
                await handleConnected()
            case .message(let data):
                await handleMessage(data)
            case .disconnected:
                break
            }
        }
    }

    private func handleConnected() async {
        guard let address = currentWallet?.hyperliquidAccount?.address else { return }
        do {
            try await subscribe(defaultSubscriptions(for: address) + activeSubscriptions.asArray())
        } catch {
            debugLog("HyperliquidObserver: subscribe failed: \(error)")
        }
    }

    private func handleMessage(_ data: Data) async {
        do {
            switch try hyperliquid.parseWebsocketData(data: data) {
            case .clearinghouseState(let balance, let newPositions):
                try handleClearinghouseState(balance: balance, newPositions: newPositions)
            case .openOrders(let orders):
                try handleOpenOrders(orders: orders)
            case .candle(let candle):
                try await handleCandle(candle: candle)
            case .allMids(let prices):
                try perpetualService.updatePrices(prices)
            case .subscriptionResponse(let subscriptionType):
                debugLog("HyperliquidObserver: subscription response - \(subscriptionType)")
            case .unknown:
                debugLog("HyperliquidObserver: unknown message: \(String(data: data, encoding: .utf8) ?? "nil")")
            }
        } catch {
            debugLog("HyperliquidObserver: handle message error: \(error)")
        }
    }

    private func handleClearinghouseState(
        balance: GemPerpetualBalance,
        newPositions: [GemPerpetualPosition]
    ) throws {
        guard let walletId = currentWallet?.walletId else { return }

        let diff = hyperliquid.diffClearinghousePositions(
            newPositions: newPositions,
            existingPositions: try perpetualService.getHypercorePositions(walletId: walletId)
        )

        try perpetualService.updateBalance(
            walletId: walletId,
            balance: balance
        )
        try perpetualService.diffPositions(
            deleteIds: diff.deletePositionIds,
            positions: diff.positions,
            walletId: walletId
        )
    }

    private func handleOpenOrders(orders: [GemHyperliquidOpenOrder]) throws {
        guard let walletId = currentWallet?.walletId else { return }

        let diff = hyperliquid.diffOpenOrdersPositions(
            orders: orders,
            existingPositions: try perpetualService.getHypercorePositions(walletId: walletId)
        )
        try perpetualService.diffPositions(
            deleteIds: diff.deletePositionIds,
            positions: diff.positions,
            walletId: walletId
        )
    }

    private func handleCandle(candle: GemChartCandleUpdate) async throws {
        await chartService.yield(candle.map())
    }

    private func defaultSubscriptions(for address: String) -> [HyperliquidSubscription] {
        [.clearinghouseState(user: address), .openOrders(user: address)]
    }

    private func subscribe(_ subscriptions: [HyperliquidSubscription]) async throws {
        try await withThrowingTaskGroup(of: Void.self) { group in
            for subscription in subscriptions {
                group.addTask {
                    try await self.send(HyperliquidRequest(method: .subscribe, subscription: subscription))
                }
            }
            try await group.waitForAll()
        }
    }

    private func send(_ request: some Encodable) async throws {
        try await webSocket.send(try encoder.encode(request).encodeString())
    }
}
